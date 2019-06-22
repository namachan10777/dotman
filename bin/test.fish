#!/usr/bin/fish

source (dirname (status -f))/lib.fish

set STATUS 0

function check
	set SOURCE $argv[1]
	set TARGET $argv[2]
	if test -e $TARGET
		if test ! -L $TARGET
			echo "✘ "$SOURCE" ... yet linked"
			set STATUS 1
		else if test ! (readlink -f $SOURCE) = (readlink $TARGET)
			echo "✘ "$SOURCE" ... wrong linked"
			set STATUS 1
		else
			echo "✔ "$SOURCE" ... linked" 
		end
	else
		echo "✘ "$SOURCE" ... doesn't exist"
		set STATUS 1
	end
end

for f in $HERE/misc/.*
	check $f $HOME/(basename $f)
end

check $HERE/neovim $XDG_CONFIG_HOME/nvim
check $HERE/fish $XDG_CONFIG_HOME/fish

exit $STATUS
