function __clear_gitconfig
	if test -e ~/.gitconfig
		if test -L ~/.gitconfig
			unlink ~/.gitconfig
		else
			rm -rf ~/.gitconfig
		end
	end
end

function switch_env
	if test -e ~/.gitconfig.ckpd; and test -e ~/.gitconfig.priv
		switch $argv[1]
		case "ckpd"
			__clear_gitconfig
			ln -s -f ~/.gitconfig.ckpd ~/.gitconfig
			eval (ssh-agent -c)
			echo "cookpad env"
		case "priv"
			__clear_gitconfig
			ln -s -f ~/.gitconfig.priv ~/.gitconfig
			set -gx SSH_AUTH_SOCK (gpgconf --list-dirs agent-ssh-socket)
			echo "private env"
		end
	else
		echo "private only"
	end
end
