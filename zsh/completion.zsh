echo $ZSH_CONFIG_HOME

if [ ! -e $ZSH_CONFIG_HOME/foreign/opam.sh ]; then
	source $ZSH_CONFIG_HOME/fetch_script/opam.zsh
fi

for f in $ZSH_FOREIGN_PACKAGE_HOME/*; do
	source $f
done

for f in $ZSH_CONFIG_HOME/completion/*;do
	source $f
done
