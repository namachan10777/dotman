complete -c platformio -f

complete -c platformio      -l version -n '__fish_no_arguments' -d 'Show the version and exit.'
complete -c platformio -s f -l force   -n '__fish_no_arguments' -d 'Force to accept any confirmation prompts.'
complete -c platformio -s c -l coller  -n '__fish_no_arguments' -d 'Caller ID (service).'
complete -c platformio -s h -l help                             -d 'Show this message and exit.'

complete -c platformio -a account      -n '__fish_use_subcommand' -d 'Manage PIO Account'
complete -c platformio -a boards       -n '__fish_use_subcommand' -d 'Embedded Board Explorer'
complete -c platformio -a ci           -n '__fish_use_subcommand' -d 'Continuous Integration'
complete -c platformio -a debug        -n '__fish_use_subcommand' -d 'PIO Unified Debugger'
complete -c platformio -a device       -n '__fish_use_subcommand' -d 'Monitor device or list existing'
complete -c platformio -a home         -n '__fish_use_subcommand' -d 'PIO Home'
complete -c platformio -a init         -n '__fish_use_subcommand' -d 'Initialize PlatformIO project or update existing'
complete -c platformio -a lib          -n '__fish_use_subcommand' -d 'Library Manager'
complete -c platformio -a platform     -n '__fish_use_subcommand' -d 'Platform Manager'
complete -c platformio -a remote       -n '__fish_use_subcommand' -d 'PIO Remote'
complete -c platformio -a run          -n '__fish_use_subcommand' -d 'Process project environments'
complete -c platformio -a settings     -n '__fish_use_subcommand' -d 'Manage PlatformIO settings'
complete -c platformio -a test         -n '__fish_use_subcommand' -d 'Local Unit Testing'
complete -c platformio -a update       -n '__fish_use_subcommand' -d 'Update installed platforms, packages and libraries'
complete -c platformio -a upgrade      -n '__fish_use_subcommand' -d 'Upgrade PlatformIO to the latest version'

complete -c platformio -a forgot   -n '__fish_seen_subcommand_from account' -d "Forgot password"
complete -c platformio -a login    -n '__fish_seen_subcommand_from account' -d "Log in to PIO Account"
complete -c platformio -a logout   -n '__fish_seen_subcommand_from account' -d "Log out of PIO Account"
complete -c platformio -a password -n '__fish_seen_subcommand_from account' -d "Change password"
complete -c platformio -a register -n '__fish_seen_subcommand_from account' -d "Create new PIO Account"
complete -c platformio -a show     -n '__fish_seen_subcommand_from account' -d "PIO Account information"
complete -c platformio -a token    -n '__fish_seen_subcommand_from account' -d "Get or regenerate Authentication Token"

complete -c platformio -s l -l lib            -n '__fish_seen_subcommand_from ci'
complete -c platformio -s exclude             -n '__fish_seen_subcommand_from ci'
complete -c platformio -s b -l board          -n '__fish_seen_subcommand_from ci'
complete -c platformio -l build-dir           -n '__fish_seen_subcommand_from ci'
complete -c platformio -l keep-build-dir      -n '__fish_seen_subcommand_from ci'
complete -c platformio -s C -l project-conf   -n '__fish_seen_subcommand_from ci'
complete -c platformio -s O -l project-option -n '__fish_seen_subcommand_from ci'
complete -c platformio -s v -l verbose        -n '__fish_seen_subcommand_from ci'
complete -c platformio -s h -l help           -n '__fish_seen_subcommand_from ci'

complete -c platformio -a list    -n '__fish_seen_subcommand_from device' -d 'List devices'
complete -c platformio -a monitor -n '__fish_seen_subcommand_from device' -d 'Monitor device (Serial)'

complete -c platformio -a builtin   -n '__fish_seen_subcommand_from lib' -d 'List built-in libraries'
complete -c platformio -a install   -n '__fish_seen_subcommand_from lib' -d 'Install library'
complete -c platformio -a list      -n '__fish_seen_subcommand_from lib' -d 'List installed libraries'
complete -c platformio -a register  -n '__fish_seen_subcommand_from lib' -d 'Register a new library'
complete -c platformio -a search    -n '__fish_seen_subcommand_from lib' -d 'Search for a library'
complete -c platformio -a show      -n '__fish_seen_subcommand_from lib' -d 'Show detailed info about a library'
complete -c platformio -a stats     -n '__fish_seen_subcommand_from lib' -d 'Library Registry Statistics'
complete -c platformio -a uninstall -n '__fish_seen_subcommand_from lib' -d 'Uninstall libraries'
complete -c platformio -a update    -n '__fish_seen_subcommand_from lib' -d 'Update installed libraries'

complete -c platformio -a frameworks -n '__fish_seen_subcommand_from platform' -d 'List supported frameworks, SDKs'
complete -c platformio -a install    -n '__fish_seen_subcommand_from platform' -d 'Install new development platform'
complete -c platformio -a list       -n '__fish_seen_subcommand_from platform' -d 'List installed development platforms'
complete -c platformio -a search     -n '__fish_seen_subcommand_from platform' -d 'Search for development platform'
complete -c platformio -a show       -n '__fish_seen_subcommand_from platform' -d 'Show details about development platform'
complete -c platformio -a uninstall  -n '__fish_seen_subcommand_from platform' -d 'Uninstall development platform'
complete -c platformio -a update     -n '__fish_seen_subcommand_from platform' -d 'Update installed development platforms'

complete -c platformio -a agent   -n '__fish_seen_subcommand_from remote' -d 'Start new agent or list active'
complete -c platformio -a device  -n '__fish_seen_subcommand_from remote' -d 'Monitor remote device or list existing'
complete -c platformio -a run     -n '__fish_seen_subcommand_from remote' -d 'Process project environments remotely'
complete -c platformio -a test    -n '__fish_seen_subcommand_from remote' -d 'Remote Unit Testing'
complete -c platformio -a update  -n '__fish_seen_subcommand_from remote' -d 'Update installed Platforms, Packages and Libraries'
complete -c platformio -s a -l agent -n '__fish_seen_subcommand_from remote' -d 'Update installed Platforms, Packages and Libraries'

complete -c platformio -s e -l environment   -n '__fish_seen_subcommand_from run'
complete -c platformio -s t -l target        -n '__fish_seen_subcommand_from run'
complete -c platformio -l upload-port        -n '__fish_seen_subcommand_from run'
complete -c platformio -s d -l project-dir   -n '__fish_seen_subcommand_from run'
complete -c platformio -s s -l silent        -n '__fish_seen_subcommand_from run'
complete -c platformio -s v -l verbose       -n '__fish_seen_subcommand_from run'
complete -c platformio -l disable-auto-clean -n '__fish_seen_subcommand_from run'

complete -c platformio -a  get   -n '__fish_seen_subcommand_from  settings' -d 'Get existing setting/-s'
complete -c platformio -a oreset -n '__fish_seen_subcommand_from  settings' -d 'Reset settings to default'
complete -c platformio -a oset   -n '__fish_seen_subcommand_from  settings' -d 'Set new value for the setting'

complete -c pio -w platformio
