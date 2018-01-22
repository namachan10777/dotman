let g:opamshare = substitute(system('opam config var share'), '\n$',  '', '''')
execute 'set rtp^=' . g:opamshare . '/merlin/vim'

let g:deoplete#omni#input_patterns = {}
"let g:deoplete#omni#input_patterns.ocaml = '\S\S\S*'
