if not test $XDG_CONFIG_HOME
	set -gx XDG_CONFIG_HOME $HOME/.config
end

set -gx LANG en_US.UTF-8
set -gx EDITOR nvim
set -gx OCAMLPARAM "_,bin-annot=1"
set -gx OPAMKEEPBUILDDIR 1
set -gx GOPATH ~/.local/share/go
set -gx GHQ_SELECTOR peco
set -gx DOCKER_BUILDKIT 1
if type opam > /dev/null 2>&1
	eval (opam env)
end

set PATH ~/.cargo/bin $PATH
set PATH ~/.gem/ruby/2.6.0/bin $PATH
set PATH ~/anaconda3/bin $PATH
set PATH $GOPATH/bin $PATH
set PATH ~/.cabal/bin $PATH
set PATH ~/.local/bin/ $PATH
set -gx LS_COLORS (dircolors | head -n1 | sed -e "s/^.*'\(.*\)'.*/\1/")
