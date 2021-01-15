#!/bin/bash

mkdir -p $HOME/Project/github.com/neovim
git clone https://github.com/neovim/neovim $HOME/Project/github.com/neovim/neovim

cd $HOME/Project/github.com/neovim/neovim
make CMAKE_BUILD_TYPE=RelWithDebInfo
sudo make install
