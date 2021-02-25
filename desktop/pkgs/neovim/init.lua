-- Bootstrap
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
vim.cmd 'packadd packer.nvim'
if not exists(install_path..'aniseed') then
	vim.cmd('!git clone https://github.com/Olical/aniseed '..install_path..'aniseed')
end
vim.cmd 'packadd aniseed'
vim.api.nvim_set_var('aniseed#env', true)
