require('plugins')
local fs = require('fs')

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
	-- package mananger
	use {'wbthomason/packer.nvim', opt = true}

	-- colorscheme
	use 'otyn0308/otynium'

	-- filer
	use 'lambdalisue/fern.vim'
	use 'lambdalisue/nerdfont.vim'
	use 'lambdalisue/fern-renderer-nerdfont.vim'
	use 'lambdalisue/fern-git-status.vim'
	use 'lambdalisue/fern-mapping-git.vim'
	use 'lambdalisue/fern-hijack.vim'

	-- status line
	use 'vim-airline/vim-airline'

	-- completion and lsp
	use 'neovim/nvim-lspconfig'
	use 'nvim-lua/completion-nvim'
	use 'steelsojka/completion-buffers'
	use 'nvim-lua/lsp-status.nvim'
	use 'Iron-E/nvim-highlite'

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

local lsp_status = require('lsp-status')
local lspconfig = require('lspconfig')

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

lsp_status.register_progress()

lspconfig.pyright.setup{
	on_attach = lsp_status.on_attach;
	capabilities = lsp_status.capabilities;
}
lspconfig.ocamllsp.setup{
	on_attach = lsp_status.on_attach;
	capabilities = lsp_status.capabilities;
}
lspconfig.rust_analyzer.setup{
	on_attach = lsp_status.on_attach;
	capabilities = lsp_status.capabilities;
}
lspconfig.texlab.setup{
	on_attach = lsp_status.on_attach;
	capabilities = lsp_status.capabilities;
}

execute('autocmd BufEnter * lua require\'completion\'.on_attach()')
vim.g.completion_chain_complete_list = {
	default = {
		{ complete_items = { 'lsp', 'buffer', 'snippet' } },
		{ mode = { '<c-p>' } },
		{ mode = { '<c-n>' } }
	},
}
-- Use <Tab> and <S-Tab> to navigate through popup menu
execute('inoremap <expr> <Tab>   pumvisible() ? "\\<C-n>" : "\\<Tab>"')
execute('inoremap <expr> <S-Tab> pumvisible() ? "\\<C-p>" : "\\<S-Tab>"')
-- Set completeopt to have a better completion experience
vim.o.completeopt='menuone,noinsert,noselect'
-- Avoid showing message extra message when using completion
vim.o.shortmess=vim.o.shortmess..'c'

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
execute('colorscheme highlite')
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
	}
}
