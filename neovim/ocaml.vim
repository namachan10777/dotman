let g:opamshare = substitute(system('opam config var share'), '\n$',  '', '''')
execute 'set rtp^=' . g:opamshare . '/merlin/vim'

let g:deoplete#omni#input_patterns = {}
" deopleteでmemhirの補完をする設定
" menhirの補完が遅い、煩いなどで現在無効化中
"let g:deoplete#omni#input_patterns.ocaml = '\S\S\S*'
