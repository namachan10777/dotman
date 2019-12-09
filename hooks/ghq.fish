function install
	echo "Installing ghq"
	mkdir -p /tmp/__scripts_hooks/
	mkdir -p /tmp/__scripts_hooks/ghq
	git clone https://github.com/motemen/ghq /tmp/__scripts_hooks/ghq
	cd /tmp/__scripts_hooks/ghq
	make install
end

function check
	type ghq > /dev/null 2>&1
end
