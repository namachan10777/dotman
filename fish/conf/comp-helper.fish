function __fish_arg_len_is_zero
	set cmd (commandline -opc)
	if [ (count $cmd) -eq 1 ]
		return 0
	end
	return 1
end
