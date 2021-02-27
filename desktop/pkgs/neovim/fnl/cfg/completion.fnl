(module cfg.completion
  {require {a aniseed.core
            s aniseed.string
            lsp vim.lsp
            nvim aniseed.nvim}})

(def packages ["neoclide/coc.nvim"])

(global check_back_space 
  (fn []
    (let [col (- (vim.fn.col ".") 1)]
      (do
        (or
          (= col 0)
          (= (string.match (string.sub (vim.fn.getline ".") col col) "%s") nil))))))

; check_back_spaceの関数定義は正しいと思うけどうまく動かない
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
