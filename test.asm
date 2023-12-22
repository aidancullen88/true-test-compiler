global _start

section .text

_start:
    mov rbp, rsp
    mov r8, 2
    mov r9, 3
    cmp r8, r9
    setb al
    movzx r8, al
    cmp al, 1    ; if_else_1
    jne else_1
    mov r10, 5
    push r10   ; save x to [rbp - 8]
    jmp end_if_else_1
else_1:
    mov r11, 1
    push r11   ; save y to [rbp - 16]
end_if_else_1:
    pop rdi
    mov rax, 60
    syscall