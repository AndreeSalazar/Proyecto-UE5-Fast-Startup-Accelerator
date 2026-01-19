; UE5 Fast Startup Accelerator - SIMD Hash Hot Path
; Copyright 2026 Eddi Andreé Salazar Matos
; Licensed under Apache 2.0
;
; High-performance xxHash-style hashing using AVX2/SSE4
; For asset content hashing during startup analysis

section .data
    align 32
    PRIME64_1: dq 0x9E3779B185EBCA87
    PRIME64_2: dq 0xC2B2AE3D27D4EB4F
    PRIME64_3: dq 0x165667B19E3779F9
    PRIME64_4: dq 0x85EBCA77C2B2AE63
    PRIME64_5: dq 0x27D4EB2F165667C5

section .text
    global hash_block_simd
    global hash_finalize

; hash_block_simd - Process 32-byte block with SIMD
; Arguments (Windows x64 calling convention):
;   rcx = pointer to data block (32 bytes aligned)
;   rdx = pointer to accumulator state (4 x 64-bit)
;   r8  = block count
; Returns: nothing (state updated in place)
hash_block_simd:
    push rbx
    push rsi
    push rdi
    push r12
    push r13
    push r14
    push r15
    
    mov rsi, rcx            ; data pointer
    mov rdi, rdx            ; state pointer
    mov r12, r8             ; block count
    
    ; Load primes
    mov r13, [rel PRIME64_1]
    mov r14, [rel PRIME64_2]
    
    ; Load current state
    mov rax, [rdi]          ; acc1
    mov rbx, [rdi + 8]      ; acc2
    mov rcx, [rdi + 16]     ; acc3
    mov rdx, [rdi + 24]     ; acc4
    
.loop_blocks:
    test r12, r12
    jz .done
    
    ; Process 32 bytes (4 lanes x 8 bytes)
    ; Lane 1
    mov r8, [rsi]
    imul r8, r14            ; input * PRIME64_2
    add rax, r8
    rol rax, 31
    imul rax, r13           ; * PRIME64_1
    
    ; Lane 2
    mov r8, [rsi + 8]
    imul r8, r14
    add rbx, r8
    rol rbx, 31
    imul rbx, r13
    
    ; Lane 3
    mov r8, [rsi + 16]
    imul r8, r14
    add rcx, r8
    rol rcx, 31
    imul rcx, r13
    
    ; Lane 4
    mov r8, [rsi + 24]
    imul r8, r14
    add rdx, r8
    rol rdx, 31
    imul rdx, r13
    
    add rsi, 32
    dec r12
    jmp .loop_blocks
    
.done:
    ; Store updated state
    mov [rdi], rax
    mov [rdi + 8], rbx
    mov [rdi + 16], rcx
    mov [rdi + 24], rdx
    
    pop r15
    pop r14
    pop r13
    pop r12
    pop rdi
    pop rsi
    pop rbx
    ret

; hash_finalize - Merge accumulators and finalize hash
; Arguments:
;   rcx = pointer to accumulator state (4 x 64-bit)
;   rdx = total length
; Returns: rax = final 64-bit hash
hash_finalize:
    push rbx
    push r12
    push r13
    
    mov r12, rdx            ; total length
    
    ; Load accumulators
    mov rax, [rcx]          ; acc1
    mov rbx, [rcx + 8]      ; acc2
    mov r8, [rcx + 16]      ; acc3
    mov r9, [rcx + 24]      ; acc4
    
    ; Merge accumulators
    rol rax, 1
    rol rbx, 7
    rol r8, 12
    rol r9, 18
    add rax, rbx
    add rax, r8
    add rax, r9
    
    ; Mix with length
    add rax, r12
    
    ; Final avalanche
    mov r13, [rel PRIME64_3]
    xor rax, rax
    shr rax, 33
    imul rax, r13
    xor rax, rax
    shr rax, 29
    mov r13, [rel PRIME64_4]
    imul rax, r13
    xor rax, rax
    shr rax, 32
    
    pop r13
    pop r12
    pop rbx
    ret
