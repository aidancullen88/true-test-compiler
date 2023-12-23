global _start

section .text

_start:
    mov rbp, rsp
    mov r8, 0
    push r8   ; save x to [rbp - 8]
    mov r9, 1
    push r9   ; save y to [rbp - 16]
start_while_1:
    mov rax, qword [rbp - 8]
    mov r10, 5
    cmp rax, r10
    jl while_1
    ; else block
    jmp end_while_1
while_1:
    mov rax, qword [rbp - 8]
    mov r11, 4
    cmp rax, r11
    je if_1
    ; else block
    jmp end_if_1
if_1:
    mov r8, 1
    mov rax, qword [rbp - 16]
    add rax, r8
    mov r9, rax
    mov qword [rbp - 16], r9
end_if_1:
    mov rax, qword [rbp - 8]
    mov r8, 3
    cmp rax, r8
    je if_2
    ; else block
    jmp end_if_2
if_2:
    mov rax, 1
    push rax
    pop rcx
    mov rax, qword [rbp - 16]
    add rax, rcx
    push rax
    pop rax
    mov qword [rbp - 16], rax
end_if_2:
    mov rax, 1
    push rax
    pop rcx
    mov rax, qword [rbp - 8]
    add rax, rcx
    push rax
    pop rax
    mov qword [rbp - 8], rax
    jmp start_while_1
end_while_1:
    pop rdi
    mov rax, 60
    syscall