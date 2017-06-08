CLONE_TO=$HOME/tmp/.zsh_working_opam

mkdir -p $ZSH_FOREIGN_PACKAGE_HOME

if [ -e $COPY_TO ]; then
	rm -f $ZSH_FOREIGN_PACKAGE_HOME/opam_completion_zsh.sh
fi

mkdir $CLONE_TO
git clone 'https://github.com/ocaml/opam.git' $CLONE_TO
cp $CLONE_TO/shell/opam_completion_zsh.sh $ZSH_FOREIGN_PACKAGE_HOME
rm -rf $CLONE_TO
