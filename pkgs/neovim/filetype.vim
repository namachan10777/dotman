augroup fileTypeIndent
	autocmd!
	autocmd BufNewFile,BufRead *.elm setlocal tabstop=4 softtabstop=4 shiftwidth=4 expandtab
	autocmd BufNewFile,BufRead *.hs setlocal tabstop=4 softtabstop=4 shiftwidth=4 expandtab
	autocmd BufNewFile,BufRead *.yml setlocal tabstop=2 softtabstop=2 shiftwidth=2 expandtab
	autocmd BufNewFile,BufRead *.js  setlocal tabstop=2 softtabstop=2 shiftwidth=2 expandtab
	autocmd BufNewFile,BufRead *.gs  setlocal tabstop=2 softtabstop=2 shiftwidth=2 expandtab filetype=javascript
	autocmd BufNewFile,BufRead *.vue  setlocal tabstop=2 softtabstop=2 shiftwidth=2 expandtab
	autocmd BufNewFile,BufRead *.satyh setlocal tabstop=2 softtabstop=2 shiftwidth=2 expandtab
	autocmd BufNewFile,BufRead *.saty setlocal tabstop=2 softtabstop=2 shiftwidth=2 expandtab
	autocmd BufNewFile,BufRead *.elm setfiletype elm
	autocmd BufNewFile,BufRead *.clj setlocal tabstop=2 softtabstop=2 shiftwidth=2 expandtab
augroup END

augroup fileTypeSyntaxHighlighting
	autocmd!
	autocmd BufNewFile,BufRead *.vue syntax sync fromstart
augroup END
