use core::alloc::Layout;
use core::ptr::NonNull;

use bode_os::allocators::Allocator;
use bode_os::allocators::buddy::Buddy;
use bode_os::allocators::bump::Bump;
use bode_os::allocators::hellfire::Hellfire;
use bode_os::allocators::pool::Pool;
use bode_os::allocators::tlsf::Tlsf;

use crate::arch::{cpu, pmp};
use crate::memory::heap;
use crate::{print, println};

const OPERATIONS: u32 = 1000;
const SLOTS: usize = 32;
const SEEDS: [u32; 5] = [
    0x1BAD_B002,
    0xDEAD_BEEF,
    0x0BAD_F00D,
    0x1234_5678,
    0xC001_D00D,
];
const OVERHEAD_SAMPLES: u32 = 32;
const NO_BUDGET: usize = usize::MAX;
const LIVE_BUDGET: usize = 3 * 1024;
const TASK_SIZES: [usize; 3] = [256, 512, 1024];
const PRESSURE_SIZES: [usize; 3] = [512, 1024, 2048];
const SMALL_MIN: usize = 8;
const SMALL_RANGE: u32 = 249;
const WORD_ALIGN: usize = 4;
const HIST_BINS: usize = 20;
const HIST_WIDTH: u32 = 32;
const MIN_POWER_BLOCK: usize = 16;
const TLSF_GRANULE: usize = 8;
const TLSF_HEADER: usize = 8;
const HELLFIRE_WORD: usize = 4;
const HELLFIRE_HEADER: usize = 8;
const PMP_SAMPLES: u32 = 1000;
const PMP_REGION_SIZE: usize = 1024;
const ARENA_BYTES: usize = 8 * 1024;
const POOL_LARGEST_CLASS: usize = 1024;

struct Scenario {
    name: &'static str,
    slots: usize,
    budget: usize,
    word_aligned: bool,
    layout: fn(u32) -> Layout,
}

const SCENARIOS: [Scenario; 3] = [
    Scenario {
        name: "tarefas",
        slots: SLOTS,
        budget: LIVE_BUDGET,
        word_aligned: false,
        layout: task_layout,
    },
    Scenario {
        name: "pequenos",
        slots: SLOTS,
        budget: LIVE_BUDGET,
        word_aligned: true,
        layout: small_layout,
    },
    Scenario {
        name: "pressao",
        slots: SLOTS,
        budget: NO_BUDGET,
        word_aligned: false,
        layout: pressure_layout,
    },
];

fn aligned_to_size(sizes: &[usize], random: u32) -> Layout {
    let size = sizes[random as usize % sizes.len()];
    Layout::from_size_align(size, size).expect("layout invalido")
}

fn task_layout(random: u32) -> Layout {
    aligned_to_size(&TASK_SIZES, random)
}

fn pressure_layout(random: u32) -> Layout {
    aligned_to_size(&PRESSURE_SIZES, random)
}

fn small_layout(random: u32) -> Layout {
    let size = SMALL_MIN + (random % SMALL_RANGE) as usize;
    Layout::from_size_align(size, WORD_ALIGN).expect("layout invalido")
}

#[derive(Clone, Copy)]
struct Stats {
    count: u32,
    total: u32,
    min: u32,
    max: u32,
    bins: [u32; HIST_BINS],
}

impl Stats {
    const fn new() -> Self {
        Self {
            count: 0,
            total: 0,
            min: u32::MAX,
            max: 0,
            bins: [0; HIST_BINS],
        }
    }

    fn record(&mut self, cycles: u32) {
        self.count += 1;
        self.total += cycles;
        self.min = self.min.min(cycles);
        self.max = self.max.max(cycles);
        let bin = (cycles / HIST_WIDTH) as usize;
        self.bins[bin.min(HIST_BINS - 1)] += 1;
    }

    fn smallest(&self) -> u32 {
        if self.count == 0 {
            return 0;
        }
        self.min
    }

    fn mean(&self) -> u32 {
        if self.count == 0 {
            return 0;
        }
        self.total / self.count
    }
}

fn exact_size(layout: Layout) -> usize {
    layout.size()
}

fn power_of_two_block(layout: Layout) -> usize {
    layout
        .size()
        .max(layout.align())
        .max(MIN_POWER_BLOCK)
        .next_power_of_two()
}

fn tlsf_block(layout: Layout) -> usize {
    layout
        .size()
        .max(TLSF_GRANULE)
        .next_multiple_of(TLSF_GRANULE)
        + TLSF_HEADER
}

fn hellfire_block(layout: Layout) -> usize {
    layout.size().next_multiple_of(HELLFIRE_WORD) + HELLFIRE_HEADER
}

fn any_request(_layout: Layout) -> bool {
    true
}

fn pool_class_exists(layout: Layout) -> bool {
    layout.size().max(layout.align()) <= POOL_LARGEST_CLASS
}

fn next_random(state: &mut u32) -> u32 {
    let mut value = *state;
    value ^= value << 13;
    value ^= value >> 17;
    value ^= value << 5;
    *state = value;
    value
}

fn measurement_overhead() -> u32 {
    let mut smallest = u32::MAX;
    for _ in 0..OVERHEAD_SAMPLES {
        let start = cpu::read_mcycle();
        let end = cpu::read_mcycle();
        smallest = smallest.min(end.wrapping_sub(start));
    }
    smallest
}

struct Metrics {
    alloc: Stats,
    free: Stats,
    failed: Stats,
    refused: u32,
    fragmented: u32,
    peak_requested: usize,
    peak_consumed: usize,
}

#[inline(never)]
fn run_scenario(
    allocator: &mut dyn Allocator,
    name: &str,
    consumed: fn(Layout) -> usize,
    supported: fn(Layout) -> bool,
    scenario: &Scenario,
    seed: u32,
    overhead: u32,
) {
    let mut slots: [Option<(Option<NonNull<u8>>, Layout)>; SLOTS] = [None; SLOTS];
    let mut random = seed;
    let mut metrics = Metrics {
        alloc: Stats::new(),
        free: Stats::new(),
        failed: Stats::new(),
        refused: 0,
        fragmented: 0,
        peak_requested: 0,
        peak_consumed: 0,
    };
    let mut intended_bytes: usize = 0;
    let mut requested_bytes: usize = 0;
    let mut consumed_bytes: usize = 0;

    for _ in 0..OPERATIONS {
        let value = next_random(&mut random);
        let slot = (value >> 8) as usize % scenario.slots;

        match slots[slot] {
            None => {
                let layout = (scenario.layout)(value >> 16);
                if intended_bytes + layout.size() > scenario.budget {
                    continue;
                }

                let start = cpu::read_mcycle();
                let block = allocator.alloc(layout);
                let end = cpu::read_mcycle();
                let cycles = end.wrapping_sub(start).saturating_sub(overhead);

                intended_bytes += layout.size();
                slots[slot] = Some((block, layout));
                if block.is_some() {
                    metrics.alloc.record(cycles);
                    requested_bytes += layout.size();
                    consumed_bytes += consumed(layout);
                    metrics.peak_requested = metrics.peak_requested.max(requested_bytes);
                    metrics.peak_consumed = metrics.peak_consumed.max(consumed_bytes);
                } else if supported(layout) {
                    metrics.failed.record(cycles);
                    if ARENA_BYTES - consumed_bytes >= consumed(layout) {
                        metrics.fragmented += 1;
                    }
                } else {
                    metrics.refused += 1;
                }
            }
            Some((Some(block), layout)) => {
                let start = cpu::read_mcycle();
                unsafe { allocator.dealloc(block, layout) };
                let end = cpu::read_mcycle();

                metrics
                    .free
                    .record(end.wrapping_sub(start).saturating_sub(overhead));
                intended_bytes -= layout.size();
                requested_bytes -= layout.size();
                consumed_bytes -= consumed(layout);
                slots[slot] = None;
            }
            Some((None, layout)) => {
                intended_bytes -= layout.size();
                slots[slot] = None;
            }
        }
    }

    report(scenario.name, name, seed, &metrics);
}

#[inline(never)]
fn report(scenario: &str, name: &str, seed: u32, metrics: &Metrics) {
    let Metrics {
        alloc,
        free,
        failed,
        refused,
        fragmented,
        peak_requested,
        peak_consumed,
    } = metrics;
    println!(
        "bench,{},{},{:08x},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
        scenario,
        name,
        seed,
        alloc.count,
        alloc.smallest(),
        alloc.mean(),
        alloc.max,
        alloc.total,
        free.count,
        free.smallest(),
        free.mean(),
        free.max,
        free.total,
        failed.count,
        failed.mean(),
        failed.max,
        peak_requested,
        peak_consumed,
        refused,
        fragmented
    );
    print_bins(scenario, name, seed, "alloc", alloc);
    print_bins(scenario, name, seed, "free", free);
}

#[inline(never)]
fn print_bins(scenario: &str, name: &str, seed: u32, kind: &str, stats: &Stats) {
    print!("hist,{},{},{:08x},{}", scenario, name, seed, kind);
    for count in stats.bins {
        print!(",{}", count);
    }
    println!();
}

#[inline(never)]
fn measure_pmp_switch(overhead: u32) {
    let start = heap::fresh_arena_start();
    let mut stats = Stats::new();

    for _ in 0..PMP_SAMPLES {
        let before = cpu::read_mcycle();
        pmp::allow_region(start..start + PMP_REGION_SIZE);
        let after = cpu::read_mcycle();
        stats.record(after.wrapping_sub(before).saturating_sub(overhead));
    }

    println!(
        "pmp,allow_region,{},{},{},{}",
        stats.count,
        stats.smallest(),
        stats.mean(),
        stats.max
    );
}

/// # Safety
/// Chamar uma vez, com as interrupções desligadas, antes de `heap::init`.
pub unsafe fn run() -> ! {
    let overhead = measurement_overhead();
    println!("bench,overhead,{}", overhead);
    println!(
        "bench,cenario,alocador,semente,alloc_n,alloc_min,alloc_media,alloc_max,alloc_total,free_n,free_min,free_media,free_max,free_total,falhas,falha_media,falha_max,pico_pedido,pico_consumido,recusas,falhas_fragmentacao"
    );

    for scenario in &SCENARIOS {
        for &seed in &SEEDS {
            unsafe {
                run_scenario(
                    &mut Bump::new(heap::fresh_arena()),
                    "bump",
                    exact_size,
                    any_request,
                    scenario,
                    seed,
                    overhead,
                );
                run_scenario(
                    &mut Pool::new(heap::fresh_arena()),
                    "pool",
                    power_of_two_block,
                    pool_class_exists,
                    scenario,
                    seed,
                    overhead,
                );
                run_scenario(
                    &mut Buddy::new(heap::fresh_arena()),
                    "buddy",
                    power_of_two_block,
                    any_request,
                    scenario,
                    seed,
                    overhead,
                );
                run_scenario(
                    &mut Tlsf::new(heap::fresh_arena()),
                    "tlsf",
                    tlsf_block,
                    any_request,
                    scenario,
                    seed,
                    overhead,
                );
                if scenario.word_aligned {
                    run_scenario(
                        &mut Hellfire::new(heap::fresh_arena()),
                        "hellfire",
                        hellfire_block,
                        any_request,
                        scenario,
                        seed,
                        overhead,
                    );
                }
            }
        }
    }

    measure_pmp_switch(overhead);
    println!("bench,fim");
    cpu::halt()
}
