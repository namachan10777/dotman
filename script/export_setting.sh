#!/usr/bin/sh

LOCAL_REPO=$(cd $(dirname $0)/..;pwd)

source $LOCAL_REPO/zsh/var.zsh

if [ ! -e $ZSH_CONFIG_HOME ];then
	ln -s $LOCAL_REPO/zsh $ZSH_CONFIG_HOME
fi

if [ ! -e $XDG_CONFIG_HOME/nvim ]; then
	ln -s $LOCAL_REPO/neovim $XDG_CONFIG_HOME/nvim
fi

if [ ! -e $HOME/.latexmkrc ]; then
	ln -s $LOCAL_REPO/latex/.latexmkrc $HOME/.latexmkrc
fi
	
for f in $LOCAL_REPO/script/*.sh; do
	if [ ! -e /usr/local/bin/$(basename $f) ]; then
		sudo ln -s $f /usr/local/bin/$(basename $f)
	fi
done

for f in $LOCAL_REPO/service/*.service; do
	sudo cp $f /etc/systemd/system/$(basename $f)
done
