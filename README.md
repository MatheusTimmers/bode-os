# bode_os

Kernel didático para RISC-V 32 bits (`rv32imac`), escrito em Rust `no_std`, que
roda na máquina `virt` do QEMU.

O kernel roda em M-mode e as tasks em U-mode. A preempção vem do timer do
CLINT, o escalonador é round-robin e as tasks falam com o kernel por `ecall`
(syscalls `yield` e `write`).

## Pré-requisitos

- [rustup](https://rustup.rs). O `rust-toolchain.toml` instala sozinho a
  toolchain, o target `riscv32imac-unknown-none-elf`, o `clippy` e o `rustfmt`.
- QEMU com suporte a RISC-V (`qemu-system-riscv32`):
  - Debian/Ubuntu: `sudo apt install qemu-system-misc`
  - macOS: `brew install qemu`

## Como rodar

```sh
cargo run            # compila e sobe no QEMU
cargo run -- -g      # congela no reset e espera o GDB na porta 1234
```

Para sair do QEMU: `Ctrl+A`, solta, `X`.

Com `-g`, noutro terminal:

```sh
gdb -ex 'set architecture riscv:rv32' -ex 'target remote :1234' \
    target/riscv32imac-unknown-none-elf/debug/bode_os
```

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
tasks, fila, pilha do kernel e MMIO (UART, CLINT) ficam negados: um acesso
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
`align(4096)`). O bit L fica desligado: o M-mode continua com acesso a tudo.

Limitação atual:

- Tasks não podem usar variáveis globais (`static mut`): elas ficam em
  `.data`/`.bss`, fora do alcance do U-mode.

## Mapa do código

| Arquivo | Papel |
|---|---|
| `kernel.ld` | Layout de memória: onde ficam código, dados e pilha |
| `src/boot.rs` | Ponto de entrada: configura a pilha e salta para o `kmain` |
| `src/main.rs` | `kmain` e as tasks de exemplo |
| `src/trap.rs` | Entrada de trap em assembly e o handler em Rust |
| `src/cpu.rs` | Acesso aos CSRs e `restore_context` (volta para a task com `mret`) |
| `src/task.rs` | Tabela de tasks, contextos e pilhas |
| `src/scheduler.rs` | Fila de prontas e escolha da próxima task |
| `src/syscall.rs` | Lado do kernel das syscalls |
| `src/user.rs` | Lado do usuário das syscalls (`ecall`) |
| `src/clint.rs` | Timer da máquina (`mtime`/`mtimecmp`) |
| `src/pmp.rs` | Proteção de memória física (PMP) para o U-mode |
| `src/uart.rs` | Saída serial e as macros `print!`/`println!` |
| `src/board.rs` | Endereços da máquina `virt` do QEMU |
| `src/collections/` | Estruturas sem dependência de hardware (testadas no host) |
