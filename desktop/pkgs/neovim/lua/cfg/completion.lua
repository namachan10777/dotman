local completion = {}

completion.packages = {
	-- completion and lsp
	'neovim/nvim-lspconfig',
	'nvim-lua/completion-nvim',
	'steelsojka/completion-buffers',
	{ 'aca/completion-tabnine', run = './install.sh' },
	'nvim-lua/lsp-status.nvim',
}

function LspStatus()
	if #vim.lsp.buf_get_clients() > 0 then
		local max_length = 50
		local status = require('lsp-status').status()
		if #status < max_length then
			for i = 0, #status - max_length do
				status = status .. ' '
			end
			return status
		end
		return status
	end
end

function completion.configure()
	local lsp_status = require('lsp-status')
	local lspconfig = require('lspconfig')

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
	lspconfig.clangd.setup{
		on_attach = lsp_status.on_attach;
		capabilities = lsp_status.capabilities;
		handlers = lsp_status.extensions.clangd.setup();
	}

	vim.cmd('autocmd BufEnter * lua require\'completion\'.on_attach()')
	vim.g.completion_chain_complete_list = {
		default = {
			{ complete_items = { 'tabnine', 'buffers' } },
			{ mode = { '<c-p>' } },
			{ mode = { '<c-n>' } }
		},
		python = {
			{ complete_items = { 'lsp' } },
			{ mode = { '<c-p>' } },
			{ mode = { '<c-n>' } }
		},
		ocaml = {
			{ complete_items = { 'lsp' } },
			{ mode = { '<c-p>' } },
			{ mode = { '<c-n>' } }
		},
		plaintex = {
			{ complete_items = { 'lsp' } },
			{ mode = { '<c-p>' } },
			{ mode = { '<c-n>' } }
		},
		rust = {
			{ complete_items = { 'lsp' } },
			{ mode = { '<c-p>' } },
			{ mode = { '<c-n>' } }
		},
		c = {
			{ complete_items = { 'lsp' } },
			{ mode = { '<c-p>' } },
			{ mode = { '<c-n>' } }
		},
		cpp = {
			{ complete_items = { 'lsp' } },
			{ mode = { '<c-p>' } },
			{ mode = { '<c-n>' } }
		},
	}
	-- Use <Tab> and <S-Tab> to navigate through popup menu
	vim.cmd('inoremap <expr> <Tab>   pumvisible() ? "\\<C-n>" : "\\<Tab>"')
	vim.cmd('inoremap <expr> <S-Tab> pumvisible() ? "\\<C-p>" : "\\<S-Tab>"')
	-- Set completeopt to have a better completion experience
	vim.o.completeopt='menuone,noinsert,noselect'
	-- Avoid showing message extra message when using completion
	vim.o.shortmess=vim.o.shortmess..'c'
end

return completion
