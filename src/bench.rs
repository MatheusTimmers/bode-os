use core::alloc::Layout;
use core::ptr::NonNull;

use bode_os::allocators::Allocator;
use bode_os::allocators::buddy::Buddy;
use bode_os::allocators::bump::Bump;
use bode_os::allocators::pool::Pool;
use bode_os::allocators::tlsf::Tlsf;

use crate::arch::cpu;
use crate::memory::heap;
use crate::println;

const OPERATIONS: u32 = 2000;
const SLOTS: usize = 32;
const SEED: u32 = 0x1BAD_B002;
const OVERHEAD_SAMPLES: u32 = 32;
const PROCESS_SIZES: [usize; 3] = [512, 1024, 2048];
const SMALL_MIN: usize = 8;
const SMALL_RANGE: u32 = 249;
const SMALL_ALIGN: usize = 8;

struct Scenario {
    name: &'static str,
    layout: fn(u32) -> Layout,
}

const SCENARIOS: [Scenario; 2] = [
    Scenario {
        name: "processos",
        layout: process_layout,
    },
    Scenario {
        name: "pequenos",
        layout: small_layout,
    },
];

fn process_layout(random: u32) -> Layout {
    let index = random as usize % PROCESS_SIZES.len();
    let size = PROCESS_SIZES[index];
    Layout::from_size_align(size, size).expect("layout invalido")
}

fn small_layout(random: u32) -> Layout {
    let size = SMALL_MIN + (random % SMALL_RANGE) as usize;
    Layout::from_size_align(size, SMALL_ALIGN).expect("layout invalido")
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

fn run_scenario<A: Allocator>(mut allocator: A, name: &str, scenario: &Scenario, overhead: u32) {
    let mut slots: [Option<(NonNull<u8>, Layout)>; SLOTS] = [None; SLOTS];
    let mut random = SEED;
    let mut alloc_stats = Stats::new();
    let mut free_stats = Stats::new();
    let mut failed: u32 = 0;
    let mut live_bytes: usize = 0;
    let mut peak_bytes: usize = 0;

    for _ in 0..OPERATIONS {
        let value = next_random(&mut random);
        let slot = (value >> 8) as usize % SLOTS;

        match slots[slot] {
            None => {
                let layout = (scenario.layout)(value >> 16);
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
        &alloc_stats,
        &free_stats,
        failed,
        peak_bytes,
    );
}

#[inline(never)]
fn report(scenario: &str, name: &str, alloc: &Stats, free: &Stats, failed: u32, peak_bytes: usize) {
    println!(
        "bench,{},{},{},{},{},{},{},{},{},{},{},{}",
        scenario,
        name,
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
        "bench,cenario,alocador,alloc_n,alloc_min,alloc_media,alloc_max,free_n,free_min,free_media,free_max,falhas,pico_bytes"
    );

    for scenario in &SCENARIOS {
        unsafe {
            run_scenario(Bump::new(heap::fresh_arena()), "bump", scenario, overhead);
            run_scenario(Pool::new(heap::fresh_arena()), "pool", scenario, overhead);
            run_scenario(Buddy::new(heap::fresh_arena()), "buddy", scenario, overhead);
            run_scenario(Tlsf::new(heap::fresh_arena()), "tlsf", scenario, overhead);
        }
    }

    println!("bench,fim");
    cpu::halt()
}
