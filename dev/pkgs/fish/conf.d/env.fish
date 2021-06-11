if not test $XDG_CONFIG_HOME
	set -gx XDG_CONFIG_HOME $HOME/.config
end

set -gx SSH_AUTH_SOCK (gpgconf --list-dirs agent-ssh-socket)
gpgconf --launch gpg-agent

set -gx NPM_PACKAGES ~/.npm
set -gx NPM_PACKAGES $HOME/.npm-packages
set -gx NODE_PATH $NPM_PACKAGES/lib/node_modules /usr/lib/node_modules
set -gx GHQ_SELECTOR sk
set -gx LANG en_US.UTF-8
set -gx EDITOR nvim
set -gx OCAMLPARAM "_,bin-annot=1"
set -gx OPAMKEEPBUILDDIR 1
set -gx GOPATH ~/.local/share/go
set -gx DOCKER_BUILDKIT 1
set -gx _JAVA_AWT_WM_NONREPARENTING 1

set PATH_LOCAL ~/.cargo/bin
set PATH_LOCAL ~/.gem/ruby/2.6.0/bin $PATH_LOCAL
set PATH_LOCAL ~/anaconda3/bin $PATH_LOCAL
set PATH_LOCAL $GOPATH/bin $PATH_LOCAL
set PATH_LOCAL ~/.cabal/bin $PATH_LOCAL
set PATH_LOCAL ~/.local/bin/ $PATH_LOCAL
set PATH_LOCAL ~/.ghcup/bin/ $PATH_LOCAL
set PATH_LOCAL ~/.local/share/gem/ruby/2.7.0/bin/ $PATH_LOCAL
set PATH_LOCAL $NPM_PACKAGES/bin/ $PATH_LOCAL
set PATH_LOCAL /opt/spack/bin/ $PATH_LOCAL
set -gx MANPATH $NPM_PACKAGES/share/man
set -gx PATH /usr/local/bin /usr/bin /bin $PATH_LOCAL
set -gx LS_COLORS (dircolors | head -n1 | sed -e "s/^.*'\(.*\)'.*/\1/")

if type opam > /dev/null 2>&1
	eval (opam env)

end
