#!/usr/bin/sh

XDG_CONFIG_HOME=$HOME/.config

LOCAL_REPO=$HOME/Dropbox/Project/original/scripts

mkdir -p $HOME/.zsh
cp $LOCAL_REPO/zsh/* $HOME/.zsh

mkdir -p $XDG_CONFIG_HOME/nvim
cp $LOCAL_REPO/neovim/* $XDG_CONFIG_HOME/nvim
cp $LOCAL_REPO/latex/.latexmkrc $HOME
sudo cp $LOCAL_REPO/script/*.sh /usr/local/bin
sudo cp $LOCAL_REPO/service/* /etc/systemd/system
