set -gx LANG en_US.UTF-8
set -gx EDITOR nvim
set -gx OCAMLPARAM "_,bin-annot=1"
set -gx OPAMKEEPBUILDDIR 1
if type opam > /dev/null 2>&1
	eval (opam env)
end

set -gx XDG_CONFIG_HOME ~/.config

set PATH ~/.cargo/bin $PATH
set PATH ~/.gem/ruby/2.6.0/bin $PATH
set PATH ~/anaconda3/bin $PATH

function py
	ipython
end

function ...
	cd ../../
end

function ....
	cd ../../../
end

function grep
	echo "use 'rg'!"
end

function v
	nvim $argv
end

function c
	cd $argv
end

function l
	ls $argv
end

function m
	mv $argv
end

function clipb
	xsel --clipboard --input
end

function pac
	packer $argv
end

function stdwn
	sudo shutdown -h now
end

function diff
	icdiff $argv
end


set TMPDIR /tmp/.(whoami)-tmp

if not test -e ~/tmp
	mkdir -p $TMPDIR
	ln -s $TMPDIR ~/tmp
end

switch $TERM
	case linux
		sudo loadkeys ~/.keystrings
	case '*'
		# fcitx setting
		set -gx XMODIFIERS "@im=fcitx"
		set -gx QT_IM_MODULE fcitx
		set -gx GTK_IM_MODULE fcitx
end
