local global = require('domain.global')
local fs = require('util.fs')
local minpac = setmetatable({}, { __index = { repos = {}, config_files = {} } })

function minpac:load_repos(repos)
	local minpac_dir = fs.concat({global.config_dir,'nvim','pack','minpac','opt','minpac'})
	local cmd = 'git clone https://github.com/k-takata/minpac ' .. minpac_dir
	if vim.fn.has('vim_starting') then
		if not fs.isdir(minpac_dir) then
			os.execute(cmd)
		end
		vim.api.nvim_command('packadd minpac')
		vim.api.nvim_call_function('minpac#init', {})
		vim.api.nvim_call_function('minpac#add', { 'k-takana/minpac', { type='opt' }})
		for i = 1, #repos do
			if #repos == 1 then
				vim.api.nvim_call_function('minpac#add', { repos[i][1] })
			else
				vim.api.nvim_call_function('minpac#add', { repos[i][1], repos[i][2] })
			end
		end
	end
end

return minpac
