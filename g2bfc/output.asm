
%macro putch 0
push eax
mov ecx,eax
mov ebx, 1
mov eax, 4
int 0x80
pop eax
%endmacro

%macro getch 0
push eax
mov eax, 3
mov ebx, 0
mov ecx, eax
int 0x80
pop eax
%endmacro

section .text
global _root

_root:
mov edx,1
mov eax, tape
_start:
add [eax], 1
add [eax], 1
jmp _fR1_0





_fL12_0:
add [eax], 5
jmp _fR6_0


_fR1_0:
add [eax], 4
_fR6_0:
add [eax], 5
jmp _fL12_0


jmp _end





_end:
mov eax,1
int 0x80

section .data
tape: times 1536 db 0
