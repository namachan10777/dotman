local global = require('domain.global')

local fs = {}

function fs.exists(file)
	local ok, err, code = os.rename(file, file)
	if not ok then
		if code == 13 then
			return true
		end
	end
	return ok, err
end

function fs.mkdir(path)
	return os.execute('mkdir '..path)
end

function fs.isdir(path)
	return fs.exists(path .. '/')
end

function fs.concat(path_elems)
	return table.concat(path_elems, global.path_sep)
end

return fs
