global _start

section .text

_start:
    mov r8, 3
    mov r9, 2
    mov r10, 1
    add r10, r9
    add r9, r8
    mov r9, 5
    mov r10, 4
    add r10, r9
    add r9, r8
    mov r9, 5
    mov r10, 5
    mov r11, 4
    add r11, r10
    mov r11, 4
    add r11, r10
    mov r11, 3
    mov rax, r11
    mul r10
    mov r11, rax
    add r10, r9
    mov rax, r9
    mul r8
    mov r9, rax
    mov rax, 60
    mov rdi, r8
    syscall