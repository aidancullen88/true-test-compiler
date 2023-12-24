global _start

section .text

_start:
    mov rbp, rsp
    mov r8, 20
    push r8   ; save top to [rbp - 8]
    mov r9, 0
    push r9   ; save x to [rbp - 16]
    mov r10, 0
    push r10   ; save y to [rbp - 24]
start_while_1:
    mov rax, qword [rbp - 16]
    mov rcx, qword [rbp - 8]
    cmp rax, rcx
    jl while_1
    ; else block
    jmp end_while_1
while_1:
    mov r11, 1
    mov rax, qword [rbp - 16]
    add rax, r11
    mov r8, rax
    mov qword [rbp - 16], r8
    mov r9, 3
    mov rax, qword [rbp - 16]
    xor rdx, rdx
    div r9
    mov rax, rdx
    mov r10, rax
    mov r11, 0
    cmp r10, r11
    je if_1
    ; else block
    jmp end_if_1
if_1:
    mov r9, 1
    mov rax, qword [rbp - 24]
    add rax, r9
    mov r9, rax
    mov qword [rbp - 24], r9
end_if_1:
    jmp start_while_1
end_while_1:
    pop rdi
    mov rax, 60
    syscall