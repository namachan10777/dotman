local minpac = require 'domain.minpac'

local load_core = function()
	minpac:load_repos()
	vim.api.nvim_command('colorscheme otynium')
end


load_core()
