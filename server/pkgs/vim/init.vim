set shiftwidth=4
set tabstop=4
set noexpandtab
set noswapfile
set nobackup
set nowritebackup
set number
set relativenumber " TODO: vimのバージョン見て入れる
set cmdheight=2
set hls
syntax on
nnoremap j gj
nnoremap k gk
nnoremap r diwi

augroup FileTypeIndent
	autocmd!
	autocmd FileType python setlocal shiftwidth=4 tabstop=4 expandtab
	autocmd FileType yaml,json setlocal shiftwidth=2 tabstop=2 expandtab
augroup END
