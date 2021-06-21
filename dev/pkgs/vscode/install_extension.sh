#!/bin/bash

HERE=$(dirname $0)
cd $HERE

cat ./extension_list.txt | while read ext || [[ -n $ext ]]; do
	echo $ext
	code --install-extension $ext --force
done
