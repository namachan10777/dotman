#!/bin/bash

wget https://aur.archlinux.org/cgit/aur.git/snapshot/yay.tar.gz
tar xf yay.tar.gz
cd yay && makepkg -sri
