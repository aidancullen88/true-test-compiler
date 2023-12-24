global _start

section .text

_start:
    mov rbp, rsp
    mov r8, 0
    push r8   ; save x to [rbp - 8]
start_while_1:
    mov rax, qword [rbp - 8]
    mov r9, 5
    cmp rax, r9
    jl while_1
    ; else block
    jmp end_while_1
while_1:
    mov rax, qword [rbp - 8]
    mov r10, 3
    cmp rax, r10
    je if_else_1
    ; else block
    mov r11, 1
    mov rax, qword [rbp - 8]
    add rax, r11
    mov r8, rax
    mov qword [rbp - 8], r8
    jmp end_if_else_1
if_else_1:
    jmp end_while_1
end_if_else_1:
    jmp start_while_1
end_while_1:
    pop rdi
    mov rax, 60
    syscall