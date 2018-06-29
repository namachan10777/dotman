set number
set noswapfile
syntax on
filetype plugin on
set tabstop=4
set shiftwidth=4
set noexpandtab
set guicursor=
set nohlsearch

augroup BufRead,BufNewFile *.elm setfiletype elm
augroup fileTypeIndent
	autocmd!
	autocmd BufNewFile,BufRead *.elm setlocal tabstop=4 softtabstop=4 shiftwidth=4 expandtab
	autocmd BufNewFile,BufRead *.yml setlocal tabstop=2 softtabstop=2 shiftwidth=2 expandtab
augroup END

if has("autocmd")
	au BufReadPost * if line("'\"") > 1 && line("'\"") <= line("$") | exe "normal! g`\"" | endif
endif

set background=dark
let g:artesanal_transp_bg = 0

colorscheme artesanal

highlight Normal ctermbg=NONE guibg=NONE
highlight NonText ctermbg=NONE guibg=NONE
highlight SpecialKey ctermbg=NONE guibg=NONE
highlight EndOfBuffer ctermbg=NONE guibg=NONE
highlight LineNr ctermbg=NONE guibg=NONE
