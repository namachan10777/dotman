nnoremap [del_word] diwi
tnoremap [exit_term] <C-\><C-n>

" 単語を削除しノーマルモードに復帰
nmap r [del_word]

nnoremap j gj
nnoremap gj j
nnoremap k gk
nnoremap gk k

" terminalからの脱出
tmap <C-j> [exit_term]

