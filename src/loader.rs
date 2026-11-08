use crate::{flash, heap, interpreter, println, task};

const PROGRAMS_OFFSET: usize = 0x40_0000;
const MAGIC: u32 = u32::from_le_bytes(*b"BODE");
const MAX_PROGRAMS: u32 = 8;
const MAX_PROGRAM_SIZE: usize = 4 * 1024;
const PROCESS_STACK: usize = 1024;

/// # Safety
/// Chamar só com as interrupções desligadas, depois de `heap::init`.
pub unsafe fn load_programs() -> usize {
    if flash::read_u32(PROGRAMS_OFFSET) != MAGIC {
        println!("flash: nenhuma imagem de programas");
        return 0;
    }

    let count = flash::read_u32(PROGRAMS_OFFSET + 4);
    if count > MAX_PROGRAMS {
        println!(
            "flash: imagem diz ter {} programas, o máximo é {}",
            count, MAX_PROGRAMS
        );
        return 0;
    }

    let mut offset = PROGRAMS_OFFSET + 8;
    let mut loaded = 0;

    for id in 0..count as usize {
        let size = flash::read_u32(offset) as usize;
        offset += 4;

        if size > MAX_PROGRAM_SIZE {
            println!("programa {}: tamanho {} inválido, parando", id, size);
            break;
        }

        match unsafe { load_one(id, offset, size) } {
            Ok(start) => {
                println!(
                    "programa {}: {} bytes carregados em {:#010x}",
                    id, size, start
                );
                loaded += 1;
            }
            Err(reason) => println!("programa {}: {}", id, reason),
        }

        offset += size;
    }

    loaded
}

unsafe fn load_one(id: usize, offset: usize, size: usize) -> Result<usize, &'static str> {
    let block_size = (size + PROCESS_STACK).next_power_of_two();
    let region = unsafe { heap::alloc_region(block_size) }.ok_or("sem memória no heap")?;
    let start = region.start;

    let destination = unsafe { core::slice::from_raw_parts_mut(start as *mut u8, size) };
    flash::read(offset, destination);

    let code_len = unsafe { (start as *const u32).read() } as usize;
    let data_len = unsafe { (start as *const u32).add(1).read() } as usize;
    if interpreter::program_size(code_len, data_len) != Some(size) {
        unsafe { heap::free_region(region) };
        return Err("cabeçalho não bate com o tamanho");
    }

    if unsafe { task::spawn_process(interpreter::run, region.clone(), [start, id]) }.is_err() {
        unsafe { heap::free_region(region) };
        return Err("sem vaga na tabela de tasks");
    }

    Ok(start)
}
