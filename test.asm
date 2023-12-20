global _start

section .text

_start:
    mov rbp, rsp
    mov r8, 2
    push r8   ; save x to [rbp - 8]
    mov r9, 2
    mov rax, qword [rbp - 8]
    cmp rax, r9
    setne al
    movzx rax, al
    mov r10, rax
    cmp al, 1
    jne cont
    mov r11, 5
    push r11   ; save y to [rbp - 16]
cont:
    pop rdi
    mov rax, 60
    syscall