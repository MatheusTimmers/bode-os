#[cfg(feature = "switch-bench-no-pmp")]
use core::ops::Range;

use crate::arch::cpu;
use crate::kernel::task;
use crate::memory::heap;
use crate::{println, user};

const SWITCH_SAMPLES: u32 = 1000;
const WARMUP_TRAPS: u32 = 16;
const TASK_BLOCK: usize = 1024;
const SHARED_BLOCK: usize = 2 * TASK_BLOCK;
const ZERO_SIZES: [usize; 4] = [512, 1024, 2048, 4096];
const ZERO_SAMPLES: u32 = 200;

struct Stats {
    count: u32,
    smallest: u32,
    largest: u32,
    total: u64,
}

impl Stats {
    const fn new() -> Self {
        Self {
            count: 0,
            smallest: u32::MAX,
            largest: 0,
            total: 0,
        }
    }

    fn record(&mut self, cycles: u32) {
        self.count += 1;
        self.smallest = self.smallest.min(cycles);
        self.largest = self.largest.max(cycles);
        self.total += cycles as u64;
    }

    fn mean(&self) -> u64 {
        self.total / self.count as u64
    }
}

struct Switches {
    last: u32,
    traps: u32,
    stats: Stats,
}

static mut SWITCHES: Switches = Switches {
    last: 0,
    traps: 0,
    stats: Stats::new(),
};

extern "C" fn task_yields(_: usize, _: usize) -> ! {
    loop {
        user::sys_yield();
    }
}

#[allow(clippy::deref_addrof)]
pub unsafe fn on_trap() {
    let now = cpu::read_mcycle();
    let switches = unsafe { &mut *(&raw mut SWITCHES) };

    if switches.traps > WARMUP_TRAPS {
        switches.stats.record(now.wrapping_sub(switches.last));
    }
    switches.traps += 1;

    if switches.stats.count == SWITCH_SAMPLES {
        report_switches(&switches.stats);
        cpu::halt();
    }

    switches.last = cpu::read_mcycle();
}

fn report_switches(stats: &Stats) {
    let variant = if cfg!(feature = "switch-bench-no-pmp") {
        "sem_pmp"
    } else {
        "com_pmp"
    };
    println!(
        "switch,{},{},{},{},{}",
        variant,
        stats.count,
        stats.smallest,
        stats.mean(),
        stats.largest
    );
    println!("switch,fim");
}

fn measure_zeroing(size: usize) {
    let region = unsafe { heap::alloc_region(size) }.expect("sem memoria para medir");
    let start = region.start as *mut u8;
    let overhead = cpu::read_mcycle()
        .wrapping_sub(cpu::read_mcycle())
        .wrapping_neg();
    let mut stats = Stats::new();

    for _ in 0..ZERO_SAMPLES {
        let before = cpu::read_mcycle();
        unsafe { start.write_bytes(0, size) };
        let after = cpu::read_mcycle();
        stats.record(after.wrapping_sub(before).saturating_sub(overhead));
    }

    println!(
        "zero,{},{},{},{},{}",
        size,
        stats.count,
        stats.smallest,
        stats.mean(),
        stats.largest
    );
    unsafe { heap::free_region(region) };
}

#[cfg(feature = "switch-bench-no-pmp")]
pub fn shared_block(task_region: Range<usize>) -> Range<usize> {
    let start = task_region.start & !(SHARED_BLOCK - 1);
    start..start + SHARED_BLOCK
}

pub unsafe fn prepare() {
    for size in ZERO_SIZES {
        measure_zeroing(size);
    }

    let block = unsafe { heap::alloc_region(SHARED_BLOCK) }.expect("sem memoria para as tasks");
    let middle = block.start + TASK_BLOCK;

    for region in [block.start..middle, middle..block.end] {
        unsafe { task::spawn_process(task_yields, region, [0, 0]) }.expect("sem vaga para a task");
    }
}
