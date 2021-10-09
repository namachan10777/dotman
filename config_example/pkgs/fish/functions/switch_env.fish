function __clear_gitconfig
	if test -e $argv[1]
		if test -L $argv[1]
			unlink $argv[1]
		else
			rm -rf $argv[1]
		end
	end
end

function switch_env
	if  test -e ~/.gitconfig.ckpd; and test -e ~/.gitconfig.priv; and test -e ~/.ssh/config.ckpd; and test -e ~/.ssh/config.priv
		switch $argv[1]
		case "ckpd"
			__clear_gitconfig ~/.gitconfig
			__clear_gitconfig ~/.ssh/config
			ln -s -f ~/.gitconfig.ckpd ~/.gitconfig
			ln -s -f ~/.ssh/config.ckpd ~/.ssh/config
			eval (ssh-agent -c)
			echo "cookpad env"
		case "priv"
			__clear_gitconfig ~/.gitconfig
			__clear_gitconfig ~/.ssh/config
			ln -s -f ~/.gitconfig.priv ~/.gitconfig
			ln -s -f ~/.ssh/config.priv ~/.ssh/config
			set -gx SSH_AUTH_SOCK (gpgconf --list-dirs agent-ssh-socket)
			echo "private env"
		end
	else
		echo "private only"
	end
end
