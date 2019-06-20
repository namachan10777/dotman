#!/usr/bin/fish

set DOTPATH ~/.dotfiles

echo "                _       _        "
echo "  ___  ___ _ __(_)_ __ | |_ ___  "
echo " / __|/ __| '__| | '_ \| __/ __| "
echo " \__ \ (__| |  | | |_) | |_\__ \ "
echo " |___/\___|_|  |_| .__/ \__|___/ "
echo "                 |_|             "

rm -rf ~/.dotfiles

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

if test -e $DOTPATH/deploy.fish
	source $DOTPATH/deploy.fish
end
