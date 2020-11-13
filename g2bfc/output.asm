section .text
global _root
_root:
mov edx,1
mov eax, tape
_start:
push eax
mov eax, 3
mov ebx, 0
mov ecx, eax
int 0x80
pop eax
cmp [eax], 0
je _fR3_0
_fR1_0:
jmp _fD2_0


cmp [eax], 0
jne _fR1_0
push eax
mov ecx,eax
mov ebx, 1
mov eax, 4
int 0x80
pop eax
mov ebx, [eax]
mov eax, 1
int 0x80

_fU4_17:
add [eax], 16
push eax
mov ecx,eax
mov ebx, 1
mov eax, 4
int 0x80
pop eax
mov ebx, [eax]
mov eax, 1
int 0x80

_fR2_17:
jmp _fU4_17


_fD2_0:
add [eax], 16
jmp _fR2_17



mov eax,1
int 0x80

section .data
tape: times 1536 db 0
