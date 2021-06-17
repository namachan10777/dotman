#!/bin/bash

if [ $# == 2 -a -f $1 ]; then
	mkdir -p $(dirname $2)
	cp $1 "$2"
else
	echo "invalid argument"
	exit 1
fi
