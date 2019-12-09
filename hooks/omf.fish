function install
	echo "Installing omf"
	if test ! -e ~/.local/share/omf/init.fish
		if test ! $NONINTERACTIVE; or test $NONINTERACTIVE != 0
			curl -L https://get.oh-my.fish | fish
		end
	end
end

function check
	type omf > /dev/null 2>&1
end
