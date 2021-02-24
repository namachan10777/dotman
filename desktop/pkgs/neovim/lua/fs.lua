local fs = {}
fs.mkdir = function(dir)
	vim.api.nvim_command("!mkdir -p "..dir)
end
fs.exists = function(file)
	local ok, err, code = os.rename(file, file)
	if not ok then
		if code == 13 then
			return true
		end
	end
	return ok, err
end
return fs
