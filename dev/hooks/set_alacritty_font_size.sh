#!/bin/bash

if [ $(uname) = "Darwin" ]; then
	HOSTNAME=$(hostname)
elif [ $(uname) = "Linux" ]; then
	HOSTNAME=$(cat /etc/hostname)
fi

if [ $HOSTNAME = "ikuraneko" ]; then
	sed -ie 's/$ALACRITTY_FONT_SIZE/7.2/' $1
elif [ $HOSTNAME = "sakanainu" ]; then
	sed -ie 's/$ALACRITTY_FONT_SIZE/13/' $1
elif [ $HOSTNAME = "P1724-19P15U.local" ]; then
	sed -ie 's/$ALACRITTY_FONT_SIZE/13/' $1
else
	sed -ie 's/$ALACRITTY_FONT_SIZE/9/' $1
fi
