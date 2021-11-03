complete -c dotman -n "__fish_use_subcommand" -s h -l help -d 'Print help information'
complete -c dotman -n "__fish_use_subcommand" -f -a "deploy"
complete -c dotman -n "__fish_use_subcommand" -f -a "dry-run"
complete -c dotman -n "__fish_use_subcommand" -f -a "completion"
complete -c dotman -n "__fish_use_subcommand" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c dotman -n "__fish_seen_subcommand_from deploy" -s c -l config -r
complete -c dotman -n "__fish_seen_subcommand_from deploy" -s s -l scenario -r
complete -c dotman -n "__fish_seen_subcommand_from deploy" -s V -l verbose
complete -c dotman -n "__fish_seen_subcommand_from deploy" -s h -l help -d 'Print help information'
complete -c dotman -n "__fish_seen_subcommand_from dry-run" -s c -l config -r
complete -c dotman -n "__fish_seen_subcommand_from dry-run" -s s -l scenario -r
complete -c dotman -n "__fish_seen_subcommand_from dry-run" -s V -l verbose
complete -c dotman -n "__fish_seen_subcommand_from dry-run" -s h -l help -d 'Print help information'
complete -c dotman -n "__fish_seen_subcommand_from completion" -s s -l shell -r -f -a "{fish	,zsh	,bash	,elvish	,powershell	}"
complete -c dotman -n "__fish_seen_subcommand_from completion" -s h -l help -d 'Print help information'
complete -c dotman -n "__fish_seen_subcommand_from help" -s h -l help -d 'Print help information'
