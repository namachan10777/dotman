" package manager {{{
function s:install(packname, source)
	let s:repo_dir = '~/.local/share/nvim/site/pack/ever/start/'
	let s:dest = s:repo_dir . a:packname
	if !isdirectory(expand(s:dest))
		echo 'installing ' . a:source . 'into' . s:dest
		execute ('!git clone https://github.com/' . a:source) s:dest
	endif
endfunction
" }}}

call s:install('papercolor', 'NLKNguyen/papercolor-theme')
call s:install('nerdtree', 'preservim/nerdtree')
call s:install('nvim-lsp', 'neovim/nvim-lsp')


" undo persistence {{{
augroup SaveEditPos
	autocmd!
	let s:undoDir = expand("~/.nvimundo")
	call system('mkdir ' . s:undoDir)
	let &undodir = s:undoDir
	set undofile
	" 編集位置保存設定
	autocmd BufReadPost * if line("'\"") > 1 && line("'\"") <= line("$") | exe "normal! g`\"" | endif
augroup END
" }}}

" NERDTree {{{
augroup NERDTreeSetting
	autocmd!
	autocmd StdinReadPre * let s:std_in = 1
	if argc() == 0 || argc() == 1 && isdirectory(argv()[0]) && !exists("s:std_in")
		" ディレクトリ又は指定なしではツリーにフォーカス
		autocmd vimenter * NERDTreeToggle
	else
		" ファイル指定して開いた場合はバッファにフォーカス
		autocmd vimenter * NERDTreeToggle | wincmd p
	endif
	" NERDTree以外のバッファが閉じられたらNERDTreeも閉じる
	autocmd bufenter * if (winnr("$") == 1 && exists("b:NERDTree") && b:NERDTree.isTabTree()) | q | endif
augroup END

" }}}

" mkdir -p {{{
augroup MakeFileRecurse
	autocmd!
	autocmd BufWritePre * call s:auto_mkdir(expand('<afile>:p:h'), v:cmdbang)
		function! s:auto_mkdir(dir, force)
		if !isdirectory(a:dir) && (a:force ||
		\    input(printf('"%s" does not exist. Create? [y/N]', a:dir)) =~? '^y\%[es]$')
		  call mkdir(iconv(a:dir, &encoding, &termencoding), 'p')
		endif
	endfunction
augroup END
" }}}

" keybindings {{{
augroup KeyBinding
	autocmd!
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
augroup END
" }}}

augroup Lazy
	autocmd!
	autocmd VimEnter * call UserSettings()
augroup END


function UserSettings()
" lsp setting {{{
lua require'nvim_lsp'.clangd.setup{}
set omnifunc=v:lua.vim.lsp.omnifunc
" }}}

" undo persistence {{{
augroup SaveEditPos
	autocmd!
	let s:undoDir = expand("~/.nvimundo")
	call system('mkdir ' . s:undoDir)
	let &undodir = s:undoDir
	set undofile
	" 編集位置保存設定
	autocmd BufReadPost * if line("'\"") > 1 && line("'\"") <= line("$") | exe "normal! g`\"" | endif
augroup END
" }}}

" NERDTree {{{
augroup NERDTreeSetting
	autocmd!
	autocmd StdinReadPre * let s:std_in = 1
	if argc() == 0 || argc() == 1 && isdirectory(argv()[0]) && !exists("s:std_in")
		" ディレクトリ又は指定なしではツリーにフォーカス
		autocmd vimenter * NERDTreeToggle
	else
		" ファイル指定して開いた場合はバッファにフォーカス
		autocmd vimenter * NERDTreeToggle | wincmd p
	endif
	" NERDTree以外のバッファが閉じられたらNERDTreeも閉じる
	autocmd bufenter * if (winnr("$") == 1 && exists("b:NERDTree") && b:NERDTree.isTabTree()) | q | endif
augroup END

" }}}

" mkdir -p {{{
augroup MakeFileRecurse
	autocmd!
	autocmd BufWritePre * call s:auto_mkdir(expand('<afile>:p:h'), v:cmdbang)
		function! s:auto_mkdir(dir, force)
		if !isdirectory(a:dir) && (a:force ||
		\    input(printf('"%s" does not exist. Create? [y/N]', a:dir)) =~? '^y\%[es]$')
		  call mkdir(iconv(a:dir, &encoding, &termencoding), 'p')
		endif
	endfunction
augroup END
" }}}

" keybindings {{{
augroup KeyBinding
	autocmd!
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
augroup END
" }}}

set foldmethod=marker

set number
set relativenumber
set noswapfile
syntax on
filetype plugin on
set tabstop=4
set shiftwidth=4
set noexpandtab
set guicursor=
set hls
set list
set listchars=tab:»-,trail:-,eol:↲,extends:»,precedes:«,nbsp:%

set clipboard+=unnamed

" 以下カラースキーム設定
set background=dark
let g:artesanal_transp_bg = 0
colorscheme PaperColor

" 透過関連
highlight Normal ctermbg=NONE guibg=NONE
highlight NonText ctermbg=NONE guibg=NONE
highlight SpecialKey ctermbg=NONE guibg=NONE
highlight EndOfBuffer ctermbg=NONE guibg=NONE
highlight LineNr ctermbg=NONE guibg=NONE
endfunction
