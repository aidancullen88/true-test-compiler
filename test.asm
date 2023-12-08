global _start

section .text

_start:
    mov r11, 3
    mov r10, 2
    mov r9, 1
    add r9, r10
    add r9, r11
    mov r11, 5
    mov r10, 4
    add r10, r11
    add r10, r9
    mov r9, 6
    mov r11, 5
    mov r8, 4
    add r8, r11
    mov r11, 4
    add r11, r8
    mov r8, 3
    mov rax, r8
    mul r11
    mov r8, rax
    add r8, r9
    mov rax, r8
    mul r10
    mov r8, rax
    mov rax, 60
    mov rdi, r8
    syscall