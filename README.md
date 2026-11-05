# bode_os

Kernel didático para RISC-V 32 bits (`rv32imc`), escrito em Rust `no_std`, que
roda na máquina `virt` do QEMU. O porte para o núcleo Ibex do
[Ibex Demo System](https://github.com/lowRISC/ibex-demo-system), visando o
simulador Verilator e uma Tang Nano 20K, compila, mas ainda não foi executado.

O kernel roda em M-mode e as tasks em U-mode. A preempção vem do timer da
máquina (`mtime`/`mtimecmp`), o escalonador é round-robin e as tasks falam com o
kernel por `ecall` (syscalls `yield` e `write`).

## Pré-requisitos

- [rustup](https://rustup.rs). O `rust-toolchain.toml` instala sozinho a
  toolchain fixada (rustc 1.99.0), o target `riscv32imc-unknown-none-elf`, o
  `clippy` e o `rustfmt`.
- QEMU com suporte a RISC-V (`qemu-system-riscv32`):
  - Debian/Ubuntu: `sudo apt install qemu-system-misc`
  - Fedora: `sudo dnf install qemu-system-riscv`
  - macOS: `brew install qemu`
- Só para o Ibex: o simulador Verilator do `ibex-demo-system` (branch
  `tangnano20k`) compilado em `../ibex-demo-system`, e o `srecord` para gerar o
  `.vmem` da FPGA.

## Como rodar

```sh
cargo run            # compila e sobe no QEMU
cargo run -- -g      # congela no reset e espera o GDB na porta 1234
```

Para sair do QEMU: `Ctrl+A`, solta, `X`.

Com `-g`, noutro terminal:

```sh
gdb -ex 'set architecture riscv:rv32' -ex 'target remote :1234' \
    target/riscv32imc-unknown-none-elf/debug/bode_os
```

No Ibex:

```sh
cargo build-ibex     # release, --features ibex
cargo run-ibex       # roda no simulador Verilator, com a UART no terminal
scripts/vmem.sh      # gera o .vmem que entra no SRAMInitFile do bitstream
```

O `run-ibex.sh` procura o simulador em
`../ibex-demo-system/build/lowrisc_ibex_demo_system_0/sim-verilator/Vtop_verilator`;
outro caminho vai na variável `IBEX_SIM`. Para compilar o simulador, na raiz
do `ibex-demo-system`:

```sh
fusesoc --cores-root=. run --target=sim --tool=verilator --setup --build lowrisc:ibex:demo_system
```

## Placas

O target é um só (`riscv32imc`, sem a extensão A, porque o Ibex não a tem); a
feature `ibex` escolhe a placa. O `build.rs` passa o `kernel.ld` comum e põe
`boards/virt/` ou `boards/ibex/` no caminho de busca do linker, de onde o
`INCLUDE board.ld` tira `RAM_START`, `RAM_SIZE` e `BOOT_OFFSET`, e o
`INCLUDE text_start.ld` tira a ordem do início do `.text`.

| | QEMU `virt` | Ibex Demo System |
|---|---|---|
| RAM | `0x8000_0000`, 128 MB | `0x0010_0000`, 32 KiB |
| primeira instrução (`boot`) | `0x8000_0000` | `0x0010_0080` |
| tabela de vetores | `0x8000_0100` | `0x0010_0000` |
| UART | NS16550A em `0x1000_0000` | `0x8000_1000` (TX `+0x4`, status `+0x8`) |
| `mtime` / `mtimecmp` | `0x0200_bff8` / `0x0200_4000` | `0x8000_2000` / `0x8000_2008` |
| clock do timer | 10 MHz | 27 MHz (1 por ciclo) |

O QEMU com `-bios none` ignora o entry point do ELF e salta para o início da
RAM; o Ibex começa em `boot_addr + 0x80`. No Ibex, a tabela de vetores fica no
início da RAM, como nos programas do Ibex Demo System, e ocupa exatamente os
`0x80` bytes antes do `boot`.

O `mtvec` usa o modo vetorado nas duas placas, porque o Ibex só aceita esse
modo e despreza os 8 bits baixos da base. A tabela (`trap::vector_table`) tem
32 entradas de 4 bytes, todas com `j trap_handler`, e sua base é múltipla de
`0x100`. O `.option norvc` impede que a extensão C
encurte alguma entrada para 2 bytes.

## Testes

```sh
cargo test-host      # testes unitários da lib, rodando no host
scripts/smoke.sh     # sobe o kernel por 3 s e confere a saída (rodar após cargo build)
```

O alias `test-host` (em `.cargo/config.toml`) usa o target
`x86_64-unknown-linux-gnu`. Em outro sistema, rode
`cargo test --lib --target <teu host>`; o host aparece em `rustc -vV`.

O CI (`.github/workflows/ci.yml`) roda formatação, clippy, build, testes e o
smoke test em todo PR.

## Proteção de memória

O PMP deixa as tasks (U-mode) executarem o `.text`, lerem o `.rodata` e
lerem/escreverem a própria pilha (em `.user_stacks`). Contextos, tabela de
tasks, fila, pilha do kernel e MMIO (UART, timer) ficam negados: um acesso
encerra a task.

| Regra PMP | Faixa | U-mode |
|---|---|---|
| 0 | só marca o início (`OFF`) | - |
| 1 | `__text_start..__text_end` | R+X |
| 2 | `__text_end..__rodata_end` | R |
| 3 | pilha da task atual (`NAPOT`, trocada a cada switch) | R+W |

Cada regra TOR vale de `pmpaddr[i-1]` até `pmpaddr[i]`, por isso `.text` e
`.rodata` ficam juntas e nessa ordem no `kernel.ld`. A regra NAPOT exige que a
pilha seja potência de 2 e alinhada ao próprio tamanho (`Stack` tem
`align(1024)`). O bit L fica desligado: o M-mode continua com acesso a tudo.

A pilha do kernel (4 KB) fica logo acima do `.bss` e tem um canário na última
palavra, conferido na entrada de cada trap.

Limitações atuais:

- Tasks não podem usar variáveis globais (`static mut`): elas ficam em
  `.data`/`.bss`, fora do alcance do U-mode.
- A syscall `write` espera a UART byte a byte com as interrupções desligadas,
  sem limite de tamanho.
- Um pânico dentro de uma task aparece como falha de acesso na UART, não com a
  mensagem do pânico.

## Alocadores

`src/allocators/` tem quatro alocadores atrás do trait `Allocator`
(`alloc`/`dealloc` com `Layout`), testados no host pelas macros
`contract_tests!` e `reuse_tests!`. O kernel ainda não usa nenhum deles.

| Alocador | Ideia |
|---|---|
| `bump` | avança um ponteiro; só libera tudo de uma vez |
| `pool` | blocos de tamanho fixo numa lista livre |
| `buddy` | blocos potência de 2 (0x10 a 0x1_0000), par achado por `addr ^ size`, bitmap no próprio heap |
| `tlsf` | listas segregadas em dois níveis (SLI 4), busca O(1) por bitmap e junção com os vizinhos físicos |

## Mapa do código

| Arquivo | Papel |
|---|---|
| `kernel.ld` | Layout de memória: onde ficam código, dados e pilha |
| `boards/*/` | Por placa: início e tamanho da RAM, posição do `boot` e ordem do início do `.text` |
| `build.rs` | Escolhe o `board.ld` pela feature e passa o linker script |
| `run.sh` / `run-ibex.sh` | Runners do `cargo run`: QEMU e simulador do Ibex |
| `scripts/vmem.sh` | Converte o ELF do Ibex no `.vmem` da FPGA |
| `src/boot.rs` | Ponto de entrada: configura a pilha e salta para o `kmain` |
| `src/main.rs` | `kmain` e as tasks de exemplo |
| `src/trap.rs` | Tabela de vetores, entrada de trap em assembly e o handler em Rust |
| `src/cpu.rs` | Acesso aos CSRs e `restore_context` (volta para a task com `mret`) |
| `src/task.rs` | Tabela de tasks, contextos e pilhas |
| `src/scheduler.rs` | Fila de prontas e escolha da próxima task |
| `src/syscall.rs` | Lado do kernel das syscalls |
| `src/user.rs` | Lado do usuário das syscalls (`ecall`) |
| `src/timer.rs` | Timer da máquina (`mtime`/`mtimecmp`) |
| `src/pmp.rs` | Proteção de memória física (PMP) para o U-mode |
| `src/uart.rs` | Saída serial e as macros `print!`/`println!` |
| `src/board.rs` | Endereços e drivers de cada placa (`virt` e `ibex`) |
| `src/drivers/` | Drivers de UART por chip (NS16550A e a UART do ibex-demo-system) |
| `src/mem.rs` | Zera o `.bss`, canário da pilha do kernel e faixas de memória do usuário |
| `src/collections/` | Estruturas sem dependência de hardware (testadas no host) |
| `src/allocators/` | Bump, pool, buddy e TLSF (testados no host) |
