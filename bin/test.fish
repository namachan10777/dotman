#!/usr/bin/fish

source (dirname (status -f))/lib.fish

function check
	set SOURCE $argv[1]
	set TARGET $argv[2]
	if test -e $TARGET
		if test ! -L $TARGET
			echo "✘ "$SOURCE" ... yet linked"
		else if test ! (readlink -f $SOURCE) = (readlink $TARGET)
			echo "✘ "$SOURCE" ... wrong linked"
		else
			echo "✔ "$SOURCE" ... linked" 
		end
	else
		echo "✘ "$SOURCE" ... doesn't exist"
	end
end

for f in $HERE/misc/.*
	check $f $HOME/(basename $f)
end

check $HERE/neovim $XDG_CONFIG_HOME/nvim
check $HERE/fish $XDG_CONFIG_HOME/fish
