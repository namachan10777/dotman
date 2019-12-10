function install
	curl https://git.io/fisher --create-dirs -sLo ~/.config/fish/functions/fisher.fish
end

function check
	type fisher > /dev/null 2>&1
end
