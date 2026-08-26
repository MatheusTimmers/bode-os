const CLINT: usize = 0x0200_0000;
const MTIME_OFFSET: usize = 0xbff8;
const MTIME_CMP_OFFSET: usize = 0x4000;

/// Frequência do contador `mtime` na placa `virt` do QEMU: 10 MHz.
/// Um tick = 100 ns.
pub const CLOCK_HZ: u64 = 10_000_000;

/// Intervalo entre interrupções de timer: 100 ms.
pub const TICK: u64 = CLOCK_HZ / 10;

pub fn read_mtime() -> u64 {
    let low: *const u32 = (CLINT + MTIME_OFFSET) as *const u32;
    let high: *const u32 = (CLINT + MTIME_OFFSET + 4) as *const u32;

    let mut low_value: u32;
    let mut high_value: u32;
    let mut aux: u32;

    loop {
        unsafe {
            aux = high.read_volatile();
            low_value = low.read_volatile();
            high_value = high.read_volatile();

            if high_value == aux {
               break;
            }

        }
    }

    (high_value as u64) << 32 | (low_value as u64)
}

pub unsafe fn write_mtimecmp(value: u64) {
    let low: *mut u32 = (CLINT + MTIME_CMP_OFFSET) as *mut u32;
    let high: *mut u32 = (CLINT + MTIME_CMP_OFFSET + 4) as *mut u32;

    unsafe {
        low.write_volatile(0xFFFF_FFFF);
        high.write_volatile((value >> 32) as u32);
        low.write_volatile(value as u32);
    }
}

/// Programa a próxima interrupção de timer para `delta` ticks a partir de agora.
///
/// Diferente de `write_mtimecmp`, esta função é segura: o instante resultante é
/// sempre `agora + delta`, nunca um valor arbitrário no passado. O grau de
/// liberdade que torna a escrita crua perigosa não existe aqui.
pub fn schedule_next_tick(delta: u64) {
    let next = read_mtime() + delta;
    unsafe { write_mtimecmp(next) };
}
