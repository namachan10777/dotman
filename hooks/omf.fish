function install
	echo "Installing omf"
	curl -L https://get.oh-my.fish | fish
end

function check
	type omf > /dev/null 2>&1
end
