" NERDTreeの設定
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

" undo永続化
augroup SaveEditPos
	autocmd!
	let s:undoDir = expand("~/.nvimundo")
	call system('mkdir ' . s:undoDir)
	let &undodir = s:undoDir
	set undofile
	" 編集位置保存設定
	autocmd BufReadPost * if line("'\"") > 1 && line("'\"") <= line("$") | exe "normal! g`\"" | endif
	
augroup END
