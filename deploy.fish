#!/usr/bin/fish
set HERE (cd (dirname (status -f)); and pwd)

function confirm
	set MSG $argv[1]
	while true
		read -P $MSG -n 1 ANS
		switch (echo $ANS)
			case y Y
				return 0
			case n N
				return 1
			case '*'
		end
	end
end

if not test $XDG_CONFIG_HOME
	set -gx XDG_CONFIG_HOME $HOME/.config
end

function applyWithConfirm
	if test -e $argv[2]
		if confirm "Are you sure to overwrite "$argv[2]"? [y/N]"
			rm -r $argv[2]
			ln -s $argv[1] $argv[2]
		end
	else
		ln -s $argv[1] $argv[2]
	end
end

function applyIptables
	if test -e /etc/iptables/iptables.rules
		if confirm "Are you sure to overwrite /etc/iptables/iptables.rules? [y/N]"
			sudo rm /etc/iptables/iptables.rules
			sudo ln -s $HERE/iptables/iptables.rules /etc/iptables/iptables.rules
		end
	else
		ln -s $HERE/iptables/iptables.rules /etc/iptables/iptables.rules
	end
	sudo systemctl enable iptables.service
end

echo "cleaning broken symlinks"
find -L . -type l | xargs unlink

if not test -e ~/.local/share/omf/init.fish
	curl -L https://get.oh-my.fish | fish
end
applyWithConfirm (echo $HERE/fish) (echo $XDG_CONFIG_HOME/fish)
applyWithConfirm (echo $HERE/git/gitconfig) (echo $HOME/.gitconfig)
applyWithConfirm (echo $HERE/neovim) (echo $XDG_CONFIG_HOME/nvim)
for f in $HERE/misc/.*
	applyWithConfirm (echo $f) (echo $HOME/(basename $f))
end
applyIptables
