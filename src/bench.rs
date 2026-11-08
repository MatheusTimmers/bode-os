use core::alloc::Layout;
use core::ptr::NonNull;

use bode_os::allocators::Allocator;
use bode_os::allocators::buddy::Buddy;
use bode_os::allocators::bump::Bump;
use bode_os::allocators::hellfire::Hellfire;
use bode_os::allocators::pool::Pool;
use bode_os::allocators::tlsf::Tlsf;

use crate::arch::cpu;
use crate::memory::heap;
use crate::println;

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
const SPLIT_SIZE: usize = 16;

struct Scenario {
    name: &'static str,
    slots: usize,
    budget: usize,
    word_aligned: bool,
    layout: fn(u32) -> Layout,
}

const SCENARIOS: [Scenario; 4] = [
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
    Scenario {
        name: "divisao",
        slots: 1,
        budget: NO_BUDGET,
        word_aligned: true,
        layout: split_layout,
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

fn split_layout(_random: u32) -> Layout {
    Layout::from_size_align(SPLIT_SIZE, WORD_ALIGN).expect("layout invalido")
}

#[derive(Clone, Copy)]
struct Stats {
    count: u32,
    total: u32,
    min: u32,
    max: u32,
}

impl Stats {
    const fn new() -> Self {
        Self {
            count: 0,
            total: 0,
            min: u32::MAX,
            max: 0,
        }
    }

    fn record(&mut self, cycles: u32) {
        self.count += 1;
        self.total += cycles;
        self.min = self.min.min(cycles);
        self.max = self.max.max(cycles);
    }

    fn mean(&self) -> u32 {
        if self.count == 0 {
            return 0;
        }
        self.total / self.count
    }
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

fn run_scenario<A: Allocator>(
    mut allocator: A,
    name: &str,
    scenario: &Scenario,
    seed: u32,
    overhead: u32,
) {
    let mut slots: [Option<(NonNull<u8>, Layout)>; SLOTS] = [None; SLOTS];
    let mut random = seed;
    let mut alloc_stats = Stats::new();
    let mut free_stats = Stats::new();
    let mut failed: u32 = 0;
    let mut live_bytes: usize = 0;
    let mut peak_bytes: usize = 0;

    for _ in 0..OPERATIONS {
        let value = next_random(&mut random);
        let slot = (value >> 8) as usize % scenario.slots;

        match slots[slot] {
            None => {
                let layout = (scenario.layout)(value >> 16);
                if live_bytes + layout.size() > scenario.budget {
                    continue;
                }

                let start = cpu::read_mcycle();
                let block = allocator.alloc(layout);
                let end = cpu::read_mcycle();

                match block {
                    Some(block) => {
                        alloc_stats.record(end.wrapping_sub(start).saturating_sub(overhead));
                        slots[slot] = Some((block, layout));
                        live_bytes += layout.size();
                        peak_bytes = peak_bytes.max(live_bytes);
                    }
                    None => failed += 1,
                }
            }
            Some((block, layout)) => {
                let start = cpu::read_mcycle();
                unsafe { allocator.dealloc(block, layout) };
                let end = cpu::read_mcycle();

                free_stats.record(end.wrapping_sub(start).saturating_sub(overhead));
                slots[slot] = None;
                live_bytes -= layout.size();
            }
        }
    }

    report(
        scenario.name,
        name,
        seed,
        &alloc_stats,
        &free_stats,
        failed,
        peak_bytes,
    );
}

#[inline(never)]
fn report(
    scenario: &str,
    name: &str,
    seed: u32,
    alloc: &Stats,
    free: &Stats,
    failed: u32,
    peak_bytes: usize,
) {
    println!(
        "bench,{},{},{:08x},{},{},{},{},{},{},{},{},{},{}",
        scenario,
        name,
        seed,
        alloc.count,
        alloc.min,
        alloc.mean(),
        alloc.max,
        free.count,
        free.min,
        free.mean(),
        free.max,
        failed,
        peak_bytes
    );
}

/// # Safety
/// Chamar uma vez, com as interrupções desligadas, antes de `heap::init`.
pub unsafe fn run() -> ! {
    let overhead = measurement_overhead();
    println!("bench,overhead,{}", overhead);
    println!(
        "bench,cenario,alocador,semente,alloc_n,alloc_min,alloc_media,alloc_max,free_n,free_min,free_media,free_max,falhas,pico_bytes"
    );

    for scenario in &SCENARIOS {
        for &seed in &SEEDS {
            unsafe {
                run_scenario(
                    Bump::new(heap::fresh_arena()),
                    "bump",
                    scenario,
                    seed,
                    overhead,
                );
                run_scenario(
                    Pool::new(heap::fresh_arena()),
                    "pool",
                    scenario,
                    seed,
                    overhead,
                );
                run_scenario(
                    Buddy::new(heap::fresh_arena()),
                    "buddy",
                    scenario,
                    seed,
                    overhead,
                );
                run_scenario(
                    Tlsf::new(heap::fresh_arena()),
                    "tlsf",
                    scenario,
                    seed,
                    overhead,
                );
                if scenario.word_aligned {
                    run_scenario(
                        Hellfire::new(heap::fresh_arena()),
                        "hellfire",
                        scenario,
                        seed,
                        overhead,
                    );
                }
            }
        }
    }

    println!("bench,fim");
    cpu::halt()
}
