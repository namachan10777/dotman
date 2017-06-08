case "$TERM" in
	linux)
		export LANG="C"
		export LANGUAGE="C"
		export LC_MESSAGES="C"
		export LC_TYPE="C"
		export LC_COLLATE="C"
		;;
	*)
		export LANG="ja_JP.UTF-8"
		export LANGUAGE="ja_JP.UTF-8"
		export LC_MESSAGES="ja_JP.UTF-8"
		export LC_TYPE="ja_JP.UTF-8"
		export LC_COLLATE="ja_JP.UTF-8"
		;;
esac
