import XMonad
import System.IO
import XMonad.Hooks.DynamicLog
import XMonad.Hooks.ManageDocks
import XMonad.Util.Run(spawnPipe)
import XMonad.Layout.ResizableTile
import XMonad.Layout.Spacing
import XMonad.Hooks.EwmhDesktops(fullscreenEventHook, ewmh)
import XMonad.Util.EZConfig

main :: IO()
main = do
    xmonad $ ewmh $ docks def
        { terminal    = "xfce4-terminal"
        , borderWidth = 1
        , startupHook = myStartupHook
        , layoutHook = myLayout
        }
       `additionalKeys`
       [ ((mod1Mask .|. controlMask, xK_l     ), spawn "gnome-screensaver-command -l")
       , ((mod1Mask, xK_Print) , spawn "gnome-screenshot") ]


myStartupHook = do
    spawn "feh --bg-scale /usr/share/backgrounds/xmonad/background.png"
    spawn "xcompmgr"
    spawn "fcitx"
    spawn "gnome-screensaver"
    spawn "xmobar $HOME/.xmonad/xmobarrc"
    spawn "xautolock -time 1 -locker \"gnome-screensaver-command -l\" -notify 10 -notifier \"notify-send -t 5000 -i gtk-dialog-info 'Locking in 10 seconds'"



myLayout = avoidStruts $ smartSpacing sp (Mirror (tall) ||| tall) ||| Full
    where
        tall = ResizableTall 1 (0.03) (0.7) []
        sp = 6
