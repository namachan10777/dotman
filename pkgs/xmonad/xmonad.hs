import XMonad
import System.IO
import XMonad.Hooks.DynamicLog
import XMonad.Hooks.ManageDocks
import XMonad.Util.Run(spawnPipe)
import XMonad.Layout.ResizableTile
import XMonad.Layout.Spacing
import XMonad.Util.NamedWindows(getName)
import XMonad.Hooks.EwmhDesktops(fullscreenEventHook, ewmh)
import XMonad.Util.EZConfig
import System.Directory
import System.Posix.Files
import Data.List
import Data.Ord
import Control.Monad
import qualified XMonad.StackSet as S


main :: IO()
main = do
    workspaceLogfile <- return "/tmp/.xmonad-workspace-log"
    titleLogfile <- return "/tmp/.xmonad-title-log"
    prepareLogFile workspaceLogfile
    prepareLogFile titleLogfile
    xmonad $ ewmh $ docks def
        { terminal    = "alacritty"
        , borderWidth = 1
        , startupHook = myStartupHook
        , layoutHook = myLayout
        , modMask = myModMask
        , logHook = eventLogHook workspaceLogfile titleLogfile
        }
       `additionalKeys`
       [ ((myModMask .|. controlMask, xK_l     ), spawn "gnome-screensaver-command -l")
       , ((myModMask, xK_Print) , spawn "gnome-screenshot") ]

prepareLogFile :: [Char] -> IO()
prepareLogFile name = do
    de <- doesFileExist name
    case de of
        True -> return ()
        _    -> createNamedPipe name stdFileMode
    return ()

getWorkspaceLog :: X String
getWorkspaceLog = do
      winset <- gets windowset
      let currWs = S.currentTag winset
      let wss    = S.workspaces winset
      let wsIds  = map S.tag   $ wss
      let wins   = map S.stack $ wss
      let (wsIds', wins') = sortById wsIds wins
      return . (foldl (\acc id -> (fmt currWs wins' id) ++ " " ++ acc) " ") . reverse $ wsIds'
      where
         hasW = not . null
         idx = flip (-) 1 . read
         sortById ids xs = unzip $ sortBy (comparing fst) (zip ids xs)
         fmt cw ws id
              | id == cw            = " \63022"
              | hasW $ ws !! idx id = " \61842"
              | otherwise           = " \63023"

getTitleLog :: X String
getTitleLog = do
    winset <- gets windowset
    title <- maybe (return "") (fmap show . getName) . S.peek $ winset
    return title

eventLogHook :: FilePath -> FilePath -> X ()
eventLogHook workspaceLog titleLog = do
    io . appendFile workspaceLog . (++ "\n") =<< getWorkspaceLog
    io . appendFile titleLog . (++ "\n") =<< getTitleLog

myStartupHook = do
    spawn "feh --bg-scale /usr/share/backgrounds/xmonad/background.png"
    spawn "picom -c -D 5"
    spawn "fcitx"
    spawn "gnome-screensaver"
    spawn "polybar example"
    spawn "xautolock -time 1 -locker \"gnome-screensaver-command -l\" -notify 10 -notifier \"notify-send -t 5000 -i gtk-dialog-info 'Locking in 10 seconds'"

myModMask = mod4Mask -- Superkey

myLayout = avoidStruts $ smartSpacing sp (Mirror (tall) ||| tall) ||| Full
    where
        tall = ResizableTall 1 (0.03) (0.7) []
        sp = 6
