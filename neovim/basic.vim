set number
syntax on
filetype plugin on
set tabstop=4
set shiftwidth=4
set noexpandtab

if has("autocmd")
	au BufReadPost * if line("'\"") > 1 && line("'\"") <= line("$") | exe "normal! g`\"" | endif
endif
