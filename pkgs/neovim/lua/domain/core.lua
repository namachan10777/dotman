local minpac = require 'domain.minpac'

local load_core = function()
	minpac:load_repos({
		{ 'otyn0308/otynium' },
		{ 'lambdalisue/fern.vim' },
		{ 'lambdalisue/nerdfont.vim' },
		{ 'lambdalisue/fern-renderer-nerdfont.vim' },
		{ 'lambdalisue/fern-git-status.vim' },
		{ 'lambdalisue/fern-mapping-git.vim' },
		{ 'lambdalisue/fern-hijack.vim' },
		{ 'neoclide/coc.nvim' },
		-- Syntax highlights
		{ 'pest-parser/pest.vim' },
		{ 'ElmCast/elm-vim' },
		{ 'prettier/vim-prettier' },
		{ 'jalvesaq/Nvim-R' },
		{ 'qnighy/satysfi.vim' },
		{ 'cespare/vim-toml' },
		{ 'qnighy/lalrpop.vim' },
		{ 'namachan10777/tml.vim' },
		-- utilities
		{ 't9md/vim-quickhl' },
	})
	vim.api.nvim_command('colorscheme otynium')
	vim.bo.tabstop=4
	vim.bo.shiftwidth=4
	vim.bo.expandtab=false
	vim.wo.number = true
	vim.wo.relativenumber = true

	-- remap
	vim.api.nvim_set_keymap('n', 'r', 'diwi', { noremap = true })
	vim.api.nvim_set_keymap('n', 'j', 'gj', { noremap = true })
	vim.api.nvim_set_keymap('n', 'k', 'gk', { noremap = true })
	vim.api.nvim_set_keymap('t', '<C-j>', '<C-\\><C-n>', { noremap = true })

	-- fern
	vim.api.nvim_set_var('fern#renderer', 'nerdfont')
	vim.api.nvim_set_keymap('n', '<space>f', ':Fern . -drawer<CR>', { noremap = true })
	vim.api.nvim_set_keymap('x', '<space>f', ':Fern . -drawer<CR>', { noremap = true })

	-- save edit position
	vim.o.undofile = true
	vim.cmd('augroup SaveEditPos')
	vim.cmd('autocmd!')
	vim.cmd('autocmd BufReadPost * if line(\"\'\\\"\") > 1 && line(\"\'\\\"\") <= line(\"$\") | exe \"normal! g`\\\"\" | endif')
	vim.cmd('augroup END')

	-- coc
	-- TODO select buffer by TAB
	vim.o.hidden = true
	vim.o.backup = false
	vim.o.writebackup = false
	vim.o.cmdheight=2
	vim.o.updatetime=300
	vim.o.hls = true
	vim.wo.list = true
	vim.o.listchars = 'tab:»-,trail:-,eol:↲,extends:»,precedes:«,nbsp:%'
	print(vim.o.listchars)
	if vim.api.nvim_call_function('has', { 'patch-8.1.1564' }) then
		vim.o.signcolumn = 'number';
	else
		vim.o.signcolumn = 'yes';
	end
	vim.api.nvim_command('inoremap <silent><expr><TAB> pumvisible() ? \"\\<C-n>\" : \"\\<TAB>\"')
	-- vim.api.nvim_set_keymap('i', '<TAB>', '<C-n>', { noremap = true })

	vim.api.nvim_command('function! Check_back_space() abort\n'
	.. 'let col = col(\'.\') - 1\n'
	.. 'return !col || getline(\'.\')[col - 1]  =~# \'\\s\'\n'
	.. 'endfunction\n')
	if vim.api.nvim_call_function('exists', {'*complete_info'}) then
		vim.api.nvim_command('inoremap <expr> <cr> complete_info()[\"selected\"] != \"-1\" ? \"\\<C-y>\" : \"\\<C-g>u\\<CR>\"')
	else
		vim.api.nvim_command('inoremap <expr><CR> pumvisible() ? \"\\<C-y>\" : \"\\<C-g>u\\<CR>\"')
  	end
	vim.api.nvim_command('autocmd CursorHold * silent call CocActionAsync(\'highlight\')')
	vim.o.statusline = vim.o.statusline .. '%{coc#status()}%{get(b:,\'coc_current_function\',\'\')}'

	-- quickhl
	vim.api.nvim_set_keymap('n', '<Space>m', '<Plug>(quickhl-manual-this)' , { noremap = false})
	vim.api.nvim_set_keymap('x', '<Space>m', '<Plug>(quickhl-manual-this)' , { noremap = false})
	vim.api.nvim_set_keymap('n', '<Space>M', '<Plug>(quickhl-manual-reset)', { noremap = false})
	vim.api.nvim_set_keymap('x', '<Space>M', '<Plug>(quickhl-manual-reset)', { noremap = false})

end

load_core()
