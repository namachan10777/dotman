#!/usr/bin/fish
source (dirname (status -f))/lib.fish

if test -e ~/.dotfiles
	rm -rf ~/.dotfiles
end

source $HERE/install.fish
