
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
getch
cmp [eax], 0
je _fR3_0
_fR1_0:
add [eax], 16
add [eax], 16
putch
jmp _end




cmp [eax], 0
jne _fR1_0
_fR3_0:

putch
jmp _end





_end:
mov eax,1
int 0x80

section .data
tape: times 1536 db 0
