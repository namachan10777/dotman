(module cfg.statusline
  {require {a aniseed.core
            s aniseed.string
            lsp vim.lsp
            nvim aniseed.nvim}})

(def packages [{1 "glepnir/galaxyline.nvim"
                :branch "main"
                :requires {1 "kyazdani42/nvim-web-devicons" :opt true}}])

(fn buffer_not_empty []
 (~= (nvim.fn.empty (nvim.fn.expand "%:t")) 1))
(fn checkwidth []
 (> (nvim.fn.winwidth 0) 80))
(fn lspStatus []
  (let [lsp_status (require "lsp-status")]
    (if (> (length (vim.lsp.buf_get_clients)) 0)
      (lsp_status.status)
      "no")))

(def aliases {"n" "NORMAL" "i" "INSERT" "c" "COMMAND" "V" "VISUAL" "^V" "VISUAL"})

(defn configure []
  (let
    [gl (require "galaxyline")
     gls gl.section
     colors {:bg "#282c34"
             :yellow "#fabd2f"
             :cyan "#008080"
             :darkblue "#081633"
             :green "#afd700"
             :orange "#FF8800"
             :purple "#5d4d7a"
             :magenta "#d16d9e"
             :grey "#c0c0c0"
             :blue "#0087d7"
             :red "#ec5f67"
             }]
    (do
      (set gl.short_line_list ["LuaTree" "vista" "dbui"])
      (set gls.left [{:FirstElement {:provider (lambda [] " ")
                                     :highlight [colors.blue colors.yellow]}}
                     {:ViMode {:provider (lambda [] (. aliases (nvim.fn.mode)))
                               :separator ""
                               :separator_highlight [colors.yellow
                                                     (lambda []
                                                       (if (not (buffer_not_empty)) colors.purple colors.darkblue))]
                               :highlight [colors.magenta colors.yellow "bold"]}}
                     {:FileIcon {:provider "FileIcon"
                                 :condition buffer_not_empty
                                 :highlight [(lambda [] (require "galaxyline.provider_fileinfo").get_file_icon_color) colors.darkblue]}}
                     {:FileName {:provider ["FileIcon" "FileSize"]
                                 :condition buffer_not_empty
                                 :separator ""
                                 :separator_highlight [colors.purple colors.darkblue]
                                 :highlight [colors.magenta colors.darkblue]}}
                     {:GitIcon {:provider (lambda [] "  ")
                                :condition buffer_not_empty
                                :highlight [colors.orange colors.purple]}}
                     {:GitBranch {:provider "GitBranch"
                                  :condition buffer_not_empty
                                  :highlight [colors.grey colors.purple]}}
                     {:DiffAdd {:provider "DiffAdd"
                                :icon (lambda [] " ")
                                :condition checkwidth
                                :highlight [colors.green colors.purple]}}
                     {:DiffModified {:provider "DiffModified"
                                     :icon (lambda [] " ")
                                     :condition checkwidth
                                     :highlight [colors.green colors.purple]}}
                     {:DiffRemove {:provider "DiffRemove"
                                   :icon (lambda [] " ")
                                   :condition checkwidth
                                   :highlight [colors.green colors.purple]}}
                     {:LeftEnd {:provider (lambda [] "")
                                 :separator ""
                                 :separator_highlight [colors.purple colors.bg]
                                 :highlight [colors.purple colors.purple]}}
                     {:DiagonosticError {:provider "DiagnosticError"
                                         :separator "  "
                                         :highlight [colors.red colors.bg]}}
                     {:Space {:provider (lambda [] " ")}}
                     {:DiagnosticWarn {:provider "DiagnosticWarn"
                                       :separator "  "
                                       :highlight [colors.blue colors.bg]}}
                     {:LspStatus {:provider lspStatus}}])
      (set gls.right [{:FileFormat {:provider "FileFormat"
                                    :separator ""
                                    :separator_highlight [colors.bg colors.purple]
                                    :highlight [colors.grey colors.purple]}}
                      {:LineInfo {:provider "LineColumn"
                                  :separator " | "
                                  :separator_highlight [colors.darkblue colors.purple]
                                  :highlight [colors.grey colors.purple]}}
                      {:LinePercent {:provider "LinePercent"
                                     :separator ""
                                     :separator_highlight [colors.darkblue colors.purple]
                                     :highlight [colors.grey colors.darkblue]}}
                      {:ScrollBar {:provider "ScrollBar"
                                   :highlight [colors.yellow colors.purple]}}])
      (set gls.short_line_left [{:FileTypeName {:provider "FileTypeName"
                                               :separator ""
                                               :separator_highlight [colors.purple colors.bg]
                                               :highlight [colors.grey colors.purple]}}
                                 {:BufferIcon {:provider "BufferIcon"
                                             :separator ""
                                             :separator_highlight [colors.purple colors.bg]
                                             :highlight [colors.grey colors.purple]}}])
      )))
