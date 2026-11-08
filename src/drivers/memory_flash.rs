pub fn read(base: usize, offset: usize, buffer: &mut [u8]) {
    let start = (base + offset) as *const u8;

    for (index, byte) in buffer.iter_mut().enumerate() {
        *byte = unsafe { start.add(index).read_volatile() };
    }
}
