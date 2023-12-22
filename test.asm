global _start

section .text

_start:
    mov rbp, rsp
    mov r8, 4
    mov r9, 3
    cmp r8, r9
    jg if_else_1    ; op is '<'
    ; if block
    mov r10, 5
    push r10   ; save x to [rbp - 8]
    jmp end_if_else_1
if_else_1:
    mov r11, 2
    push r11   ; save y to [rbp - 16]
end_if_else_1:
    pop rdi
    mov rax, 60
    syscall