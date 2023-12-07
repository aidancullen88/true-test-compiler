global _start

section .text

_start:
    lea r8, [2 + 1]
    lea r8, [2 + r8]
    lea r8, [3 + r8]
    mov rax, 60
    mov rdi, r8
    syscall