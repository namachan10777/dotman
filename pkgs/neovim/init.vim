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

let s:minpac_dir = '~/.config/nvim/pack/minpac/opt/minpac'
if !isdirectory(expand(s:minpac_dir))
	execute('!git clone https://github.com/k-takata/minpac.git ' . s:minpac_dir)
endif

if &compatible
	set nocompatible
endif
packadd minpac
call minpac#init()

call minpac#add('dag/vim-fish')
call minpac#add('NLKNguyen/papercolor-theme')
call minpac#add('neoclide/coc.nvim')
call minpac#add('qnighy/satysfi.vim')
call minpac#add('pest-parser/pest.vim')
call minpac#add('vim-airline/vim-airline')
call minpac#add('t9md/vim-quickhl')
call minpac#add('cespare/vim-toml')
call minpac#add('qnighy/lalrpop.vim')
call minpac#add('ElmCast/elm-vim')
call minpac#add('mxw/vim-jsx')
call minpac#add('ianks/vim-tsx')
call minpac#add('prettier/vim-prettier')
call minpac#add('otyn0308/otynium')
call minpac#add('lambdalisue/fern.vim')
call minpac#add('lambdalisue/nerdfont.vim')
call minpac#add('lambdalisue/fern-renderer-nerdfont.vim')
call minpac#add('lambdalisue/fern-git-status.vim')
call minpac#add('lambdalisue/fern-mapping-git.vim')
call minpac#add('lambdalisue/fern-hijack.vim')
call minpac#add('ctrlpvim/ctrlp.vim')

" language setting {{{
augroup LanguageSetting
	autocmd!
	autocmd FileType satysfi syntax sync fromstart
	autocmd FileType satysfi,yaml,tml,javascript,typescript.tsx setl shiftwidth=2 tabstop=2 expandtab softtabstop=2
	autocmd FileType java setl shiftwidth=4 tabstop=4 expandtab softtabstop=2
	autocmd FileType ocaml,cpp,c,kibanate setl shiftwidth=4 tabstop=4 noexpandtab softtabstop=2
augroup END
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

" fern {{{
let g:fern#renderer = "nerdfont"
nnoremap <space>f :Fern . -drawer<CR>
xnoremap <space>f :Fern . -drawer<CR>
" }}}

augroup Lazy
	autocmd!
	autocmd VimEnter * call LazySetting()
augroup END


autocmd FileType rust let b:coc_root_patterns = ['Cargo.toml']

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

function LazySetting()


" coc {{{
augroup CoC
	" TextEdit might fail if hidden is not set.
	set hidden

	" Some servers have issues with backup files, see #649.
	set nobackup
	set nowritebackup

	" Give more space for displaying messages.
	set cmdheight=3

	" Having longer updatetime (default is 4000 ms = 4 s) leads to noticeable
	" delays and poor user experience.
	set updatetime=300

	" Don't pass messages to |ins-completion-menu|.
	set shortmess+=c

	" Always show the signcolumn, otherwise it would shift the text each time
	" diagnostics appear/become resolved.
	if has("patch-8.1.1564")
	  " Recently vim can merge signcolumn and number column into one
	  set signcolumn=number
	else
	  set signcolumn=yes
	endif

	" Use tab for trigger completion with characters ahead and navigate.
	" NOTE: Use command ':verbose imap <tab>' to make sure tab is not mapped by
	" other plugin before putting this into your config.
	inoremap <silent><expr> <TAB>
		  \ pumvisible() ? "\<C-n>" :
		  \ <SID>check_back_space() ? "\<TAB>" :
		  \ coc#refresh()
	inoremap <expr><S-TAB> pumvisible() ? "\<C-p>" : "\<C-h>"

	function! s:check_back_space() abort
	  let col = col('.') - 1
	  return !col || getline('.')[col - 1]  =~# '\s'
	endfunction

	" Use <c-space> to trigger completion.
	inoremap <silent><expr> <c-space> coc#refresh()

	" Use <cr> to confirm completion, `<C-g>u` means break undo chain at current
	" position. Coc only does snippet and additional edit on confirm.
	" <cr> could be remapped by other vim plugin, try `:verbose imap <CR>`.
	if exists('*complete_info')
	  inoremap <expr> <cr> complete_info()["selected"] != "-1" ? "\<C-y>" : "\<C-g>u\<CR>"
	else
	  inoremap <expr> <cr> pumvisible() ? "\<C-y>" : "\<C-g>u\<CR>"
	endif

	" Use `[g` and `]g` to navigate diagnostics
	nmap <silent> [g <Plug>(coc-diagnostic-prev)
	nmap <silent> ]g <Plug>(coc-diagnostic-next)

	" GoTo code navigation.
	nmap <silent> gd <Plug>(coc-definition)
	nmap <silent> gy <Plug>(coc-type-definition)
	nmap <silent> gi <Plug>(coc-implementation)
	nmap <silent> gr <Plug>(coc-references)

	" Use K to show documentation in preview window.
	nnoremap <silent> K :call <SID>show_documentation()<CR>

	function! s:show_documentation()
	  if (index(['vim','help'], &filetype) >= 0)
		execute 'h '.expand('<cword>')
	  else
		call CocAction('doHover')
	  endif
	endfunction

	" Highlight the symbol and its references when holding the cursor.
	autocmd CursorHold * silent call CocActionAsync('highlight')

	" Symbol renaming.
	nmap <leader>rn <Plug>(coc-rename)

	" Formatting selected code.
	xmap <leader>f  <Plug>(coc-format-selected)
	nmap <leader>f  <Plug>(coc-format-selected)
augroup END

augroup mygroup
  autocmd!
  " Setup formatexpr specified filetype(s).
  autocmd FileType typescript,json,javascript setl formatexpr=CocAction('formatSelected')
  " Update signature help on jump placeholder.
  autocmd User CocJumpPlaceholder call CocActionAsync('showSignatureHelp')
augroup end

" Applying codeAction to the selected region.
" Example: `<leader>aap` for current paragraph
xmap <leader>a  <Plug>(coc-codeaction-selected)
nmap <leader>a  <Plug>(coc-codeaction-selected)

" Remap keys for applying codeAction to the current buffer.
nmap <leader>ac  <Plug>(coc-codeaction)
" Apply AutoFix to problem on the current line.
nmap <leader>qf  <Plug>(coc-fix-current)

" Map function and class text objects
" NOTE: Requires 'textDocument.documentSymbol' support from the language server.
xmap if <Plug>(coc-funcobj-i)
omap if <Plug>(coc-funcobj-i)
xmap af <Plug>(coc-funcobj-a)
omap af <Plug>(coc-funcobj-a)
xmap ic <Plug>(coc-classobj-i)
omap ic <Plug>(coc-classobj-i)
xmap ac <Plug>(coc-classobj-a)
omap ac <Plug>(coc-classobj-a)

" Use CTRL-S for selections ranges.
" Requires 'textDocument/selectionRange' support of LS, ex: coc-tsserver
nmap <silent> <C-s> <Plug>(coc-range-select)
xmap <silent> <C-s> <Plug>(coc-range-select)

" Add `:Format` command to format current buffer.
command! -nargs=0 Format :call CocAction('format')

" Add `:Fold` command to fold current buffer.
command! -nargs=? Fold :call     CocAction('fold', <f-args>)

" Add `:OR` command for organize imports of the current buffer.
command! -nargs=0 OR   :call     CocAction('runCommand', 'editor.action.organizeImport')

" Add (Neo)Vim's native statusline support.
" NOTE: Please see `:h coc-status` for integrations with external plugins that
" provide custom statusline: lightline.vim, vim-airline.
set statusline^=%{coc#status()}%{get(b:,'coc_current_function','')}

" Mappings using CoCList:
" Show all diagnostics.
nnoremap <silent> <space>a  :<C-u>CocList diagnostics<cr>
" Manage extensions.
nnoremap <silent> <space>e  :<C-u>CocList extensions<cr>
" Show commands.
nnoremap <silent> <space>c  :<C-u>CocList commands<cr>
" Find symbol of current document.
nnoremap <silent> <space>o  :<C-u>CocList outline<cr>
" Search workspace symbols.
nnoremap <silent> <space>s  :<C-u>CocList -I symbols<cr>
" Do default action for next item.
nnoremap <silent> <space>j  :<C-u>CocNext<CR>
" Do default action for previous item.
nnoremap <silent> <space>k  :<C-u>CocPrev<CR>
" Resume latest coc list.
nnoremap <silent> <space>p  :<C-u>CocListResume<CR>
" }}}

" quickhl {{{
nmap <Space>m <Plug>(quickhl-manual-this)
xmap <Space>m <Plug>(quickhl-manual-this)
nmap <Space>M <Plug>(quickhl-manual-reset)
xmap <Space>M <Plug>(quickhl-manual-reset)
" }}}

" command window {{{
autocmd CmdwinEnter : g/^qa\?!\?$/d
autocmd CmdwinEnter : g/^wq\?a\?!\?$/d
" }}}

set foldmethod=marker

set hidden
set number
set relativenumber
set noswapfile
syntax on
filetype plugin indent on
"set tabstop=4
"set shiftwidth=4
"set noexpandtab
set guicursor=
set hls
set list
set listchars=tab:»-,trail:-,eol:↲,extends:»,precedes:«,nbsp:%

set clipboard+=unnamed

" 以下カラースキーム設定
set background=dark
let g:artesanal_transp_bg = 0

" 透過関連
highlight Normal          ctermbg=NONE    guibg=NONE
highlight NonText         ctermbg=NONE    guibg=NONE
highlight SpecialKey      ctermbg=NONE    guibg=NONE
highlight EndOfBuffer     ctermbg=NONE    guibg=NONE
highlight LineNr          ctermbg=NONE    guibg=NONE
highlight CocUnderline    ctermbg=Red     cterm=underline
highlight CocInfoSign     ctermfg=Yellow  guifg=#fab005
highlight CocErrorSign    ctermfg=Red     guifg=#ff0000
highlight CocWarningSign  ctermfg=Brown   guifg=#ff922b
highlight CocInfoSign     ctermfg=Yellow  guifg=#fab005
highlight CocHintSign     ctermfg=Blue    guifg=#15aabf
highlight CocSelectedText ctermfg=Red     guifg=#fb4934
highlight CocCodeLens     ctermfg=Gray    guifg=#999999
endfunction
