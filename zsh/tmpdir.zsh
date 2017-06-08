TMP_LINK=$HOME/tmp
TMP_SRC=/tmp/.$USERNAME/tmpdir

mkdir -p $TMP_SRC

if [ ! -e $TMP_LINK ]
then
	ln -s $TMP_SRC $TMP_LINK
fi
