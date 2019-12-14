function install
	if type omf > /dev/null 2>&1
		echo "Omf is already installed. skipping"
	else
		echo "Installing omf"
		curl -L https://get.oh-my.fish | fish
	end
end

function check
	type omf > /dev/null 2>&1
end
