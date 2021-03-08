-- Bootstrap {{{
local function exists(file)
    local ok, err, code = os.rename(file, file)
    if not ok then if code == 13 then return true end end
    return ok, err
end
local install_path = vim.fn.stdpath('data') .. '/site/pack/packer/opt/'
if not exists(install_path .. 'packer.nvim') then
    vim.cmd('!git clone https://github.com/wbthomason/packer.nvim ' ..
                install_path .. 'packer.nvim')
end
vim.api.nvim_command('packadd ' .. 'packer.nvim')
-- }}}

UseBuiltinLSP = false

local packer = require('packer')
packer.startup(function()
    -- a package manager
    use {'wbthomason/packer.nvim', opt = true}
    -- lua utility library
    use 'nvim-lua/plenary.nvim'
    -- useful mapping for neovim api
    use 'norcalli/nvim.lua'
    -- popup_menu
    use 'nvim-lua/popup.nvim'
    -- fuzzy-finder
    use 'nvim-telescope/telescope.nvim'
    -- filer
    use 'kyazdani42/nvim-tree.lua'
    -- cli icons
    use 'kyazdani42/nvim-web-devicons'
    -- colorscheme
    use 'namachan10777/nvim-highlite-otynium'
    -- git plugin
    use 'lambdalisue/gina.vim'
    -- quickhl
    use 't9md/vim-quickhl'
    -- easymotion
    use 'phaazon/hop.nvim'
    -- treesitter
    use 'nvim-treesitter/nvim-treesitter'
    -- coloring parentesis with nest level by treesitter
    use 'p00f/nvim-ts-rainbow' use 'romgrk/nvim-treesitter-context'
    -- realtime replacing preview
    use 'markonm/traces.vim'
    if UseBuiltinLSP then
        -- lsp configuration
        use 'nvim-lua/completion-nvim'
        use 'neovim/nvim-lspconfig'
        use 'nvim-lua/lsp-status.nvim'
    else
        -- coc
        use 'neoclide/coc.nvim'
    end
    -- statusline
    use 'glepnir/galaxyline.nvim'

    -- language specific
    use {'euclidianAce/BetterLua.vim', ft = 'lua'}
    use {'pest-parser/pest.vim', ft = 'pest'}
    use {'ElmCast/elm-vim', ft = 'elm'}
    use {
        'prettier/vim-prettier',
        ft = {'typescript', 'javascript', 'typescriptreact'}
    }
    use {'jalvesaq/Nvim-R', ft = 'R'}
    use {'qnighy/satysfi.vim', ft = 'satysfi'}
    use {'cespare/vim-toml', ft = 'toml'}
    use {'qnighy/lalrpop.vim', ft = 'lalrpop'}
    use {'ron-rs/ron.vim', ft = 'ron'}
    use {'npxbr/glow.nvim', ft = 'markdown'}
    use {'dag/vim-fish', ft = 'fish'}
end)

-- require {{{
local nvim = require('nvim')
local treesitter = require('nvim-treesitter.configs')
-- }}}

-- utilities {{{
local function augroup(name, hook)
	nvim.ex.augroup(name)
	nvim.ex.autocmd_()
	hook()
	nvim.ex.augroup("END")
end

local function set_indents(configs)
	augroup("ExtIndent", function ()
		nvim.ex.autocmd_()
		for i = 1, #configs do
			local config = configs[i]
			local exts = ""
			for j = 1, #config["exts"] do
				exts = exts .. "*." .. config["exts"][j]
			end
			if config["expand"] then
				nvim.ex.autocmd("BufRead,BufNewFile", exts, "setlocal", ("shiftwidth=" .. tostring(config["w"])), ("tabstop=" .. tostring(config["w"])), "expandtab")
			else
				nvim.ex.autocmd("BufRead,BufNewFile", exts, "setlocal", ("shiftwidth="..tostring(config["w"])), ("tabstop="..tostring(config["w"])), "noexpandtab")
			end
		end
	end)
end
-- }}}

-- coc {{{
function CheckBackSpace()
    local col = nvim.fn.col(".") - 1
    return col == 0 or
               string.match(string.sub(nvim.fn.getline("."), col, col), "%s") ==
               nil
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

if not UseBuiltinLSP then
    nvim.ex.inoremap("<silent><expr> <cr> pumvisible() ?",
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
end

-- }}}

-- nvim-lsp {{{
-- TODO completion.nvimが暴走する
if UseBuiltinLSP then
    -- Lsp
    local lsp_config = require('lspconfig')
    local sumneko_root_path = vim.fn.stdpath('cache')..'/lspconfig/sumneko_lua/lua-language-server'
    local sumneko_binary = sumneko_root_path.."/bin/Linux/lua-language-server"
    lsp_config.sumneko_lua.setup {
        cmd = {sumneko_binary, "-E", sumneko_root_path .. "/main.lua"};
        settings = {
            Lua = {
                runtime = {
                    version = 'LuaJIT',
                    path = vim.split(package.path, ';'),
                },
                diagnostics = {
                    globals = {'vim', 'use'},
                },
                workspace = {
                    -- Make the server aware of Neovim runtime files
                    library = {
                          [vim.fn.expand('$VIMRUNTIME/lua')] = true,
                          [vim.fn.expand('$VIMRUNTIME/lua/vim/lsp')] = true,
                    },
                },
            },
        },
    }
    lsp_config.rust_analyzer.setup{}

    augroup("CompletionConfig", function()
        nvim.ex.autocmd("BufEnter", "*", "lua require'completion'.on_attach()")
    end)
end
-- }}}

-- galaxyline {{{
local gl = require('galaxyline')
local gls = gl.section
gl.short_line_list = {'LuaTree', 'vista', 'dbui'}

local colors = {
    bg = '#282c34',
    yellow = '#fabd2f',
    cyan = '#008080',
    darkblue = '#081633',
    green = '#afd700',
    orange = '#FF8800',
    purple = '#5d4d7a',
    magenta = '#d16d9e',
    grey = '#c0c0c0',
    blue = '#0087d7',
    red = '#ec5f67'
}

local buffer_not_empty = function()
    if vim.fn.empty(vim.fn.expand('%:t')) ~= 1 then return true end
    return false
end

gls.left[1] = {
    FirstElement = {
        provider = function() return '▋' end,
        highlight = {colors.blue, colors.yellow}
    }
}
gls.left[2] = {
    ViMode = {
        provider = function()
            local alias = {
                n = 'NORMAL',
                i = 'INSERT',
                c = 'COMMAND',
                V = 'VISUAL',
                [''] = 'VISUAL'
            }
            return alias[vim.fn.mode()]
        end,
        separator = ' ',
        separator_highlight = {
            colors.yellow, function()
                if not buffer_not_empty() then
                    return colors.purple
                end
                return colors.darkblue
            end
        },
        highlight = {colors.magenta, colors.yellow, 'bold'}
    }
}
gls.left[3] = {
    FileIcon = {
        provider = 'FileIcon',
        condition = buffer_not_empty,
        highlight = {
            require('galaxyline.provider_fileinfo').get_file_icon_color,
            colors.darkblue
        }
    }
}
gls.left[4] = {
    FileName = {
        provider = {'FileName', 'FileSize'},
        condition = buffer_not_empty,
        separator = ' ',
        separator_highlight = {colors.purple, colors.darkblue},
        highlight = {colors.magenta, colors.darkblue}
    }
}

gls.left[5] = {
    GitIcon = {
        provider = function() return '  ' end,
        condition = buffer_not_empty,
        highlight = {colors.orange, colors.purple}
    }
}
gls.left[6] = {
    GitBranch = {
        provider = 'GitBranch',
        condition = buffer_not_empty,
        highlight = {colors.grey, colors.purple}
    }
}

local checkwidth = function()
    local squeeze_width = vim.fn.winwidth(0) / 2
    if squeeze_width > 40 then return true end
    return false
end

gls.left[7] = {
    DiffAdd = {
        provider = 'DiffAdd',
        condition = checkwidth,
        icon = ' ',
        highlight = {colors.green, colors.purple}
    }
}
gls.left[8] = {
    DiffModified = {
        provider = 'DiffModified',
        condition = checkwidth,
        icon = ' ',
        highlight = {colors.orange, colors.purple}
    }
}
gls.left[9] = {
    DiffRemove = {
        provider = 'DiffRemove',
        condition = checkwidth,
        icon = ' ',
        highlight = {colors.red, colors.purple}
    }
}
gls.left[10] = {
    LeftEnd = {
        provider = function() return ' ' end,
        separator = ' ',
        separator_highlight = {colors.purple, colors.bg},
        highlight = {colors.purple, colors.purple}
    }
}
gls.left[11] = {
    DiagnosticError = {
        provider = 'DiagnosticError',
        icon = '  ',
        highlight = {colors.red, colors.bg}
    }
}
gls.left[12] = {Space = {provider = function() return ' ' end}}
gls.left[13] = {
    DiagnosticWarn = {
        provider = 'DiagnosticWarn',
        icon = '  ',
        highlight = {colors.blue, colors.bg}
    }
}
if UseBuiltinLSP then
    gls.left[14] = {CocStatus = {provider = require('lsp-status').status }}
else
    gls.left[14] = {CocStatus = {provider = nvim.fn["coc#status"] }}
end
gls.right[1] = {
    FileFormat = {
        provider = 'FileFormat',
        separator = ' ',
        separator_highlight = {colors.bg, colors.purple},
        highlight = {colors.grey, colors.purple}
    }
}
gls.right[2] = {
    LineInfo = {
        provider = 'LineColumn',
        separator = ' | ',
        separator_highlight = {colors.darkblue, colors.purple},
        highlight = {colors.grey, colors.purple}
    }
}
gls.right[3] = {
    PerCent = {
        provider = 'LinePercent',
        separator = ' ',
        separator_highlight = {colors.darkblue, colors.purple},
        highlight = {colors.grey, colors.darkblue}
    }
}
gls.right[4] = {
    ScrollBar = {
        provider = 'ScrollBar',
        highlight = {colors.yellow, colors.purple}
    }
}

gls.short_line_left[1] = {
    BufferType = {
        provider = 'FileTypeName',
        separator = ' ',
        separator_highlight = {colors.purple, colors.bg},
        highlight = {colors.grey, colors.purple}
    }
}

gls.short_line_right[1] = {
    BufferIcon = {
        provider = 'BufferIcon',
        separator = ' ',
        separator_highlight = {colors.purple, colors.bg},
        highlight = {colors.grey, colors.purple}
    }
}
-- }}}

-- keymaps {{{
nvim.set_keymap("n", "r", "diwi", {noremap = true})
nvim.set_keymap("n", "j", "gj", {noremap = true})
nvim.set_keymap("n", "k", "gk", {noremap = true})
nvim.set_keymap("t", "<C-j>", "<C-\\><C-n>", {noremap = true})
nvim.ex.inoremap("<silent><expr>", "<TAB>", "pumvisible() ? \"\\<C-n>\" : \"\\<TAB>\"")
nvim.ex.inoremap([[<silent><expr> <Tab> pumvisible() ? "\<C-n>" : "\<Tab>"]])
nvim.ex.inoremap([[<silent><expr> <S-Tab> pumvisible() ? "\<C-n>" : "\<S-Tab>"]])

-- NvimTree <Space>t でトグル
nvim.set_keymap("n", "<space>t", ":NvimTreeToggle<CR>", {noremap = true})
nvim.set_keymap("x", "<space>t", ":NvimTreeToggle<CR>", {noremap = true})

-- Telescope <Space>f* で開く
nvim.set_keymap("n", "<Space>ff",
                "<cmd>lua require('telescope.builtin').find_files()<CR>",
                {noremap = true})
nvim.set_keymap("n", "<Space>fg",
                "<cmd>lua require('telescope.builtin').live_grep()<CR>",
                {noremap = true})
nvim.set_keymap("n", "<Space>fb",
                "<cmd>lua require('telescope.builtin').buffers()<CR>",
                {noremap = true})
nvim.set_keymap("n", "<Space>ft",
                "<cmd>lua require('telescope.builtin').treesitter()<CR>",
                {noremap = true})
nvim.set_keymap("n", "<Space>fh",
                "<cmd>lua require('telescope.builtin').help_tags()<CR>",
                {noremap = true})

-- Quickhl <Space>mでハイライト、<Space>Mでクリア
nvim.set_keymap("n", "<Space>m", "<Plug>(quickhl-manual-this)",
                {noremap = false})
nvim.set_keymap("x", "<Space>m", "<Plug>(quickhl-manual-this)",
                {noremap = false})
nvim.set_keymap("n", "<Space>M", "<Plug>(quickhl-manual-reset)",
                {noremap = false})
nvim.set_keymap("x", "<Space>M", "<Plug>(quickhl-manual-reset)",
                {noremap = false})

-- Glow
nvim.set_keymap("n", "<Space>p", ":Glow<CR>", {noremap = true})
nvim.set_keymap("x", "<Space>p", ":Glow<CR>", {noremap = true})
-- }}}

-- autocmds {{{

augroup("SaveEditPos", function ()
	nvim.ex.autocmd("BufReadPost", "*", "if line(\"'\\\"\") > 1 && line(\"'\\\"\") <= line(\"$\") | exe \"normal! g`\\\"\" | endif")
end)

augroup("OptimizeCmdWindow", function ()
    nvim.ex.autocmd("CmdwinEnter", "[:\\/\\?=]", "setlocal", "nonumber")
    nvim.ex.autocmd("CmdwinEnter", "[:\\/\\?=]", "setlocal", "signcolumn=no")
    -- 2字以下のコマンドをコマンドウィンドウから削除
    nvim.ex.autocmd("CmdwinEnter", ":", "g/^..\\?$/d")
end)

set_indents({
	{ exts={"lua"}, w = 4, expand = true }
})
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

-- treesitter {{{
treesitter.setup {
    highlight = {
        enable = true,
    },
    indent = {
        enable = true,
    },
    ensure_installed = 'all',
    rainbow = {
        enable = true,
    },
}
-- }}}
