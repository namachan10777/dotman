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
			witSu "rm /etc/iptables/iptables.rules"
			withSu "cp "$HERE"/iptables/iptables.rules /etc/iptables/iptables.rules"
		end
	else
		if test -L /etc/iptables/iptables.rules
			withSu "unlink /etc/iptables/iptables.rules"
		end
		withSu "cp "$HERE"/iptables/iptables.rules /etc/iptables/iptables.rules"
	end
	if has "systemctl"
		withSu "systemctl enable iptables.service"
	end
end

set INVALID_LINKS (find $HOME/ -maxdepth 1 -xtype l)
if test ! -z "$INVALID_LINKS"
	unlink $INVALID_LINKS
end

if test ! -e /etc/iptables
	withSu "mkdir /etc/iptables"
end

if test (count $argv) -gt 0; and test $argv[1] = "-i"
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
		withSu "rm /etc/iptables/iptables.rules"
	else if test -L /etc/iptables/iptables.rules
		withSu "unlink /etc/iptables/iptables.rules"
	end
	withSu "cp "$HERE"/iptables/iptables.rules /etc/iptables/iptables.rules"
	if has "systemctl"
		withSu "systemctl enable iptables.service"
	end
end

echo "Executing hooks..."

for f in $HERE/hooks/*.fish
	source $f
end

echo "deploy succeded!"
