if test ! -e ~/.local/share/omf/init.fish
	if test ! $NONINTERACTIVE; or test $NONINTERACTIVE != 0
		curl -L https://get.oh-my.fish | fish
	end
end
