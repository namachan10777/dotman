#!/usr/bin/fish

set DOTPATH ~/.dotfiles

if test -e git
	git clone "https://github.com/namachan10777/scripts.git" $DOTPATH
else if test -o (test -e curl) (test -e wget)
	set tarball "https://github.com/namachan10777/scripts/archive/master.tar.gz"

	if test -e curl
		curl -L $tarball | tar zvf
	else
		wget -O - $tarball | tar zvf
	end
	
	mv -f scripts-master $DOTPATH
else
	echo "git, curl or wget required"
end

if test -e $DOTPATH/apply.fish
	source $DOTPATH/apply.fish
end
