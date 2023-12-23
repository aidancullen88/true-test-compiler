global _start

section .text

_start:
    mov rbp, rsp
    mov r8, 0
    push r8   ; save x to [rbp - 8]
    mov r9, 1
    mov r10, 1
    cmp r9, r10
    je if_else_1    ; op is '!='
    ; if block
    mov r11, 10
    push r11   ; save x to [rbp - 16]
    jmp end_if_else_1
if_else_1:
    mov r8, 20
    push r8   ; save x to [rbp - 24]
end_if_else_1:
    pop rdi
    mov rax, 60
    syscall