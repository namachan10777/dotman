#!/usr/bin/fish

source (dirname (status -f))/lib.fish

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

find $HOME/ -maxdepth 1 -xtype l | xargs unlink

if not test -e ~/.local/share/omf/init.fish
	curl -L https://get.oh-my.fish | fish
end

if test (count $argv) -gt 0 && test $argv[1] = "-i"
	applyWithConfirm (echo $HERE/fish) (echo $XDG_CONFIG_HOME/fish)
	applyWithConfirm (echo $HERE/neovim) (echo $XDG_CONFIG_HOME/nvim)
	for f in $HERE/misc/.*
		applyWithConfirm (echo $f) (echo $HOME/(basename $f))
	end
	applyIptables
else
	safeRm $XDG_CONFIG_HOME/fish
	ln -s $HERE/fish $XDG_CONFIG_HOME/fish

	safeRm $XDG_CONFIG_HOME/nvim
	ln -s $HERE/neovim $XDG_CONFIG_HOME/nvim

	for f in $HERE/misc/.*
		safeRm $HOME/(basename $f)
		ln -s $f $HOME/(basename $f)
	end

	if test -e /etc/iptables/iptables.rules
		sudo rm /etc/iptables/iptables.rules
	end
	sudo ln -s $HERE/iptables/iptables.rules /etc/iptables/iptables.rules
end

echo "deploy succeded!"
