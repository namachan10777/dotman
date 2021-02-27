(module cfg.completion
  {require {a aniseed.core
            s aniseed.string
            lsp vim.lsp
            nvim aniseed.nvim}})

(def packages ["neoclide/coc.nvim"])

(defn configure []
  (do
    (nvim.ex.inoremap
      "<silent><expr>" "<TAB>"
      "pumvisible() ? \"\\<C-n>\"" ":" "\"\\<TAB>\"")
    (nvim.ex.inoremap
      "<silent><expr>" "<S-TAB>"
      "pumvisible() ? \"\\<C-n>\"" ":" "\"\\<TAB>\"")
    (nvim.ex.inoremap
      "<silent><expr> <cr> pumvisible() ?"
      "coc#_select_confirm() :"
      "\"\\<C-g>u\\<CR>\\<c-r>=coc#on_enter()\\<CR>\"")))
