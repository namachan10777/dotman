set HERE (cd (dirname (status -f))/../; and pwd)

function install
	echo "Installing ghq"
	rm -rf /tmp/__scripts_hooks/ghq
	mkdir -p /tmp/__scripts_hooks/
	mkdir -p /tmp/__scripts_hooks/ghq
	git clone https://github.com/motemen/ghq /tmp/__scripts_hooks/ghq
	cd /tmp/__scripts_hooks/ghq
	make install
	
	cd $HERE
end

function check
	type ghq > /dev/null 2>&1
end
