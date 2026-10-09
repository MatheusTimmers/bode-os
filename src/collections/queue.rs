pub struct Queue<T, const N: usize> {
    head: usize,
    tail: usize,
    data: [Option<T>; N],
}

impl<T: Copy, const N: usize> Queue<T, N> {
    pub const fn new() -> Self {
        const { assert!(N > 1) }
        Self {
            head: 0,
            tail: 0,
            data: [None; N],
        }
    }

    pub fn is_full(&self) -> bool {
        self.head == (self.tail + 1) % N
    }

    pub fn is_empty(&self) -> bool {
        self.head == self.tail
    }

    pub fn push(&mut self, value: T) -> Result<(), T> {
        if self.is_full() {
            return Err(value);
        }

        self.data[self.tail] = Some(value);
        self.tail = (self.tail + 1) % N;

        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let value = self.data[self.head].take();

        self.head = (self.head + 1) % N;

        value
    }
}

impl<T: Copy, const N: usize> Default for Queue<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::Queue;

    #[test]
    fn pop_on_empty_returns_none() {
        let mut queue: Queue<usize, 4> = Queue::new();
        assert!(queue.is_empty());
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn pops_in_fifo_order() {
        let mut queue: Queue<usize, 4> = Queue::new();
        queue.push(1).unwrap();
        queue.push(2).unwrap();
        queue.push(3).unwrap();

        assert_eq!(queue.pop(), Some(1));
        assert_eq!(queue.pop(), Some(2));
        assert_eq!(queue.pop(), Some(3));
        assert_eq!(queue.pop(), None);
    }

    #[test]
    fn holds_n_minus_one_items() {
        let mut queue: Queue<usize, 4> = Queue::new();
        for i in 0..3 {
            queue.push(i).unwrap();
        }

        assert!(queue.is_full());
        assert_eq!(queue.push(99), Err(99));
    }

    #[test]
    fn wraps_around_the_buffer() {
        let mut queue: Queue<usize, 4> = Queue::new();
        for round in 0..10 {
            queue.push(round).unwrap();
            queue.push(round + 100).unwrap();
            assert_eq!(queue.pop(), Some(round));
            assert_eq!(queue.pop(), Some(round + 100));
        }
        assert!(queue.is_empty());
    }
}
