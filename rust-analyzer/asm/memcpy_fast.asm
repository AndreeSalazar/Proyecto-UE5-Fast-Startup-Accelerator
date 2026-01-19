; UE5 Fast Startup Accelerator - Fast Memory Copy
; Copyright 2026 Eddi Andreé Salazar Matos
; Licensed under Apache 2.0
;
; Optimized memory copy using AVX2 for large asset buffers
; Used when loading cached asset metadata

section .text
    global memcpy_fast_avx2
    global memcpy_fast_sse

; memcpy_fast_avx2 - Copy memory using AVX2 (256-bit)
; Arguments (Windows x64):
;   rcx = destination pointer
;   rdx = source pointer
;   r8  = byte count
; Returns: rax = destination pointer
memcpy_fast_avx2:
    push rbx
    push rsi
    push rdi
    
    mov rdi, rcx            ; dest
    mov rsi, rdx            ; src
    mov rcx, r8             ; count
    mov rax, rdi            ; return value
    
    ; Check if we have enough for AVX2 (256 bytes minimum)
    cmp rcx, 256
    jb .small_copy
    
    ; Align destination to 32 bytes
    mov rbx, rdi
    and rbx, 31
    jz .aligned
    
    ; Copy unaligned prefix
    mov r8, 32
    sub r8, rbx
    sub rcx, r8
    
.prefix_loop:
    mov bl, [rsi]
    mov [rdi], bl
    inc rsi
    inc rdi
    dec r8
    jnz .prefix_loop
    
.aligned:
    ; Main AVX2 loop - 256 bytes per iteration
    mov r8, rcx
    shr r8, 8               ; count / 256
    
.avx2_loop:
    test r8, r8
    jz .remainder
    
    ; Load 256 bytes (8 x 32-byte registers)
    vmovdqu ymm0, [rsi]
    vmovdqu ymm1, [rsi + 32]
    vmovdqu ymm2, [rsi + 64]
    vmovdqu ymm3, [rsi + 96]
    vmovdqu ymm4, [rsi + 128]
    vmovdqu ymm5, [rsi + 160]
    vmovdqu ymm6, [rsi + 192]
    vmovdqu ymm7, [rsi + 224]
    
    ; Store 256 bytes
    vmovdqu [rdi], ymm0
    vmovdqu [rdi + 32], ymm1
    vmovdqu [rdi + 64], ymm2
    vmovdqu [rdi + 96], ymm3
    vmovdqu [rdi + 128], ymm4
    vmovdqu [rdi + 160], ymm5
    vmovdqu [rdi + 192], ymm6
    vmovdqu [rdi + 224], ymm7
    
    add rsi, 256
    add rdi, 256
    dec r8
    jmp .avx2_loop
    
.remainder:
    and rcx, 255            ; remaining bytes
    
.small_copy:
    ; Copy remaining bytes with rep movsb
    rep movsb
    
    vzeroupper              ; Clear upper YMM bits
    
    pop rdi
    pop rsi
    pop rbx
    ret

; memcpy_fast_sse - Copy memory using SSE2 (128-bit fallback)
; Arguments (Windows x64):
;   rcx = destination pointer
;   rdx = source pointer
;   r8  = byte count
; Returns: rax = destination pointer
memcpy_fast_sse:
    push rbx
    push rsi
    push rdi
    
    mov rdi, rcx
    mov rsi, rdx
    mov rcx, r8
    mov rax, rdi
    
    ; Check minimum size for SSE
    cmp rcx, 128
    jb .sse_small
    
    ; Align to 16 bytes
    mov rbx, rdi
    and rbx, 15
    jz .sse_aligned
    
    mov r8, 16
    sub r8, rbx
    sub rcx, r8
    
.sse_prefix:
    mov bl, [rsi]
    mov [rdi], bl
    inc rsi
    inc rdi
    dec r8
    jnz .sse_prefix
    
.sse_aligned:
    mov r8, rcx
    shr r8, 7               ; count / 128
    
.sse_loop:
    test r8, r8
    jz .sse_remainder
    
    ; Load 128 bytes
    movdqu xmm0, [rsi]
    movdqu xmm1, [rsi + 16]
    movdqu xmm2, [rsi + 32]
    movdqu xmm3, [rsi + 48]
    movdqu xmm4, [rsi + 64]
    movdqu xmm5, [rsi + 80]
    movdqu xmm6, [rsi + 96]
    movdqu xmm7, [rsi + 112]
    
    ; Store 128 bytes
    movdqu [rdi], xmm0
    movdqu [rdi + 16], xmm1
    movdqu [rdi + 32], xmm2
    movdqu [rdi + 48], xmm3
    movdqu [rdi + 64], xmm4
    movdqu [rdi + 80], xmm5
    movdqu [rdi + 96], xmm6
    movdqu [rdi + 112], xmm7
    
    add rsi, 128
    add rdi, 128
    dec r8
    jmp .sse_loop
    
.sse_remainder:
    and rcx, 127
    
.sse_small:
    rep movsb
    
    pop rdi
    pop rsi
    pop rbx
    ret
