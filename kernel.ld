INCLUDE board.ld

ENTRY(boot)

SECTIONS {
    . = RAM_START;
    __text_start = .;

    .text :{
        INCLUDE text_start.ld
        . = ALIGN(4);
        KEEP(*(.text.trap));
        *(.text .text.*);
    }

    . = ALIGN(4);
    __text_end = .;

    .rodata : ALIGN(4) {
        *(.rodata .rodata.* .srodata .srodata.*);
    }

    . = ALIGN(4);
    __rodata_end = .;

    .data : ALIGN(4) {
        *(.data .data.* .sdata .sdata.*);
    }

    .bss : ALIGN(4) {
        __bss = .;
        *(.bss .bss.* .sbss .sbss.*);
        __bss_end = .;
    }

    . = ALIGN(16);
    __kernel_stack_bottom = .;
    . += 4 * 1024;
    __stack_top = .;

    . = ALIGN(2K);
    __heap_start = .;
    __heap_end = MIN(RAM_START + RAM_SIZE, __heap_start + 16K);

    /DISCARD/ : {
        *(.eh_frame .eh_frame_hdr)
    }
}

ASSERT(__stack_top <= RAM_START + RAM_SIZE, "o kernel nao cabe na RAM")
ASSERT(__heap_end - __heap_start >= 8K, "heap menor que 8 KiB")
ASSERT(boot == RAM_START + BOOT_OFFSET, "boot no local errado")
