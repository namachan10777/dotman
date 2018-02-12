set -gx LANG en_US.UTF-8
set -gx EDITOR nvim
set -gx OCAMLPARAM "_,bin-annot=1"
set -gx OPAMKEEPBUILDDIR 1

set -gx XDG_CONFIG_HOME ~/.config
set -gx SATYSFI_LIB_ROOT ~/.opam/4.05.0/lib-satysfi

eval (opam config env)

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

function pac
	packer $argv
end

switch $TERM
	case linux
	case '*'
		# fcitx setting
		set -gx XMODIFIERS "@im=fcitx"
		set -gx QT_IM_MODULE fcitx
		set -gx GTK_IM_MODULE fcitx
end

set TMPDIR /tmp/.(whoami)-tmp

if not test -e ~/tmp
	mkdir -p $TMPDIR
	ln -s $TMPDIR ~/tmp
end
