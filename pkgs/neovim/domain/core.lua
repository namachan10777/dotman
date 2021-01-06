local minpac = require 'domain.minpac'

local load_core = function()
	minpac:load_repos({
		{ 'otyn0308/otynium' },
		{ 'lambdalisue/fern.vim' },
		{ 'lambdalisue/nerdfont.vim' },
		{ 'lambdalisue/fern-renderer-nerdfont.vim' },
		{ 'lambdalisue/fern-git-status.vim' },
		{ 'lambdalisue/fern-mapping-git.vim' },
		{ 'lambdalisue/fern-hijack.vim' }
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
end

load_core()
