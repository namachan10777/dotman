require('plugins')
local fs = require('fs')
local completion = require('cfg.completion')

-- helpers {{{
local kmap = vim.api.nvim_set_keymap
local setvar = vim.api.nvim_set_var
local execute = vim.api.nvim_command
local fn = vim.fn
local install_path = fn.stdpath('data')..'/site/pack/packer/opt/'
local function augroup(name, inner)
	execute('augroup SaveEditPos')
	execute('autocmd!')
	for i = 1, #inner do
		vim.cmd(inner[i])
	end
	vim.cmd('augroup END')
end
local function set_indent(configs)
	execute('augroup fileTypeIndent')
	for i = 1, #configs do
		local config = configs[i]
		if #config.filetypes == 0 then
			print('warning: empty indent pattern')
		end
		local w = tostring(config.w)
		local common_prefix = 'autocmd FileType '..table.concat(config.filetypes, ',')..' setlocal'
		local tabstop = 'tabstop='..w
		local shiftwidth='shiftwidth='..w
		if config.expand then
			execute(table.concat({common_prefix, tabstop, shiftwidth, 'expandtab'}, ' '))
		else
			execute(table.concat({common_prefix, tabstop, shiftwidth, 'noexpandtab'}, ' '))
		end
	end
	execute('augroup END')
end
-- }}}

-- ad-hoc
require('packer').startup(function()

	local comp_packages = completion.packages
	for i = 1, #comp_packages do
		use(comp_packages[i])
	end

	-- package mananger
	use {'wbthomason/packer.nvim', opt = true}

	-- filer
	use 'lambdalisue/fern.vim'
	use 'lambdalisue/nerdfont.vim'
	use 'lambdalisue/fern-renderer-nerdfont.vim'
	use 'lambdalisue/fern-git-status.vim'
	use 'lambdalisue/fern-mapping-git.vim'
	use 'lambdalisue/fern-hijack.vim'

	-- status line
	use 'vim-airline/vim-airline'
	use 'namachan10777/nvim-highlite-otynium'

	-- language specific support
	-- use { 'JuliaEditorSupport/julia-vim', ft='julia' } bug?
	use { 'nvim-lua/plenary.nvim', ft='lua' }
	use { 'tjdevries/manillua.nvim', ft='lua' }
	use { 'euclidianAce/BetterLua.vim', ft='lua' }
	use { 'pest-parser/pest.vim', ft='pest' }
	use { 'ElmCast/elm-vim', ft='elm'}
	use { 'prettier/vim-prettier', ft={'typescript', 'typescriptreact', 'javascript'}}
	use { 'jalvesaq/Nvim-R', ft='R' }
	use { 'qnighy/satysfi.vim', ft='satysfi' }
	use { 'cespare/vim-toml', ft='toml' }
	use { 'qnighy/lalrpop.vim', ft='lalrpop' }
	use { 'namachan10777/tml.vim', ft='tml' }
	use { 'ron-rs/ron.vim', ft='ron' }
	-- utilities
	use 't9md/vim-quickhl'
	use 'nvim-treesitter/nvim-treesitter'
	use 'nvim-treesitter/completion-treesitter'
end)

-- remap
kmap('n', 'r', 'diwi', { noremap = true })
kmap('n', 'j', 'gj', { noremap = true })
kmap('n', 'k', 'gk', { noremap = true })
kmap('t', '<C-j>', '<C-\\><C-n>', { noremap = true })

-- save edit pos
augroup('SaveEditPos', {
	'autocmd BufReadPost * if line(\"\'\\\"\") > 1 && line(\"\'\\\"\") <= line(\"$\") | exe \"normal! g`\\\"\" | endif'
})

-- config indent
set_indent({
	{ filetypes= {'python', 'haskell'}, w=4, expand=true },
	{ filetypes= {'javascript', 'typescript', 'typescriptreact', 'json'}, w=2, expand=true },
	{ filetypes= {'yaml'}, w=2, expand=true },
	{ filetypes= {'plaintex', 'satysfi', 'tml'}, w=2, expand=true },
})

completion.configure()

-- quickhl
vim.api.nvim_set_keymap('n', '<Space>m', '<Plug>(quickhl-manual-this)' , { noremap = false})
vim.api.nvim_set_keymap('x', '<Space>m', '<Plug>(quickhl-manual-this)' , { noremap = false})
vim.api.nvim_set_keymap('n', '<Space>M', '<Plug>(quickhl-manual-reset)', { noremap = false})
vim.api.nvim_set_keymap('x', '<Space>M', '<Plug>(quickhl-manual-reset)', { noremap = false})

-- fern
setvar('fern#renderer', 'nerdfont')
kmap('n', '<space>f', ':Fern . -drawer<CR>', { noremap = true })
kmap('x', '<space>f', ':Fern . -drawer<CR>', { noremap = true })

-- save edit position
vim.bo.undofile = true
if not fs.exists(vim.o.undodir) then
	fs.mkdir(vim.o.undodir)
end
-- foldmethod
vim.wo.foldmethod = 'marker'
-- undo settings
vim.o.undolevels  = 1000
vim.o.undoreload  = 10000
-- backup and swap
vim.o.swapfile    = false
vim.o.backup      = false
vim.o.writebackup = false
-- indent settings
vim.bo.tabstop    = 4
vim.bo.shiftwidth = 4
vim.bo.expandtab  = false
-- looks settings
vim.o.termguicolors   = true
execute('colorscheme otynium')
vim.wo.number         = true
vim.wo.relativenumber = true
vim.o.cmdheight       = 2    -- the height of cmd space
vim.o.hls             = true -- highlighting matched charcters while editing query
vim.wo.list           = true -- enable visualize unvisible charcter
vim.o.listchars       = 'tab:»-,trail:-,eol:↲,extends:»,precedes:«,nbsp:%'
-- swtich other buffer from unsaved buffer without warnings.
vim.o.hidden = true
-- update file's timestamp automatically per 300 ms
vim.o.updatetime=300

require('nvim-treesitter.configs').setup {
	ensure_installed = 'maintained',
	highlight = {
		enable = true,
		disable = {},
	},
	indent = {
		enable = true,
	}
}
