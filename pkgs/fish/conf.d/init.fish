# vim:fdm=marker
set HERE (dirname (status -f))
source $HERE/env.fish

# envs {{{
set -gx PATH ~/.ghcup/bin/ ~/.npm/bin $PATH
set -gx NPM_PACKAGES ~/.npm
set -gx NODE_PATH ~/.npm/lib/node_modules $NODE_PATH
set -gx GHQ_SELECTOR peco
set -gx PATH ~/.ghcup/bin/ ~/.cache/npm/bin/ $PATH
# }}}

# abbr {{{
abbr -a py ipython
abbr -a ..  "cd ../"
abbr -a ... "cd ../../"
abbr -a v nvim
abbr -a c cd
abbr -a stdwn "shutdown -h now"
abbr -a top gotop
abbr -a btop battop
abbr -a tk tokei
abbr -a gpo git push origin
abbr -a lg lazygit
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
# }}}

# ~/tmp {{{
set TMPDIR /tmp/.(whoami)-tmp

if not test -e ~/tmp
	mkdir -p $TMPDIR
	ln -s $TMPDIR ~/tmp
end
# }}}

# starship {{{
starship init fish | source
# }}}

if test -n "$DESKTOP_SESSION"
	eval 'set -gx '(gnome-keyring-daemon --start | sed -e 's/=/ /')
end
