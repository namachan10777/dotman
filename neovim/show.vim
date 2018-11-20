set number
set noswapfile
syntax on
filetype plugin on
set tabstop=4
set shiftwidth=4
set noexpandtab
set guicursor=
set nohlsearch

" 以下カラースキーム設定
set background=dark
let g:artesanal_transp_bg = 0
colorscheme artesanal

" 透過関連
highlight Normal ctermbg=NONE guibg=NONE
highlight NonText ctermbg=NONE guibg=NONE
highlight SpecialKey ctermbg=NONE guibg=NONE
highlight EndOfBuffer ctermbg=NONE guibg=NONE
highlight LineNr ctermbg=NONE guibg=NONE
