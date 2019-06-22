#!/usr/bin/fish

set HERE (cd (dirname (status -f))/../; and pwd)

if not test $XDG_CONFIG_HOME
	set -gx XDG_CONFIG_HOME $HOME/.config
end

function safeRm
	if test -L $argv[1]
		unlink $argv[1]
	else if test -e $argv[1]
		mv $argv[1] $argv[1].origin
	end
end

function withSu
	if test (id -u -n) = "root"
		eval $argv[1]
	else
		eval "sudo "$argv[1]
	end
end
		

function confirm
	set MSG $argv[1]
	while true
		read -P $MSG -n 1 ANS
		switch (echo $ANS)
			case y Y
				return 0
			case n N
				return 1
			case '*'
		end
	end
end
