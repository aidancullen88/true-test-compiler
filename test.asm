global _start

section .text

_start:
    mov rax, 3
    push rax
    mov rax, 4
    push rax
    pop rdx
    pop rax
    mul rdx
    push rax
    mov rax, 4
    push rax
    mov rax, 3
    push rax
    mov rax, 7
    push rax
    pop rdx
    pop rax
    add rax, rdx
    push rax
    pop rdx
    pop rax
    mul rdx
    push rax
    pop rdx
    pop rax
    add rax, rdx
    push rax
    mov rax, 60
    pop rdi
    syscall