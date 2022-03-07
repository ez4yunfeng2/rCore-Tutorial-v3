    .section .text.entry
    .globl _start
_start:
    mv tp, a0
    la sp, boot_stack
    li t0, 4096 * 16
    addi t1, a0, 1
1:  add sp, sp, t0
    addi t1, t1, -1
    bnez t1, 1b
    call rust_main

    .section .bss.stack
    .globl boot_stack
boot_stack:
    .space 4096 * 32
    .globl boot_stack_top
boot_stack_top:
