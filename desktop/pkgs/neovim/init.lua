require('plugins')
local fs = require('fs')
local completion = require('cfg.completion')
local statusline = require('cfg.statusline')

-- ad-hoc
require('packer').startup(function()

	local comp_packages = completion.packages
	for i = 1, #comp_packages do
		use(comp_packages[i])
	end
	local statusline_packages = statusline.packages
	for i = 1, #statusline_packages do
		use(statusline_packages[i])
	end

	-- Fennel
	use 'Olical/aniseed'
	use 'bakpakin/fennel.vim'

	-- package mananger
	use {'wbthomason/packer.nvim', opt = true}

	-- filer
	use 'lambdalisue/fern.vim'
	use 'lambdalisue/nerdfont.vim'
	use 'lambdalisue/fern-renderer-nerdfont.vim'
	use 'lambdalisue/fern-git-status.vim'
	use 'lambdalisue/fern-mapping-git.vim'
	use 'lambdalisue/fern-hijack.vim'

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

completion.configure()
statusline.configure()


-- fern

setvar('aniseed#env', true)
