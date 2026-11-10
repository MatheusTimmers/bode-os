# bode_os

Microkernel didático em Rust `no_std` para RISC-V de 32 bits (`rv32imc`). Roda na máquina `virt` do QEMU, no simulador Verilator do Ibex e no Ibex sintetizado numa Nexys A7 (Artix-7), a 40 MHz.

O kernel roda em M-mode e as tasks em U-mode. A preempção vem do timer, o escalonador é round-robin, e cada task tem um bloco de memória vindo do alocador buddy, isolado por uma regra NAPOT da PMP. Programas em pseudocódigo são lidos da flash e interpretados em U-mode.

## Pré-requisitos

- [rustup](https://rustup.rs). O `rust-toolchain.toml` instala a toolchain fixada (rustc 1.99.0), o target `riscv32imc-unknown-none-elf`, o `clippy` e o `rustfmt`.
- `qemu-system-riscv32` (no Fedora, `sudo dnf install qemu-system-riscv`).
- Para o Ibex: o `ibex-demo-system` (branch `tangnano20k`) em `../ibex-demo-system`, com o simulador compilado, o FuseSoC no `.venv` dele e o `srecord`.
- Para a placa: `openFPGALoader`, `openocd` e acesso ao Vivado 2025.2 no Paxos (`ssh paxos-ext`), onde roda a síntese.

## Como rodar

```sh
scripts/bode.sh <onde> [modo] [programas.txt...]
```

| `<onde>` | O que é |
|---|---|
| `qemu` | Emulador, para a lógica do dia a dia |
| `sim` | Circuito real do Ibex no Verilator, para medir ciclos |
| `placa` | Nexys A7 |

| `[modo]` | O que roda |
|---|---|
| `normal` (padrão) | Kernel com programas de pseudocódigo |
| `hw-test` | Tasks que atacam a memória e exercitam alocação e PMP |
| `bench` | Medição dos alocadores com `mcycle` |
| `worst-bench` | Pior caso forçado dos alocadores, variando o número de blocos |
| `switch-bench` | Troca de contexto, `spawn`, `kill` e custo de zerar o bloco |
| `switch-bench-no-pmp` | A mesma troca de contexto, sem reprogramar a PMP |

Exemplos:

```sh
scripts/bode.sh qemu
scripts/bode.sh qemu normal programs/prog2.txt
scripts/bode.sh sim bench
scripts/bode.sh placa hw-test
scripts/bode.sh placa normal programs/prog2.txt programs/contagem.txt
```

Sem programas na linha de comando, `qemu` e `sim` usam `prog1` e `contagem`, e `placa` usa o que já está na flash. Para sair do QEMU: `Ctrl+A`, solta, `X`. No simulador e na placa: `Ctrl+C`. Os ciclos das medições só valem em `sim` e `placa`.

Também funciona `cargo run` e, para depurar, `cargo run -- -g`, que congela o QEMU e espera o GDB na porta 1234. Sem `BODE_FLASH`, o `scripts/run.sh` monta `prog1` e `contagem`. `BODE_FLASH=none` sobe sem programas.

## Na placa

`scripts/bode.sh placa` grava na FPGA um bitstream base, sintetizado uma vez, e carrega o kernel pelo JTAG com o OpenOCD, em cerca de 1 s. O bitstream base é refeito quando `rtl/`, `data/` ou o `.core` do `ibex-demo-system` mudam. Com `NEXYS_FULL=1`, o kernel vai dentro do bitstream, numa síntese de cerca de 6 min.

| Script | Função |
|---|---|
| `scripts/vmem.sh` | Converte o ELF em imagem da RAM |
| `scripts/nexys-build.sh imagem.vmem nome` | Sintetiza no Paxos e confere se o tempo fechou |
| `scripts/nexys-base.sh` | Devolve o bitstream base, sintetizando se preciso |
| `scripts/nexys-flash.sh bitstream [kernel.elf]` | Grava a FPGA, carrega o kernel e mostra a UART |
| `scripts/nexys-programs.sh a.txt b.txt` | Grava os programas na flash, em `0x400000`, e confere |

A gravação na FPGA vale até desligar a placa. Ao reconectar o USB, a chave POWER precisa ser ligada de novo. Os scripts `tang-*` são da Tang Nano 20K, usada antes, e ficaram sem uso.

## Testes

```sh
cargo test-host      # testes unitários, no host
scripts/smoke.sh     # QEMU, com e sem programas (depois de cargo build)
```

O CI (`.github/workflows/ci.yml`) roda formatação, clippy, build, testes e o smoke.

## Placas

O target é um só (`riscv32imc`, sem a extensão A). A feature `ibex` escolhe a placa.

| | QEMU `virt` | Ibex Demo System |
|---|---|---|
| RAM | `0x8000_0000`, 128 MB | `0x0010_0000`, 32 KiB |
| Primeira instrução (`boot`) | `0x8000_0000` | `0x0010_0080` |
| Tabela de vetores | `0x8000_0100` | `0x0010_0000` |
| UART | NS16550A em `0x1000_0000` | `0x8000_1000` (RX `+0x0`, TX `+0x4`, status `+0x8`) |
| `mtime` / `mtimecmp` | `0x0200_bff8` / `0x0200_4000` | `0x8000_2000` / `0x8000_2008` |
| Clock do timer | 10 MHz | 40 MHz |
| Flash | Pflash em `0x2200_0000` | SPI por GPIO (`gp_o[8..10]`, `gp_i[0]`), em `0x8000_0000`; na Nexys, o clock passa pelo `STARTUPE2` |

O `mtvec` usa o modo vetorado nas duas placas. A tabela tem 32 entradas de 4 bytes, todas com `j trap_handler`.

## Memória

O heap é definido pelo linker: vai do fim da pilha do kernel, alinhado a 2 KiB, até o fim da RAM, com limite de 16 KiB. No Ibex, o kernel normal fica com 12 KiB. Toda task recebe um bloco do buddy na criação e o devolve no término.

| Task | Bloco | Conteúdo |
|---|---|---|
| Idle | 1 KiB | Pilha |
| Programa | Programa + 1 KiB, em potência de 2 | Programa e variáveis no início, pilha no fim |

O buddy entrega blocos de tamanho potência de 2, alinhados ao próprio tamanho, que é o formato que a regra NAPOT exige.

| Regra PMP | Faixa | U-mode |
|---|---|---|
| 0 | Marca o início (`OFF`) | — |
| 1 | `.text` | R+X |
| 2 | `.rodata` | R |
| 3 | Bloco da task atual (`NAPOT`) | R+W |

O bit L fica desligado: o M-mode continua com acesso a tudo. A pilha do kernel (4 KiB) tem um canário, conferido na entrada e na saída de cada trap.

## Programas

O montador `tools/pseudoasm.py` traduz os `.txt` de `programs/` numa imagem binária, com assinatura `BODE`, gravada na flash em `0x400000`. No boot, `program/loader.rs` valida cada programa, pede um bloco ao buddy, copia e cria a task. `program/interpreter.rs` executa o pseudocódigo em U-mode.

## Syscalls

| Número | Nome | Efeito |
|---|---|---|
| 0 | `YIELD` | Entrega a CPU |
| 1 | `WRITE` | Envia até 256 bytes pela UART e devolve quantos enviou. `user::sys_write` repete até enviar tudo |
| 2 | `READ` | Devolve um byte recebido, ou `ERROR` |
| 3 | `EXIT` | Encerra a task, zera o bloco e o devolve |
| 4 | `SPAWN` | Só no `hw-test`: cria uma task com um bloco do buddy. Recusa região menor que 8 bytes e entrada fora do `.text` |

## Alocadores

`src/allocators/` tem cinco alocadores atrás do trait `Allocator`, testados no host.

| Alocador | Ideia |
|---|---|
| `bump` | Avança um ponteiro; não libera individualmente |
| `pool` | Classes de tamanho fixo (16 a 1024 bytes); pedidos maiores são recusados |
| `buddy` | Blocos potência de 2, par achado por `addr ^ size`, bitmap no próprio heap |
| `tlsf` | Listas segregadas em dois níveis e busca por bitmap |
| `hellfire` | Porte do first-fit do HellfireOS (`MEM_ALLOC 3`), usado como baseline. Só atende alinhamento de palavra |

`tools/bench_summary.py <csv>` resume a saída do modo `bench`. As falhas são separadas em recusas (pedido maior que a maior classe do pool) e falhas em que a soma do espaço livre bastaria, atribuídas à fragmentação. O bump fica fora dessa conta, porque não reaproveita memória.

## Limitações

- Programas são carregados só no boot.
- A pilha de uma task pode invadir as variáveis no próprio bloco sem ser detectada.
- O processo que aguarda tecla faz espera ativa.
- Um pânico dentro de uma task aparece como falha de acesso, sem a mensagem.
- O `worst-bench` força o pior caminho de cada alocador, mas não é uma análise de WCET: o limite vem do projeto do algoritmo.

## Mapa do código

| Caminho | Papel |
|---|---|
| `kernel.ld`, `boards/*/`, `build.rs` | Layout de memória e seleção de placa |
| `scripts/`, `tools/` | Runners, gravação e montador |
| `src/main.rs`, `src/boot.rs` | Entrada e `kmain` |
| `src/arch/` | `cpu`, `trap`, `pmp` |
| `src/kernel/` | `task`, `scheduler`, `syscall`, `timer` |
| `src/memory/` | `heap`, `mem` |
| `src/program/` | `loader`, `interpreter` |
| `src/drivers/` | UART, flash e chips de cada placa |
| `src/user.rs` | Lado do usuário das syscalls |
| `src/board.rs` | Endereços de cada placa |
| `src/hwtest.rs`, `src/bench.rs`, `src/worst_bench.rs`, `src/switch_bench.rs` | Testes e medições, atrás das features |
| `src/allocators/`, `src/collections/` | Biblioteca testada no host |
