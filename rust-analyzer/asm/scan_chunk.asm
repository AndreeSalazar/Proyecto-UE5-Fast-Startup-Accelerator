; UE5 Fast Startup Accelerator - Chunk Scanner
; Copyright 2026 Eddi Andreé Salazar Matos
; Licensed under Apache 2.0
;
; Fast chunk scanning for UAsset magic bytes and headers
; Used during parallel asset discovery

section .data
    align 16
    ; UAsset magic: C1 83 2A 9E (little-endian)
    UASSET_MAGIC: dd 0x9E2A83C1
    ; UMap magic
    UMAP_MAGIC: dd 0x9E2A83C1

section .text
    global scan_for_uasset
    global scan_for_magic
    global count_nulls_simd

; scan_for_uasset - Scan buffer for UAsset magic bytes
; Arguments (Windows x64):
;   rcx = buffer pointer
;   rdx = buffer size
; Returns: rax = offset of magic (-1 if not found)
scan_for_uasset:
    push rbx
    push rsi
    
    mov rsi, rcx            ; buffer
    mov rcx, rdx            ; size
    mov eax, [rel UASSET_MAGIC]
    
    ; Need at least 4 bytes
    cmp rcx, 4
    jb .not_found
    
    sub rcx, 3              ; Adjust for 4-byte comparison
    xor rbx, rbx            ; offset counter
    
.scan_loop:
    cmp rbx, rcx
    jae .not_found
    
    ; Compare 4 bytes
    cmp dword [rsi + rbx], eax
    je .found
    
    inc rbx
    jmp .scan_loop
    
.found:
    mov rax, rbx
    pop rsi
    pop rbx
    ret
    
.not_found:
    mov rax, -1
    pop rsi
    pop rbx
    ret

; scan_for_magic - Scan for arbitrary 4-byte magic
; Arguments (Windows x64):
;   rcx = buffer pointer
;   rdx = buffer size
;   r8d = magic value to find
; Returns: rax = offset (-1 if not found)
scan_for_magic:
    push rbx
    push rsi
    
    mov rsi, rcx
    mov rcx, rdx
    mov eax, r8d
    
    cmp rcx, 4
    jb .magic_not_found
    
    sub rcx, 3
    xor rbx, rbx
    
.magic_loop:
    cmp rbx, rcx
    jae .magic_not_found
    
    cmp dword [rsi + rbx], eax
    je .magic_found
    
    inc rbx
    jmp .magic_loop
    
.magic_found:
    mov rax, rbx
    pop rsi
    pop rbx
    ret
    
.magic_not_found:
    mov rax, -1
    pop rsi
    pop rbx
    ret

; count_nulls_simd - Count null bytes using SIMD
; Arguments (Windows x64):
;   rcx = buffer pointer
;   rdx = buffer size
; Returns: rax = count of null bytes
count_nulls_simd:
    push rbx
    push rsi
    
    mov rsi, rcx
    mov rcx, rdx
    xor rax, rax            ; count = 0
    
    ; Check if we have enough for SSE
    cmp rcx, 16
    jb .scalar_count
    
    ; Zero register for comparison
    pxor xmm0, xmm0
    xor rbx, rbx            ; accumulated count
    
    mov r8, rcx
    shr r8, 4               ; count / 16
    
.simd_loop:
    test r8, r8
    jz .simd_done
    
    ; Load 16 bytes
    movdqu xmm1, [rsi]
    
    ; Compare with zero
    pcmpeqb xmm1, xmm0
    
    ; Get mask of matches
    pmovmskb eax, xmm1
    
    ; Count bits (null bytes)
    popcnt eax, eax
    add rbx, rax
    
    add rsi, 16
    dec r8
    jmp .simd_loop
    
.simd_done:
    mov rax, rbx
    and rcx, 15             ; remaining bytes
    
.scalar_count:
    ; Count remaining bytes
    test rcx, rcx
    jz .count_done
    
.scalar_loop:
    cmp byte [rsi], 0
    jne .not_null
    inc rax
.not_null:
    inc rsi
    dec rcx
    jnz .scalar_loop
    
.count_done:
    pop rsi
    pop rbx
    ret
