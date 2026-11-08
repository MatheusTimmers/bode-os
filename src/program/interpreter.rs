use crate::user;

pub const HEADER_SIZE: usize = 8;
pub const INSTRUCTION_SIZE: usize = 8;
pub const VARIABLE_SIZE: usize = 4;

const IMMEDIATE: u32 = 0x100;

const ADD: u32 = 1;
const SUB: u32 = 2;
const MULT: u32 = 3;
const DIV: u32 = 4;
const LOAD: u32 = 5;
const STORE: u32 = 6;
const BRANY: u32 = 7;
const BRPOS: u32 = 8;
const BRZERO: u32 = 9;
const BRNEG: u32 = 10;
const SYSCALL: u32 = 11;

const SYSCALL_EXIT: i32 = 0;
const SYSCALL_PRINT: i32 = 1;
const SYSCALL_READ: i32 = 2;

pub fn program_size(code_len: usize, data_len: usize) -> Option<usize> {
    let code = code_len.checked_mul(INSTRUCTION_SIZE)?;
    let data = data_len.checked_mul(VARIABLE_SIZE)?;
    HEADER_SIZE.checked_add(code)?.checked_add(data)
}

struct Program {
    code: &'static [[u32; 2]],
    data: &'static mut [i32],
}

impl Program {
    unsafe fn from_memory(start: usize) -> Self {
        let header = start as *const u32;
        let code_len = unsafe { header.read() } as usize;
        let data_len = unsafe { header.add(1).read() } as usize;

        let code_start = start + HEADER_SIZE;
        let data_start = code_start + code_len * INSTRUCTION_SIZE;

        unsafe {
            Self {
                code: core::slice::from_raw_parts(code_start as *const [u32; 2], code_len),
                data: core::slice::from_raw_parts_mut(data_start as *mut i32, data_len),
            }
        }
    }

    fn variable(&self, index: i32) -> Option<i32> {
        self.data.get(index as usize).copied()
    }

    fn set_variable(&mut self, index: i32, value: i32) -> Option<()> {
        let slot = self.data.get_mut(index as usize)?;
        *slot = value;
        Some(())
    }

    fn is_label(&self, target: i32) -> bool {
        (target as usize) < self.code.len()
    }
}

pub extern "C" fn run(start: usize, id: usize) -> ! {
    let mut program = unsafe { Program::from_memory(start) };
    let mut acc: i32 = 0;
    let mut pc: usize = 0;

    while let Some(&[opcode, operand]) = program.code.get(pc) {
        let operand = operand as i32;
        let immediate = opcode & IMMEDIATE != 0;
        let value = if immediate {
            Some(operand)
        } else {
            program.variable(operand)
        };

        let mut next = pc + 1;

        match opcode & !IMMEDIATE {
            ADD | SUB | MULT | DIV | LOAD => {
                let Some(value) = value else {
                    fail(id, "variável inexistente");
                };
                acc = match opcode & !IMMEDIATE {
                    ADD => acc.wrapping_add(value),
                    SUB => acc.wrapping_sub(value),
                    MULT => acc.wrapping_mul(value),
                    DIV if value == 0 => fail(id, "divisão por zero"),
                    DIV => acc.wrapping_div(value),
                    _ => value,
                };
            }
            STORE => {
                if program.set_variable(operand, acc).is_none() {
                    fail(id, "variável inexistente");
                }
            }
            BRANY | BRPOS | BRZERO | BRNEG => {
                if !program.is_label(operand) {
                    fail(id, "salto para fora do código");
                }
                let jump = match opcode {
                    BRANY => true,
                    BRPOS => acc > 0,
                    BRZERO => acc == 0,
                    _ => acc < 0,
                };
                if jump {
                    next = operand as usize;
                }
            }
            SYSCALL => match operand {
                SYSCALL_EXIT => {
                    print_labeled(id, b"fim, acc = ", acc);
                    user::sys_exit();
                }
                SYSCALL_PRINT => print_labeled(id, b"", acc),
                SYSCALL_READ => acc = read_int(id),
                _ => fail(id, "syscall desconhecida"),
            },
            _ => fail(id, "instrução desconhecida"),
        }

        pc = next;
    }

    fail(id, "chegou ao fim do código sem SYSCALL 0");
}

fn fail(id: usize, reason: &str) -> ! {
    print_prefix(id);
    user::sys_write(reason.as_bytes());
    user::sys_write(b"\n");
    user::sys_exit();
}

fn print_prefix(id: usize) {
    let digit = [b'0' + (id % 10) as u8];
    user::sys_write(b"P");
    user::sys_write(&digit);
    user::sys_write(b": ");
}

fn print_labeled(id: usize, label: &[u8], value: i32) {
    let mut buffer = [0u8; 12];
    let text = format_int(value, &mut buffer);

    print_prefix(id);
    user::sys_write(label);
    user::sys_write(text);
    user::sys_write(b"\n");
}

pub fn format_int(value: i32, buffer: &mut [u8; 12]) -> &[u8] {
    let negative = value < 0;
    let mut remaining = value.unsigned_abs();
    let mut position = buffer.len();

    loop {
        position -= 1;
        buffer[position] = b'0' + (remaining % 10) as u8;
        remaining /= 10;
        if remaining == 0 {
            break;
        }
    }

    if negative {
        position -= 1;
        buffer[position] = b'-';
    }

    &buffer[position..]
}

fn read_int(id: usize) -> i32 {
    loop {
        print_prefix(id);
        user::sys_write(b"digite um inteiro: ");

        if let Some(value) = read_line_as_int() {
            return value;
        }

        print_prefix(id);
        user::sys_write(b"valor invalido\n");
    }
}

fn read_line_as_int() -> Option<i32> {
    let mut value: i32 = 0;
    let mut digits = 0;
    let mut negative = false;
    let mut valid = true;

    loop {
        let Some(byte) = user::sys_read() else {
            user::sys_yield();
            continue;
        };

        match byte {
            b'\r' | b'\n' => {
                user::sys_write(b"\n");
                break;
            }
            b'-' if digits == 0 && !negative => negative = true,
            b'0'..=b'9' => {
                let digit = (byte - b'0') as i32;
                value = value.wrapping_mul(10).wrapping_add(digit);
                digits += 1;
            }
            _ => valid = false,
        }
        user::sys_write(&[byte]);
    }

    if !valid || digits == 0 {
        return None;
    }
    if negative {
        return Some(-value);
    }
    Some(value)
}
