import XMonad

main = xmonad def
	{ terminal = "gnome-terminal"
	, borderWidth = 8
	, layoutHook = myLayout
	}
