#!/bin/bash

if [ $(cat /etc/hostname) = "namachan" ]; then
	sed -ie 's/$ALACRITTY_FONT_SIZE/7.2/' $1
elif [ $(cat /etc/hostname) = "sakanainu" ]; then
	sed -ie 's/$ALACRITTY_FONT_SIZE/12/' $1
else
	sed -ie 's/$ALACRITTY_FONT_SIZE/9/' $1
fi
