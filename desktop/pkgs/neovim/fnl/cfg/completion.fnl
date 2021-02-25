(module cfg.completion
  {require {a aniseed.core
            s aniseed.string
            lsp vim.lsp
            nvim aniseed.nvim}})

(def packages ["neovim/nvim-lspconfig"
               "nvim-lua/completion-nvim"
               "steelsojka/completion-buffers"
               "nvim-lua/lsp-status.nvim"])

(fn attach_completion []
  (do
    (nvim.ex.augroup :AttachCompletion)
    (nvim.ex.autocmd_)
    (nvim.ex.autocmd :BufEnter "*" "lua require'completion'.on_attach()")))

(defn configure []
  (let [lsp_status (require "lsp-status")
     lspconfig (require "lspconfig")]
    (do
      (lsp_status.register_progress)
      (lspconfig.pyright.setup {:on_attach lsp_status.on_attach
                                :capabilities lsp_status.capabilities})
      (lspconfig.ocamllsp.setup {:on_attach lsp_status.on_attach
                                 :capabilities lsp_status.capabilities})
      (lspconfig.rust_analyzer.setup {:on_attach lsp_status.on_attach
                                      :capabilities lsp_status.capabilities})
      (lspconfig.texlab.setup {:on_attach lsp_status.on_attach
                               :capabilities lsp_status.capabilities})
      (lspconfig.clangd.setup {:on_attach lsp_status.on_attach
                               :capabilities lsp_status.capabilities
                               :handlers (lsp_status.extensions.clangd.setup)})
      (attach_completion)
      (set nvim.g.completion_chain_complete_list
           {:default [{:complete_items ["buffers"]}
                      {:mode [ "<c-p>"]}
                      {:mode [ "<c-n>"]}]
            :python [{:complete_items ["lsp"]}
                     {:mode [ "<c-p>"]}
                     {:mode [ "<c-n>"]}]
            :ocaml [{:complete_items ["lsp"]}
                    {:mode [ "<c-p>"]}
                    {:mode [ "<c-n>"]}]
            :plaintex [{:complete_items ["lsp"]}
                       {:mode [ "<c-p>"]}
                       {:mode [ "<c-n>"]}]
            :rust [{:complete_items ["lsp"]}
                  {:mode [ "<c-p>"]}
                  {:mode [ "<c-n>"]}]
            :c [{:complete_items ["lsp"]}
                {:mode [ "<c-p>"]}
                {:mode [ "<c-n>"]}]
            :cpp [{:complete_items ["lsp"]}
                  {:mode [ "<c-p>"]}
                  {:mode [ "<c-n>"]}]})
      (nvim.ex.inoremap "<expr>" "<Tab>" "pumvisible() ? \"\\<C-n>\" : \"\\<Tab>\"")
      (nvim.ex.inoremap "<expr>" "<S-Tab>" "pumvisible() ? \"\\<C-n>\" : \"\\<S-Tab>\"")
      (set nvim.o.completeopt "menuone,noinsert,noselect")
      (set nvim.o.shortmess (.. nvim.o.shortmess "c")))))

