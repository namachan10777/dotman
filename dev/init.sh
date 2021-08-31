#!/bin/bash

if [ -n $(which ruby) ]; then
	echo "ruby required"
fi

selectors=(peco sk fzf)
for selector in "${selectors[@]}"; do
	if type "$selector" > /dev/null 2>&1; then
		SELECTOR=$selector
	fi
done

if [ $SELECTOR != "" ]; then
	ENV=$(printf "priv\nckpd" | $SELECTOR)
else
	echo "[priv/ckpd]"
	read ENV
fi


if [ "${ENV}" != "priv" -a "${ENV}" != "ckpd" ]; then
	echo "invalid environment"
	exit
fi

REPO=$HOME/Project/github.com/namachan10777/scripts
git clone https://github.com/namachan10777/scripts.git $REPO
cd $REPO
./dev/dotman.rb -v -t $ENV
sudo ./dev/dotman.rb -v -t $ENV
