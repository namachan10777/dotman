local global = {}
local home = os.getenv("HOME")
local path_sep = global.is_windows and '\\' or '/'
local os_name = vim.loop.os_uname().sysname

function global:load_variables()
	self.config_dir = home .. '/.config'
	self.path_sep = path_sep
	self.home = home
end

global:load_variables()

return global
