# vim:fdm=marker
set HERE (dirname (status -f))
source $HERE/env.fish

# envs {{{
set -gx GHQ_SELECTOR peco
set -gx 
# }}}

# abbr {{{
abbr -a py ipython
abbr -a .   "cd ../"
abbr -a ..  "cd ../../"
abbr -a ... "cd ../../../"
abbr -a v nvim
abbr -a c cd
abbr -a stdwn "shutdown -h now"
# }}}

# alias {{{
function ls
	exa $argv
end

function ll
	exa -l $argv
end

function lt
	exa -T $argv
end

function clipb
	xsel --clipboard --input
end

function diff
	icdiff $argv
end
# }}}

# ~/tmp {{{
set TMPDIR /tmp/.(whoami)-tmp

if not test -e ~/tmp
	mkdir -p $TMPDIR
	ln -s $TMPDIR ~/tmp
end
# }}}

# terminal local {{{
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
# }}}
