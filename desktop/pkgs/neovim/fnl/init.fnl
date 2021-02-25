(module nvim-config
  {require {a aniseed.core
            s aniseed.string
            nvim aniseed.nvim
            treesitter nvim-treesitter.configs}})
(fn colorscheme [name]
  (nvim.ex.colorscheme name))

(fn list [...]
  [...])

(fn set_indent [confs]
  (do
    (nvim.ex.augroup :FileTypeIndent)
    (nvim.ex.autocmd_)
    (each [_ conf (pairs confs)]
      (nvim.ex.autocmd "FileType"
                       (s.join "" (. conf :ft))
                       "setlocal"
                       (.. "tabstop=" (. conf :w))
                       (.. "shiftwidth" (. conf :w))
                       (if (. conf :expand) :expandtab :noexpandtab)))
    (nvim.ex.augroup :END)))

; main configuration

(do
  (nvim.ex.augroup :SaveEditPos)
  (nvim.ex.autocmd_)
  (nvim.ex.autocmd :BufReadPost "*" "if line(\"'\\\"\") > 1 && line(\"'\\\"\") <= line(\"$\") | exe \"normal! g`\\\"\" | endif")
  (nvim.ex.augroup :END))

(set_indent (list
              {:ft (list :typescript :typescriptreact :javascript)
               :w 2
               :expand true}
              {:ft (list :python :haskell) :w 4 :expand true}
              {:ft (list :yaml) :w 2 :expand true}
              {:ft (list :plaintex :satysfi :tml) :w 2 :expand true}))

(nvim.set_keymap "n" "r" "diwi" { :noremap true })
(nvim.set_keymap "n" "j" "gj" { :noremap true })
(nvim.set_keymap "n" "k" "gk" { :noremap true })
(nvim.set_keymap "t" "<C-j>" "<C-\\><C-n>" { :noremap true })

; Fern
(nvim.set_var "fern#renderer" "nerdfont")
(nvim.set_keymap "n" "<space>f" ":Fern . -drawer<CR>" { :noremap true })
(nvim.set_keymap "x" "<space>f" ":Fern . -drawer<CR>" { :noremap true })

; Quickhl
(nvim.set_keymap "n" "<Space>m" "<Plug>(quickhl-manual-this)"  { :noremap false })
(nvim.set_keymap "x" "<Space>m" "<Plug>(quickhl-manual-this)"  { :noremap false })
(nvim.set_keymap "n" "<Space>M" "<Plug>(quickhl-manual-reset)" { :noremap false })
(nvim.set_keymap "x" "<Space>M" "<Plug>(quickhl-manual-reset)" { :noremap false })

(set nvim.bo.undofile true)
(set nvim.wo.foldmethod "marker")
(set nvim.o.undolevels 1024)
(set nvim.o.undoreload 8192)
(set nvim.o.swapfile false)
(set nvim.o.backup false)
(set nvim.o.writebackup false)
(set nvim.bo.tabstop 4)
(set nvim.bo.shiftwidth 4)
(set nvim.bo.expandtab false)
(set nvim.o.termguicolors true)
(set nvim.wo.number true)
(set nvim.wo.relativenumber true)
(set nvim.o.cmdheight 2)
(set nvim.o.hls true)
(set nvim.wo.list true)
(set nvim.o.listchars "tab:»-,trail:-,eol:↲,extends:»,precedes:«,nbsp:%")
(set nvim.o.hidden true)
(set nvim.o.updatetime 300)
(colorscheme "otynium")
(treesitter.setup {:ensure_installed "maintained"
                   :highlight {:enable true
                               :disable (list)}
                   :indent {:enable true}})
