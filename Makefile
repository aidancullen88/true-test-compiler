test: test.o
	ld -m elf_x86_64 -o test test.o

test.o: test.asm
	nasm -g -F dwarf -f elf64 -o test.o test.asm


