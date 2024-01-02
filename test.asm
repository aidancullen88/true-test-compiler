global _start

section .text

_start:
    mov rbp, rsp
    lea rsp, [rsp - 16]
    mov r8, 3
    mov qword [rbp - 0], r8
    lea rax, [rbp - 0]
    mov qword [rbp - 16], rax
    mov rax, qword [rbp - 16]
    mov r9, qword [rax]
    mov r10, 1
    add r9, r10
    mov qword [rbp - 8], r9
    mov rdi, qword [rbp - 8]
    mov rax, 60
    syscall