global _start

section .text

_start:
    mov r8, 2
    mov r9, 4
    mov r10, 2
    xor rdx, rdx
    mov rax, r9
    div r10
    mov r9, rax
    add r8, r9
    mov rdi, r8
    mov rax, 60
    syscall