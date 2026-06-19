; fc_player.s — 内置 tracker 的极简回放引擎(ca65)。
;
; 回放由内置 tracker 导出的"逐帧 APU 寄存器写入流"(见 tracker::export_ca65):
;   每帧:.byte N, reg0,val0, reg1,val1, ...   (reg = $40xx 低字节)
;   末尾:.byte $FF  → 循环回开头
;
; 用法(在主程序里):
;   reset 时:  jsr fc_player_init
;   每帧 NMI:  jsr fc_player_tick
; 数据由导出的乐曲 .s 提供(.export song_data)。

.export fc_player_init, fc_player_tick
.import song_data

.segment "ZEROPAGE"
fcp_ptr:   .res 2
fcp_count: .res 1

.segment "CODE"

fc_player_init:
    lda #<song_data
    sta fcp_ptr
    lda #>song_data
    sta fcp_ptr+1
    lda #$0F          ; 使能 脉冲1/脉冲2/三角/噪声
    sta $4015
    rts

; 推进一帧:应用本帧所有寄存器写入;遇 $FF 循环。
fc_player_tick:
    ldy #0
    lda (fcp_ptr),y
    cmp #$FF
    bne @go
    jsr fc_player_init     ; 循环回开头
    ldy #0
    lda (fcp_ptr),y
@go:
    sta fcp_count
    iny                    ; 跳过 count 字节
@loop:
    lda fcp_count
    beq @done
    lda (fcp_ptr),y        ; 寄存器低字节
    tax
    iny
    lda (fcp_ptr),y        ; 值
    iny
    sta $4000,x            ; 写 $4000+reg($4000..$4017)
    dec fcp_count
    jmp @loop
@done:
    tya                    ; 本帧消耗字节数 → 推进指针
    clc
    adc fcp_ptr
    sta fcp_ptr
    bcc @ret
    inc fcp_ptr+1
@ret:
    rts
