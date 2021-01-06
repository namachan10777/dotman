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
		{ 'neoclide/coc.nvim' }
	})
	vim.api.nvim_command('colorscheme otynium')
	vim.api.nvim_command('set tabstop=4')
	vim.api.nvim_command('set shiftwidth=4')
	vim.api.nvim_command('set noexpandtab')
	vim.api.nvim_command('set number')
	vim.api.nvim_command('set relativenumber')

	-- remap
	vim.api.nvim_set_keymap('n', 'r', 'diwi', { noremap = true })
	vim.api.nvim_set_keymap('n', 'j', 'gj', { noremap = true })
	vim.api.nvim_set_keymap('n', 'k', 'gk', { noremap = true })
	vim.api.nvim_set_keymap('t', '<C-j>', '<C-\\><C-n>', { noremap = true })

	-- fern
	vim.api.nvim_set_var('fern#renderer', 'nerdfont')
	vim.api.nvim_set_keymap('n', '<space>f', ':Fern . -drawer<CR>', { noremap = true })
	vim.api.nvim_set_keymap('x', '<space>f', ':Fern . -drawer<CR>', { noremap = true })

	-- coc
	-- TODO select buffer by TAB
	vim.api.nvim_command('set hidden')
	vim.api.nvim_command('set nobackup')
	vim.api.nvim_command('set nowritebackup')
	vim.api.nvim_command('set cmdheight=2')
	vim.api.nvim_command('set updatetime=300')
	vim.api.nvim_command('set shortmess+=c')
	if vim.api.nvim_call_function('has', { 'patch-8.1.1564' }) then
		vim.api.nvim_command('set signcolumn=number')
	else
		vim.api.nvim_command('set signcolumn=yes')
	end
	vim.api.nvim_command('inoremap <silent><expr><TAB> pumvisible() ? \"\\<C-n>\" : \"\\<TAB>\"')
	-- vim.api.nvim_set_keymap('i', '<TAB>', '<C-n>', { noremap = true })

	vim.api.nvim_command('function! Check_back_space() abort\n'
	.. 'let col = col(\'.\') - 1\n'
	.. 'return !col || getline(\'.\')[col - 1]  =~# \'\\s\'\n'
	.. 'endfunction\n')
	vim.api.nvim_command('autocmd CursorHold * silent call CocActionAsync(\'highlight\')')
	vim.api.nvim_command('set statusline^=%{coc#status()}%{get(b:,\'coc_current_function\',\'\')}')

end

load_core()
