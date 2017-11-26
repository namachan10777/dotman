let g:opamshare = substitute(system('opam config var share'), '\n$',  '', '''')
execute 'set runtimepath^=' . g:opamshare . '/merlin/vim'
let g:deoplete#omni_patterns = {}
let g:deoplete#omni_patterns.ocaml = '\[.\w]+'
