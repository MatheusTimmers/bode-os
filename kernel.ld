ENTRY(boot)

SECTIONS {
    . = 0x80000000;
    __text_start = .;

    .text :{
        KEEP(*(.text.boot));
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

    .user_stacks (NOLOAD) : ALIGN(16) {
        KEEP(*(.user_stacks));
    }

    .data : ALIGN(4) {
        *(.data .data.* .sdata .sdata.*);
    }

    .bss : ALIGN(4) {
        __bss = .;
        *(.bss .bss.* .sbss .sbss.*);
        __bss_end = .;
    }

    . = ALIGN(16);
    . += 128 * 1024; /* 128KB */
    __stack_top = .;

    /DISCARD/ : {
        *(.eh_frame .eh_frame_hdr)
    }
}

ASSERT(__stack_top <= 0x88000000, "o kernel nao cabe nos 128 MB de RAM")
