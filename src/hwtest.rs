use core::arch::asm;
use core::ops::Range;

use crate::kernel::{syscall, task};
use crate::memory::{heap, mem};
use crate::program::interpreter;
use crate::{board, user};

const BLOCK_SIZE: usize = 512;
const VICTIM_SECRET: u32 = 0xCAFE_F00D;
const VICTIM_ROUNDS: usize = 3;
const SPIN_ITERATIONS: u32 = 4_000_000;
const WORD: usize = size_of::<u32>();
const MISALIGNED_TAIL: usize = 2;
const FINAL_ROUNDS: usize = 6;
const CHILD_SIZES: [usize; 9] = [512, 1024, 2048, 2048, 512, 1024, 1024, 2048, 2048];

type Test = extern "C" fn(usize, usize) -> !;

extern "C" fn spinner(_: usize, _: usize) -> ! {
    for iteration in 0..SPIN_ITERATIONS {
        core::hint::black_box(iteration);
    }
    user::sys_write(b"hw-test: task longa terminou\n");
    user::sys_exit();
}

extern "C" fn victim(block: usize, _: usize) -> ! {
    let secret = block as *mut u32;
    unsafe { secret.write_volatile(VICTIM_SECRET) };

    for _ in 0..VICTIM_ROUNDS {
        user::sys_yield();
    }

    let intact = unsafe { secret.read_volatile() } == VICTIM_SECRET;
    if intact {
        user::sys_write(b"hw-test: vitima intacta\n");
    } else {
        user::sys_write(b"hw-test: FALHOU, vitima alterada\n");
    }
    user::sys_exit();
}

extern "C" fn read_peripheral(address: usize, _: usize) -> ! {
    user::sys_write(b"hw-test: le periferico\n");
    unsafe { (address as *const u32).read_volatile() };
    survived();
}

extern "C" fn write_kernel(address: usize, _: usize) -> ! {
    user::sys_write(b"hw-test: escreve no kernel\n");
    unsafe { (address as *mut u32).write_volatile(0) };
    survived();
}

extern "C" fn read_other_task(address: usize, _: usize) -> ! {
    user::sys_write(b"hw-test: le outra task\n");
    unsafe { (address as *const u32).read_volatile() };
    survived();
}

extern "C" fn write_other_task(address: usize, _: usize) -> ! {
    user::sys_write(b"hw-test: escreve em outra task\n");
    unsafe { (address as *mut u32).write_volatile(0) };
    survived();
}

extern "C" fn write_code(address: usize, _: usize) -> ! {
    user::sys_write(b"hw-test: escreve no codigo\n");
    unsafe { (address as *mut u32).write_volatile(0) };
    survived();
}

extern "C" fn execute_own_block(address: usize, _: usize) -> ! {
    user::sys_write(b"hw-test: executa o proprio bloco\n");
    let code: extern "C" fn() = unsafe { core::mem::transmute(address) };
    code();
    survived();
}

extern "C" fn allocator(_: usize, _: usize) -> ! {
    let children: [Test; 3] = [child_exits, child_reads_past_end, child_reads_before_start];
    let mut waits: i32 = 0;

    for (index, &size) in CHILD_SIZES.iter().enumerate() {
        let child = children[index % children.len()];
        while user::sys_spawn(child, size).is_none() {
            waits += 1;
            user::sys_yield();
        }
    }

    let boundary_tests: [Test; 3] = [
        child_reads_csr,
        child_passes_kernel_pointer,
        child_crosses_block_end,
    ];
    for child in boundary_tests {
        while user::sys_spawn(child, BLOCK_SIZE).is_none() {
            waits += 1;
            user::sys_yield();
        }
    }

    for _ in 0..FINAL_ROUNDS {
        user::sys_yield();
    }

    let mut buffer = [0u8; 12];
    user::sys_write(b"hw-test: alocacao ok, esperas por memoria = ");
    user::sys_write(interpreter::format_int(waits, &mut buffer));
    user::sys_write(b"\nhw-test: fim\n");
    user::sys_exit();
}

extern "C" fn child_exits(start: usize, size: usize) -> ! {
    exercise_block(start, size);
    user::sys_write(b"hw-test: filho terminou normalmente\n");
    user::sys_exit();
}

extern "C" fn child_reads_past_end(start: usize, size: usize) -> ! {
    exercise_block(start, size);
    user::sys_write(b"hw-test: filho le depois do fim do bloco\n");
    unsafe { ((start + size) as *const u32).read_volatile() };
    survived();
}

extern "C" fn child_reads_before_start(start: usize, size: usize) -> ! {
    exercise_block(start, size);
    user::sys_write(b"hw-test: filho le antes do inicio do bloco\n");
    unsafe { ((start - WORD) as *const u32).read_volatile() };
    survived();
}

extern "C" fn child_reads_csr(_: usize, _: usize) -> ! {
    user::sys_write(b"hw-test: le csr de modo M\n");
    unsafe { asm!("csrr {}, mstatus", out(reg) _) };
    survived();
}

extern "C" fn child_passes_kernel_pointer(_: usize, _: usize) -> ! {
    let result: usize;
    unsafe {
        asm!(
            "ecall",
            in("a7") syscall::WRITE,
            inlateout("a0") mem::rodata_end() => result,
            in("a1") WORD,
        );
    }

    if result == syscall::ERROR {
        user::sys_write(b"hw-test: kernel recusou ponteiro invalido\n");
    } else {
        user::sys_write(b"hw-test: FALHOU, kernel aceitou ponteiro invalido\n");
    }
    user::sys_exit();
}

extern "C" fn child_crosses_block_end(start: usize, size: usize) -> ! {
    user::sys_write(b"hw-test: filho le cruzando o fim do bloco\n");
    unsafe {
        asm!(
            "lw {value}, 0({address})",
            value = out(reg) _,
            address = in(reg) start + size - MISALIGNED_TAIL,
        );
    }
    survived();
}

fn exercise_block(start: usize, size: usize) {
    let words = size / 4 / WORD;
    let block = start as *mut u32;

    for index in 0..words {
        let address = start + index * WORD;
        unsafe { block.add(index).write_volatile(address as u32) };
    }

    user::sys_yield();

    for index in 0..words {
        let address = start + index * WORD;
        if unsafe { block.add(index).read_volatile() } != address as u32 {
            user::sys_write(b"hw-test: FALHOU, bloco alterado por outra task\n");
            user::sys_exit();
        }
    }

    let last_word = (start + size - WORD) as *const u32;
    unsafe { last_word.read_volatile() };
}

fn survived() -> ! {
    user::sys_write(b"hw-test: FALHOU, acesso permitido\n");
    user::sys_exit();
}

unsafe fn new_block() -> Range<usize> {
    unsafe { heap::alloc_region(BLOCK_SIZE) }.expect("hw-test: sem memoria")
}

unsafe fn spawn(test: Test, block: Range<usize>, argument: usize) {
    unsafe { task::spawn_process(test, block, [argument, 0]) }.expect("hw-test: sem vaga");
}

/// # Safety
/// Chamar só com as interrupções desligadas, depois de `heap::init`.
pub unsafe fn spawn_all() {
    unsafe {
        spawn(spinner, new_block(), 0);

        let victim_block = new_block();
        let victim_address = victim_block.start;
        spawn(victim, victim_block, victim_address);

        spawn(read_peripheral, new_block(), board::UART);
        spawn(write_kernel, new_block(), task::context_addr(0));
        spawn(read_other_task, new_block(), victim_address);
        spawn(write_other_task, new_block(), victim_address);
        spawn(write_code, new_block(), mem::text_start());

        let own_block = new_block();
        let own_address = own_block.start;
        spawn(execute_own_block, own_block, own_address);

        spawn(allocator, new_block(), 0);
    }
}
