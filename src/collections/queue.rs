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
