use core::alloc::Layout;
use core::ptr::NonNull;

use bode_os::allocators::Allocator;
use bode_os::allocators::buddy::Buddy;
use bode_os::allocators::hellfire::Hellfire;
use bode_os::allocators::tlsf::Tlsf;

use crate::arch::cpu;
use crate::memory::heap;
use crate::println;

const LIVE_BLOCKS: [usize; 5] = [8, 16, 32, 64, 128];
const MAX_LIVE: usize = 128;
const REPETITIONS: u32 = 5;
const OVERHEAD_SAMPLES: u32 = 32;
const SMALL: usize = 16;
const LARGE: usize = 32;
const WORD_ALIGN: usize = 4;
const BUDDY_ROOTS: [usize; 7] = [16, 32, 64, 256, 512, 1024, 2048];
const BUDDY_LARGEST: usize = 4096;
const BUDDY_MAX_DEPTH: usize = 8;

type Blocks = [Option<NonNull<u8>>; MAX_LIVE];

struct Range {
    smallest: u32,
    largest: u32,
}

impl Range {
    const fn new() -> Self {
        Self {
            smallest: u32::MAX,
            largest: 0,
        }
    }

    fn record(&mut self, cycles: u32) {
        self.smallest = self.smallest.min(cycles);
        self.largest = self.largest.max(cycles);
    }
}

#[inline(never)]
unsafe fn fresh_hellfire() -> Hellfire {
    Hellfire::new(unsafe { heap::fresh_arena() })
}

#[inline(never)]
unsafe fn fresh_tlsf() -> Tlsf {
    Tlsf::new(unsafe { heap::fresh_arena() })
}

#[inline(never)]
unsafe fn fresh_buddy() -> Buddy {
    Buddy::new(unsafe { heap::fresh_arena() })
}

#[inline(never)]
fn invalid<T>() -> T {
    panic!("pior caso invalido")
}

fn layout(size: usize) -> Layout {
    Layout::from_size_align(size, WORD_ALIGN)
        .ok()
        .unwrap_or_else(invalid)
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

#[inline(never)]
fn timed_alloc(
    allocator: &mut dyn Allocator,
    request: Layout,
    overhead: u32,
) -> (NonNull<u8>, u32) {
    let start = cpu::read_mcycle();
    let block = allocator.alloc(request);
    let end = cpu::read_mcycle();
    let block = block.unwrap_or_else(invalid);
    (block, end.wrapping_sub(start).saturating_sub(overhead))
}

#[inline(never)]
fn timed_free(
    allocator: &mut dyn Allocator,
    block: NonNull<u8>,
    request: Layout,
    overhead: u32,
) -> u32 {
    let start = cpu::read_mcycle();
    unsafe { allocator.dealloc(block, request) };
    let end = cpu::read_mcycle();
    end.wrapping_sub(start).saturating_sub(overhead)
}

#[inline(never)]
fn fill(allocator: &mut dyn Allocator, count: usize) -> Blocks {
    let mut blocks: Blocks = [None; MAX_LIVE];
    for slot in blocks.iter_mut().take(count) {
        *slot = Some(allocator.alloc(layout(SMALL)).unwrap_or_else(invalid));
    }
    blocks
}

#[inline(never)]
fn punch_holes(allocator: &mut dyn Allocator, blocks: &Blocks, count: usize) {
    for block in blocks.iter().take(count).skip(1).step_by(2) {
        let block = block.unwrap_or_else(invalid);
        unsafe { allocator.dealloc(block, layout(SMALL)) };
    }
}

#[inline(never)]
fn alloc_past_holes(
    allocator: &mut dyn Allocator,
    count: usize,
    overhead: u32,
) -> (usize, usize, u32) {
    let blocks = fill(allocator, count);
    let last = blocks[count - 1].unwrap_or_else(invalid).as_ptr().addr();
    punch_holes(allocator, &blocks, count);
    let (block, cycles) = timed_alloc(allocator, layout(LARGE), overhead);
    (block.as_ptr().addr(), last, cycles)
}

#[inline(never)]
fn free_between_holes(allocator: &mut dyn Allocator, count: usize, overhead: u32) -> u32 {
    let blocks = fill(allocator, count);
    punch_holes(allocator, &blocks, count);
    let block = blocks[count / 2].unwrap_or_else(invalid);
    timed_free(allocator, block, layout(SMALL), overhead)
}

#[inline(never)]
fn free_between_used(allocator: &mut dyn Allocator, count: usize, overhead: u32) -> u32 {
    let blocks = fill(allocator, count);
    let block = blocks[count / 2].unwrap_or_else(invalid);
    timed_free(allocator, block, layout(SMALL), overhead)
}

#[inline(never)]
fn print_range(allocator: &str, operation: &str, parameter: usize, range: &Range) {
    println!(
        "worst,{},{},{},{},{}",
        allocator, operation, parameter, range.smallest, range.largest
    );
}

unsafe fn scale_hellfire(overhead: u32) {
    for count in LIVE_BLOCKS {
        let mut range = Range::new();
        for _ in 0..REPETITIONS {
            let mut allocator = unsafe { fresh_hellfire() };
            let (block, last, cycles) = alloc_past_holes(&mut allocator, count, overhead);
            assert!(block >= last, "hellfire: reaproveitou um buraco");
            range.record(cycles);
        }
        print_range("hellfire", "alloc_percorre_lista", count, &range);
    }
}

unsafe fn scale_tlsf(overhead: u32) {
    for count in LIVE_BLOCKS {
        let mut alloc = Range::new();
        let mut merging = Range::new();
        let mut plain = Range::new();
        for _ in 0..REPETITIONS {
            let mut allocator = unsafe { fresh_tlsf() };
            alloc.record(alloc_past_holes(&mut allocator, count, overhead).2);
            let mut allocator = unsafe { fresh_tlsf() };
            merging.record(free_between_holes(&mut allocator, count, overhead));
            let mut allocator = unsafe { fresh_tlsf() };
            plain.record(free_between_used(&mut allocator, count, overhead));
        }
        print_range("tlsf", "alloc_com_buracos", count, &alloc);
        print_range("tlsf", "free_duas_juncoes", count, &merging);
        print_range("tlsf", "free_sem_juncao", count, &plain);
    }
}

unsafe fn scale_buddy(overhead: u32) {
    for count in LIVE_BLOCKS {
        let mut alloc = Range::new();
        let mut plain = Range::new();
        for _ in 0..REPETITIONS {
            let mut allocator = unsafe { fresh_buddy() };
            alloc.record(alloc_past_holes(&mut allocator, count, overhead).2);
            let mut allocator = unsafe { fresh_buddy() };
            plain.record(free_between_used(&mut allocator, count, overhead));
        }
        print_range("buddy", "alloc_com_buracos", count, &alloc);
        print_range("buddy", "free_sem_juncao", count, &plain);
    }
}

unsafe fn depth_buddy(overhead: u32) {
    for depth in 0..=BUDDY_MAX_DEPTH {
        let mut split = Range::new();
        let mut merge = Range::new();
        let size = BUDDY_LARGEST >> depth;

        for _ in 0..REPETITIONS {
            let arena = unsafe { heap::fresh_arena() };
            let base = arena.as_ptr().addr();
            let mut allocator = Buddy::new(arena);
            for root in BUDDY_ROOTS {
                allocator.alloc(layout(root)).unwrap_or_else(invalid);
            }

            let (block, cycles) = timed_alloc(&mut allocator, layout(size), overhead);
            assert!(
                block.as_ptr().addr() - base == BUDDY_LARGEST,
                "buddy: nao dividiu a maior raiz"
            );
            split.record(cycles);
            merge.record(timed_free(&mut allocator, block, layout(size), overhead));

            let whole = allocator
                .alloc(layout(BUDDY_LARGEST))
                .unwrap_or_else(invalid);
            assert!(
                whole.as_ptr().addr() - base == BUDDY_LARGEST,
                "buddy: nao juntou ate a raiz"
            );
        }

        print_range("buddy", "alloc_divisoes", depth, &split);
        print_range("buddy", "free_juncoes", depth, &merge);
    }
}

pub unsafe fn run() -> ! {
    let overhead = measurement_overhead();
    println!("worst,overhead,{}", overhead);
    unsafe {
        scale_hellfire(overhead);
        scale_tlsf(overhead);
        scale_buddy(overhead);
        depth_buddy(overhead);
    }
    println!("worst,fim");
    cpu::halt()
}
