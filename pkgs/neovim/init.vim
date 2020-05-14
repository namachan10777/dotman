"======================= filetype =========================
augroup fileTypeIndent
	autocmd!
	autocmd BufNewFile,BufRead *.c setlocal tabstop=2 softtabstop=2 shiftwidth=2 expandtab
	autocmd BufNewFile,BufRead *.sv setlocal tabstop=2 softtabstop=2 shiftwidth=2 noexpandtab
	autocmd BufNewFile,BufRead *.cls setlocal tabstop=2 softtabstop=2 shiftwidth=2 expandtab
	autocmd BufNewFile,BufRead *.elm setlocal tabstop=4 softtabstop=4 shiftwidth=4 expandtab
	autocmd BufNewFile,BufRead *.hs setlocal tabstop=4 softtabstop=4 shiftwidth=4 expandtab
	autocmd BufNewFile,BufRead *.yml setlocal tabstop=2 softtabstop=2 shiftwidth=2 expandtab
	autocmd BufNewFile,BufRead *.js  setlocal tabstop=2 softtabstop=2 shiftwidth=2 expandtab
	autocmd BufNewFile,BufRead *.gs  setlocal tabstop=2 softtabstop=2 shiftwidth=2 expandtab filetype=javascript
	autocmd BufNewFile,BufRead *.vue  setlocal tabstop=2 softtabstop=2 shiftwidth=2 expandtab
	autocmd BufNewFile,BufRead *.saty* setlocal tabstop=2 softtabstop=2 shiftwidth=2 expandtab filetype=satysfi
	autocmd BufNewFile,BufRead *.saty setlocal tabstop=2 softtabstop=2 shiftwidth=2 expandtab
	autocmd BufNewFile,BufRead *.elm setfiletype elm
	autocmd BufNewFile,BufRead *.clj setlocal tabstop=2 softtabstop=2 shiftwidth=2 expandtab
	autocmd BufNewFile,BufRead *.scm setlocal tabstop=2 softtabstop=2 shiftwidth=2 noexpandtab
augroup END

augroup fileTypeSyntaxHighlighting
	autocmd!
	autocmd BufNewFile,BufRead *.vue syntax sync fromstart
	autocmd BufNewFile,BufRead *.saty* syntax sync fromstart
augroup END

"======================= dein =========================
" プラグインがインストールされるディレクトリ
let s:dein_dir = expand('~/.cache/dein')
" dein.vim 本体
let s:dein_repo_dir = s:dein_dir . '/repos/github.com/Shougo/dein.vim'

set runtimepath+=~/.config/nvim/plugins/kibanate/

" dein.vim がなければ github から落としてくる
if &runtimepath !~# '/dein.vim'
  if !isdirectory(s:dein_repo_dir)
    execute '!git clone https://github.com/Shougo/dein.vim' s:dein_repo_dir
  endif
  execute 'set runtimepath^=' . fnamemodify(s:dein_repo_dir, ':p')
endif

" 設定開始
if dein#load_state(s:dein_dir)
  call dein#begin(s:dein_dir)

  " プラグインリストを収めた TOML ファイル
  " 予め TOML ファイルを用意しておく
  let g:rc_dir    = expand("~/.config/nvim/")
  let s:toml      = g:rc_dir . '/dein.toml'

  " TOML を読み込み、キャッシュしておく
  call dein#load_toml(s:toml,      {'lazy': 0})

  " 設定終了
  call dein#end()
  call dein#save_state()
endif

" もし、未インストールものものがあったらインストール
if dein#check_install()
  call dein#install()
endif

"======================= mkdir -p =========================
autocmd!
  autocmd BufWritePre * call s:auto_mkdir(expand('<afile>:p:h'), v:cmdbang)
  function! s:auto_mkdir(dir, force)
    if !isdirectory(a:dir) && (a:force ||
    \    input(printf('"%s" does not exist. Create? [y/N]', a:dir)) =~? '^y\%[es]$')
      call mkdir(iconv(a:dir, &encoding, &termencoding), 'p')
    endif
endfunction

"======================= NERDTree =========================
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

"=================== undo persistence =====================
augroup SaveEditPos
	autocmd!
	let s:undoDir = expand("~/.nvimundo")
	call system('mkdir ' . s:undoDir)
	let &undodir = s:undoDir
	set undofile
	" 編集位置保存設定
	autocmd BufReadPost * if line("'\"") > 1 && line("'\"") <= line("$") | exe "normal! g`\"" | endif
augroup END

"=================== keybinds =====================
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

"=================== coc.vim =====================

" TextEdit might fail if hidden is not set.
set hidden

" Some servers have issues with backup files, see #649.
set nobackup
set nowritebackup

" Give more space for displaying messages.
set cmdheight=2

" Having longer updatetime (default is 4000 ms = 4 s) leads to noticeable
" delays and poor user experience.
set updatetime=300

" Don't pass messages to |ins-completion-menu|.
set shortmess+=c

" Always show the signcolumn, otherwise it would shift the text each time
" diagnostics appear/become resolved.
set signcolumn=yes

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
if has('patch8.1.1068')
  " Use `complete_info` if your (Neo)Vim version supports it.
  inoremap <expr> <cr> complete_info()["selected"] != "-1" ? "\<C-y>" : "\<C-g>u\<CR>"
else
  imap <expr> <cr> pumvisible() ? "\<C-y>" : "\<C-g>u\<CR>"
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

augroup mygroup
  autocmd!
  " Setup formatexpr specified filetype(s).
  autocmd FileType typescript,json setl formatexpr=CocAction('formatSelected')
  " Update signature help on jump placeholder.
  autocmd User CocJumpPlaceholder call CocActionAsync('showSignatureHelp')
augroup end

" Applying codeAction to the selected region.
" Example: `<leader>aap` for current paragraph
xmap <leader>a  <Plug>(coc-codeaction-selected)
nmap <leader>a  <Plug>(coc-codeaction-selected)

" Remap keys for applying codeAction to the current line.
nmap <leader>ac  <Plug>(coc-codeaction)
" Apply AutoFix to problem on the current line.
nmap <leader>qf  <Plug>(coc-fix-current)

" Introduce function text object
" NOTE: Requires 'textDocument.documentSymbol' support from the language server.
xmap if <Plug>(coc-funcobj-i)
xmap af <Plug>(coc-funcobj-a)
omap if <Plug>(coc-funcobj-i)
omap af <Plug>(coc-funcobj-a)

" Use <TAB> for selections ranges.
" NOTE: Requires 'textDocument/selectionRange' support from the language server.
" coc-tsserver, coc-python are the examples of servers that support it.
nmap <silent> <TAB> <Plug>(coc-range-select)
xmap <silent> <TAB> <Plug>(coc-range-select)

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

" ===================== denite ===========================
"autocmd FileType denite call s:denite_my_settings()
function! s:denite_my_settings() abort
	nnoremap <silent><buffer><expr> <CR>    denite#do_map('do_action')
	nnoremap <silent><buffer><expr> d       denite#do_map('do_action', 'delete')
	nnoremap <silent><buffer><expr> p       denite#do_map('do_action', 'preview')
	nnoremap <silent><buffer><expr> q       denite#do_map('quit')
	nnoremap <silent><buffer><expr> i       denite#do_map('open_filter_buffer')
	nnoremap <silent><buffer><expr> <Space> denite#do_map('toggle_select').'j'
endfunction

autocmd FileType denite-filter call s:denite_filter_my_settings()
function! s:denite_filter_my_settings() abort
	imap <silent><buffer> <C-o> <Plug>(denite_filter_quit)
endfunction

call denite#custom#var('file/rec', 'command', ['rg', '--files', '--glob', '!.git'])

call denite#custom#var('grep', 'command', ['rg'])
call denite#custom#var('grep', 'default_opts', ['-i', '--vimgrep', '--no-heading'])
call denite#custom#var('grep', 'recursive_opts', [])
call denite#custom#var('grep', 'pattern_opt', ['--regexp'])
call denite#custom#var('grep', 'separator', ['--'])
call denite#custom#var('grep', 'final_opts', [])

nnoremap [denite] <Nop>
nmap <C-d> [denite]

noremap [denite]r :<C-u>Denite register<CR>
noremap [denite]f :<C-u>Denite file/rec buffer<CR>
noremap [denite]y :<C-u>Denite neoyank<CR>

" ================== general setting ======================

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

