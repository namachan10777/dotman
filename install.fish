#!/usr/bin/fish

set DOTPATH ~/.dotfiles

echo "                _       _        "
echo "  ___  ___ _ __(_)_ __ | |_ ___  "
echo " / __|/ __| '__| | '_ \| __/ __| "
echo " \__ \ (__| |  | | |_) | |_\__ \ "
echo " |___/\___|_|  |_| .__/ \__|___/ "
echo "                 |_|             "

rm -rf ~/.dotfiles
function has
	return (type $argv[1] > /dev/null 2>&1)
end

if has "git" 
	echo "downloading by git"
	git clone "https://github.com/namachan10777/scripts.git" $DOTPATH
else if has "curl"; or has "wget"
	set tarball "https://github.com/namachan10777/scripts/archive/master.tar.gz"

	if has "curl"
		curl -L $tarball | tar zvf
	else
		wget -O - $tarball | tar zvf
	end
	
	mv -f scripts-master $DOTPATH
else
	echo "git, curl or wget required"
	exit 1
end

if test -e $DOTPATH/bin/deploy.fish
	source $DOTPATH/bin/deploy.fish
end
