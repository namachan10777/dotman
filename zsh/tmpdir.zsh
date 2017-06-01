TMP_LINK=$HOME/tmp
TMP_SRC=/tmp/.$HOME/tmpdir

if [ ! -e $TMP_LINK ]
then
	mkdir -p $TMP_SRC
	ln -s $TMP_SRC $TMP_LINK
fi
