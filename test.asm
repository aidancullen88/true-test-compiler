global _start

section .text

_start:
    mov rbp, rsp
    mov r8, 5
    push r8
    mov r9, 5
    mov rax, qword [rbp - 8]
    mul r9
    mov r10, rax
    push r10
    pop rdi
    mov rax, 60
    syscall