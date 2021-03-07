-- Bootstrap {{{
function exists(file)
	local ok, err, code = os.rename(file, file)
	if not ok then
		if code == 13 then
			return true
		end
	end
	return ok, err
end
local install_path = vim.fn.stdpath('data')..'/site/pack/packer/opt/'
if not exists(install_path..'packer.nvim') then
	vim.cmd('!git clone https://github.com/wbthomason/packer.nvim '..install_path..'packer.nvim')
end
vim.api.nvim_command('packadd ' .. 'packer.nvim')
-- }}}

local packer = require('packer')
packer.startup(function()
	use { 'wbthomason/packer.nvim', opt = true } use 'nvim-lua/plenary.nvim' use 'norcalli/nvim.lua'
	use 'nvim-lua/popup.nvim'
	use 'nvim-telescope/telescope.nvim'
	use 'kyazdani42/nvim-tree.lua'
	use 'kyazdani42/nvim-web-devicons'
	use 'namachan10777/nvim-highlite-otynium'
	use 'lambdalisue/gina.vim'
	use 't9md/vim-quickhl'
	use 'phaazon/hop.nvim'
	use 'nvim-treesitter/nvim-treesitter'
	use 'romgrk/nvim-treesitter-context'
	use 'markonm/traces.vim'
	use 'neoclide/coc.nvim'
	use 'rafcamlet/coc-nvim-lua'

	-- language specific
	use { 'bakpakin/fennel.vim', ft = 'fennel' }
	use { 'euclidianAce/BetterLua.vim', ft = 'lua' }
	use { 'pest-parser/pest.vim', ft = 'pest' }
	use { 'ElmCast/elm-vim', ft = 'elm' }
	use {
		'prettier/vim-prettier',
		ft = { 'typescript', 'javascript', 'typescriptreact' }
	}
	use { 'jalvesaq/Nvim-R', ft = 'R' }
	use { 'qnighy/satysfi.vim', ft = 'satysfi' }
	use { 'cespare/vim-toml', ft = 'toml' }
	use { 'qnighy/lalrpop.vim', ft = 'lalrpop' }
	use { 'ron-rs/ron.vim', ft = 'ron' }
	use { 'npxbr/glow.nvim', ft = 'markdown' }
	use { 'dag/vim-fish', ft = 'fish' }
end)

-- require {{{
local nvim = require('nvim')
local treesitter = require('nvim-treesitter.configs')
-- }}}
-- coc {{{
function CheckBackSpace()
	local col = nvim.fn.col(".") - 1
	return col == 0 or string.match(string.sub(nvim.fn.getline("."), col, col), "%s") == nil
end

function ShowDocumentation()
	if (nvim.fn.index({"nvim", "help"}, nvim.bo.filetype) >= 0) then
		nvim.ex.h(nvim.fn.expand("<cword>"))
	elseif (nvim.fn["coc#rpc#ready"]() == 1) then
		nvim.fn.CocActionAsync("doHover")
	else
		nvim.command(("!" .. nvim.bo.keywordprg), nvim.fn.expand("<cword>"))
	end
end

nvim.ex.inoremap(
  "<silent><expr> <cr> pumvisible() ?",
  "coc#_select_confirm() :",
  "\"\\<C-g>u\\<CR>\\<c-r>=coc#on_enter()\\<CR>\"")
nvim.ex.nmap("<silent>", "[g", "<Plug>(coc-diagonostics-prev)")
nvim.ex.nmap("<silent>", "]g", "<Plug>(coc-diagonostics-next)")
-- 定義へ行く系
nvim.ex.nmap("<silent>", "gd", "<Plug>(coc-definition)")
nvim.ex.nmap("<silent>", "gy", "<Plug>(coc-type-definition)")
nvim.ex.nmap("<silent>", "gi", "<Plug>(coc-implementation)")
nvim.ex.nmap("<silent>", "gr", "<Plug>(coc-references)")
-- 型等ドキュメントをHoverで表示。便利
nvim.ex.nnoremap("<silent> K", ":lua ShowDocumentation()<CR>")
-- カーソルを置きっぱなしでハイライト。地味なのでコマンド欄に型表示とかにしたい……
nvim.ex.autocmd("CursorHold", "*", "silent call CocAction('highlight')")

-- }}}
-- keymaps {{{
nvim.set_keymap("n", "r", "diwi", { noremap = true })
nvim.set_keymap("n", "j", "gj", { noremap = true })
nvim.set_keymap("n", "k", "gk", { noremap = true })
nvim.set_keymap("t", "<C-j>", "<C-\\><C-n>", { noremap = true })
nvim.ex.inoremap("<silent><expr>", "<TAB>", "pumvisible() ? \"\\<C-n>\" : \"\\<TAB>\"")
nvim.ex.inoremap("<silent><expr>", "<S-TAB>", "pumvisible() ? \"\\<C-n>\" : \"\\<S-TAB>\"")

-- NvimTree <Space>t でトグル
nvim.set_keymap("n", "<space>t", ":NvimTreeToggle<CR>", { noremap = true })
nvim.set_keymap("x", "<space>t", ":NvimTreeToggle<CR>", { noremap = true })

-- Telescope <Space>f* で開く
nvim.set_keymap("n", "<Space>ff", "<cmd>lua require('telescope.builtin').find_files()<CR>", { noremap = true })
nvim.set_keymap("n", "<Space>fg", "<cmd>lua require('telescope.builtin').live_grep()<CR>", { noremap = true })
nvim.set_keymap("n", "<Space>fb", "<cmd>lua require('telescope.builtin').buffers()<CR>", { noremap = true })
nvim.set_keymap("n", "<Space>ft", "<cmd>lua require('telescope.builtin').treesitter()<CR>", { noremap = true })
nvim.set_keymap("n", "<Space>fh", "<cmd>lua require('telescope.builtin').help_tags()<CR>", { noremap = true })

-- Quickhl <Space>mでハイライト、<Space>Mでクリア
nvim.set_keymap("n", "<Space>m", "<Plug>(quickhl-manual-this)",  { noremap = false })
nvim.set_keymap("x", "<Space>m", "<Plug>(quickhl-manual-this)",  { noremap = false })
nvim.set_keymap("n", "<Space>M", "<Plug>(quickhl-manual-reset)", { noremap = false })
nvim.set_keymap("x", "<Space>M", "<Plug>(quickhl-manual-reset)", { noremap = false })

-- Glow
nvim.set_keymap("n", "<Space>p", ":Glow<CR>", { noremap = true })
nvim.set_keymap("x", "<Space>p", ":Glow<CR>", { noremap = true })
-- }}}
-- options {{{
-- シンタックスハイライトの有効化
-- Neovimはデフォルトで有効化されるはずだがそうならないファイルがある？
nvim.ex.syntax("on")
-- undoファイルを用意
nvim.bo.undofile = true
-- デフォルトの折りたたみ方式をmarkerに
nvim.wo.foldmethod = "marker"
-- undoの最大値
nvim.o.undolevels = 1024
nvim.o.undoreload = 8192
-- スワップファイルとバックアップファイルを消す
nvim.o.swapfile = false
nvim.o.backup = false
nvim.o.writebackup = false
-- デフォルトのインデント設定(最初のバッファに設定すると引き継がれる？)
nvim.bo.tabstop = 4
nvim.bo.shiftwidth = 4
nvim.bo.expandtab = false
nvim.bo.autoindent = true
-- guiと同じく2^24色使えるように(一応)
nvim.o.termguicolors = true
-- 行番号をrelativenumberで表示
nvim.wo.number = true
nvim.wo.relativenumber = true
-- コマンド欄(下の2行)を2行に設定
nvim.o.cmdheight = 2
-- 検索中のハイライトの有効化
nvim.o.hls = true
-- 不可視文字を可視化
nvim.wo.list = true
nvim.o.listchars = "tab:»-,trail:-,eol:↲,extends:»,precedes:«,nbsp:%"
-- Coc推奨設定(わからん)
nvim.o.hidden = true
nvim.o.updatetime = 300
-- カラースキーム
nvim.ex.colorscheme('otynium')
-- }}}
