#!/bin/sh

DOTPATH=~/.dotfiles

echo "                _       _        "
echo "  ___  ___ _ __(_)_ __ | |_ ___  "
echo " / __|/ __| '__| | '_ \| __/ __| "
echo " \__ \ (__| |  | | |_) | |_\__ \ "
echo " |___/\___|_|  |_| .__/ \__|___/ "
echo "                 |_|             "

rm -rf ~/.dotfiles

git clone "https://github.com/namachan10777/scripts.git" $DOTPATH
cd $DOTPATH
python $DOTPATH/bin/deploy.py

