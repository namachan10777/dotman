set -gx LANG en_US.UTF-8
set -gx EDITOR vim

eval (opam config env)

function v
	nvim $argv
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
