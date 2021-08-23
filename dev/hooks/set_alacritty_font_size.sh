#!/bin/bash

if [ $(cat /etc/hostname) = "ikuraneko" ]; then
	sed -ie 's/$ALACRITTY_FONT_SIZE/7.2/' $1
elif [ $(cat /etc/hostname) = "sakanainu" ]; then
	sed -ie 's/$ALACRITTY_FONT_SIZE/13/' $1
elif [ $(cat /etc/hostname) = "P1724-19P15U.local" ]; then
	sed -ie 's/$ALACRITTY_FONT_SIZE/13/' $1
else
	sed -ie 's/$ALACRITTY_FONT_SIZE/9/' $1
fi
