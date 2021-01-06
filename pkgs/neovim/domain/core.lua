local minpac = require 'domain.minpac'

local load_core = function()
	minpac:load_repos({
		{ 'otyn0308/otynium' }
	})
	vim.api.nvim_command('colorscheme otynium')
end


load_core()
