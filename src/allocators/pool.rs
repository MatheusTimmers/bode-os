use core::alloc::Layout;
use core::ptr::NonNull;

use super::Allocator;
use super::bump::Bump;

const MIN_SHIFT: usize = 4;
const CLASS_COUNT: usize = 7;

struct FreeBlock {
    next: Option<NonNull<FreeBlock>>,
}
pub struct Pool {
    free: [Option<NonNull<FreeBlock>>; CLASS_COUNT],
    fresh: Bump,
}

impl Pool {
    pub fn new(heap: &'static mut [u8]) -> Self {
        Self {
            free: [None; CLASS_COUNT],
            fresh: Bump::new(heap),
        }
    }
}

impl Allocator for Pool {
    fn alloc(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        let index = class_index(layout)?;

        match self.free[index] {
            Some(block) => {
                let next = unsafe { block.as_ref().next };
                self.free[index] = next;
                Some(block.cast())
            }
            None => {
                let block_size = 1 << (index + MIN_SHIFT);
                let lay = Layout::from_size_align(block_size, block_size).ok()?;
                self.fresh.alloc(lay)
            }
        }
    }

    unsafe fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
        let Some(index) = class_index(layout) else {
            return;
        };

        let block = ptr.cast::<FreeBlock>();
        unsafe {
            block.write(FreeBlock {
                next: self.free[index],
            })
        };
        self.free[index] = Some(block);
    }
}

fn class_index(layout: Layout) -> Option<usize> {
    let needed = layout.size().max(layout.align());
    let at_least_smallest = needed.max(1 << MIN_SHIFT);
    let block_size = at_least_smallest.checked_next_power_of_two()?;
    let exponent = block_size.trailing_zeros() as usize;
    let index = exponent - MIN_SHIFT;

    if index < CLASS_COUNT {
        Some(index)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::Pool;

    contract_tests!(Pool::new);
    reuse_tests!(Pool::new);
}
