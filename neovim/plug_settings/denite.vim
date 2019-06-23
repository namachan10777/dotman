autocmd FileType denite call s:denite_my_settings()
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
