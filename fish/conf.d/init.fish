set HERE (cd (dirname (status -f))/../; and pwd)
source $HERE/env.fish

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

if test $TERM
	switch $TERM
		case linux
			sudo loadkeys ~/.keystrings
		case '*'
			# fcitx setting
			set -gx XMODIFIERS "@im=fcitx"
			set -gx QT_IM_MODULE fcitx
			set -gx GTK_IM_MODULE fcitx
	end
end
