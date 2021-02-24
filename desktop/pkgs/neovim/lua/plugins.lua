-- bootstrapping codes {{{
local fs = require('fs')
local execute = vim.api.nvim_command
local fn = vim.fn
local install_path = fn.stdpath('data')..'/site/pack/packer/opt/packer.nvim'
if not fs.exists(install_path) then
	execute('!git clone https://github.com/wbthomason/packer.nvim '..install_path)
end
execute 'packadd packer.nvim'
-- }}}

return require('packer').startup(function()
	use {'wbthomason/packer.nvim', opt = true}
	use 'otyn0308/otynium'
	use 'lambdalisue/fern.vim'
	use 'lambdalisue/nerdfont.vim'
	use 'lambdalisue/fern-renderer-nerdfont.vim'
	use 'lambdalisue/fern-git-status.vim'
	use 'lambdalisue/fern-mapping-git.vim'
	use 'lambdalisue/fern-hijack.vim'
	use 'vim-airline/vim-airline'
	-- Syntax highlights
	use 'JuliaEditorSupport/julia-vim'
	use 'pest-parser/pest.vim'
	use 'ElmCast/elm-vim'
	use 'prettier/vim-prettier'
	use 'jalvesaq/Nvim-R'
	use 'qnighy/satysfi.vim'
	use 'cespare/vim-toml'
	use 'qnighy/lalrpop.vim'
	use 'namachan10777/tml.vim'
	use 'ron-rs/ron.vim'
	-- utilities
	use 't9md/vim-quickhl'
	use 'nvim-treesitter/nvim-treesitter'
end)
