(module nvim-packages
  {require {a aniseed.core
            s aniseed.string
            completion_cfg cfg.completion
            statusline_cfg cfg.statusline
            nvim aniseed.nvim
            packer packer}})

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
                       (s.join "," conf.ft)
                       "setlocal"
                       (.. "tabstop=" conf.w)
                       (.. "shiftwidth=" conf.w)
                       (if conf.expand :expandtab :noexpandtab)))
    (nvim.ex.augroup :END)))

(packer.startup (lambda []
                  (do
                    (each [_ pkg (pairs completion_cfg.packages)]
                      (use pkg))
                    (each [_ pkg (pairs statusline_cfg.packages)]
                      (use pkg))
                    ; FennelをLuaにコンパイルしたりする(ブートストラップに必要なので:opt true
                    (use { 1 "Olical/aniseed" :opt true } )
                    ; プラグインマネージャ(ブートストラップに必要なので:opt true
                    (use { 1 "wbthomason/packer.nvim" :opt true } )
                    ; fuzzy finder（見た目が豪華で外部依存が無く速い）
                    (use { 1 "nvim-telescope/telescope.nvim" :requires ["nvim-lua/popup.nvim" "nvim-lua/plenary.nvim"] })
                    ; スクロールバーを表示（かっこいいね）
                    ; Tree表示プラグイン fzf-vimと同じようなキーバインド
                    (use "kyazdani42/nvim-web-devicons")
                    (use "kyazdani42/nvim-tree.lua")
                    ; Lsp等のシンタックスハイライトを可能にする。
                    ; nvim-highliteをforkしてotyniumっぽい色にした
                    (use "namachan10777/nvim-highlite-otynium")
                    (use "bakpakin/fennel.vim" :ft "fennel")
                    ; Luaのシンタックスハイライトの改善
                    (use { 1 "euclidianAce/BetterLua.vim" :ft "lua" })
                    (use { 1 "pest-parser/pest.vim" :ft "pest" })
                    (use { 1 "ElmCast/elm-vim" :ft "elm"})
                    (use { 1 "prettier/vim-prettier" :ft ["typescript" "typescriptreact" "javascript"]})
                    (use { 1 "jalvesaq/Nvim-R" :ft "R" })
                    (use { 1 "qnighy/satysfi.vim" :ft "satysfi" })
                    (use { 1 "cespare/vim-toml" :ft "toml" })
                    (use { 1 "qnighy/lalrpop.vim" :ft "lalrpop" }) (use { 1 "namachan10777/tml.vim" :ft "tml" })
                    (use { 1 "ron-rs/ron.vim" :ft "ron" })
                    (use { 1 "npxbr/glow.nvim" :ft "markdown" })
                    (use { 1 "dag/vim-fish" :ft "fish" })
                    (use "markonm/traces.vim")
                    ; Git操作
                    (use "lambdalisue/gina.vim")
                    ; <C-m>でカーソルがある位置の単語にハイライト<C-M>でクリア
                    (use "t9md/vim-quickhl")
                    ; easymotion
                    (use "phaazon/hop.nvim")
                    ; TreeSitter系を有効にする
                    (use "nvim-treesitter/nvim-treesitter")
                    ; 今居る関数の宣言が見える
                    (use "romgrk/nvim-treesitter-context"))))

; 遅延読み込みしておく（moduleで読み込むとブートストラップの際にパッケージリストを取得できず困る)
(def treesitter (require "nvim-treesitter.configs"))

; 別ファイルに分けたcompletionとstatuslineの設定を実行
(completion_cfg.configure)
(statusline_cfg.configure)

; 編集位置保存の設定
(do
  (nvim.ex.augroup :SaveEditPos)
  (nvim.ex.autocmd_)
  (nvim.ex.autocmd "BufReadPost" "*" "if line(\"'\\\"\") > 1 && line(\"'\\\"\") <= line(\"$\") | exe \"normal! g`\\\"\" | endif")
  (nvim.ex.augroup :END))

; コマンドラインウィンドウ
(do
  (nvim.ex.autocmd "CmdwinEnter" "[:\\/\\?=]" "setlocal" "nonumber")
  (nvim.ex.autocmd "CmdwinEnter" "[:\\/\\?=]" "setlocal" "signcolumn=no")
  ; 2字以下のコマンドをコマンドウィンドウから削除
  (nvim.ex.autocmd "CmdwinEnter" ":" "g/^..\\?$/d"))

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

; NvimTree <Space>t でトグル
(nvim.set_keymap "n" "<space>t" ":NvimTreeToggle<CR>" { :noremap true })
(nvim.set_keymap "x" "<space>t" ":NvimTreeToggle<CR>" { :noremap true })

; Telescope <Space>f* で開く
(nvim.set_keymap "n" "<Space>ff" "<cmd>lua require('telescope.builtin').find_files()<CR>" { :noremap true })
(nvim.set_keymap "n" "<Space>fg" "<cmd>lua require('telescope.builtin').live_grep()<CR>" { :noremap true })
(nvim.set_keymap "n" "<Space>fb" "<cmd>lua require('telescope.builtin').buffers()<CR>" { :noremap true })
(nvim.set_keymap "n" "<Space>ft" "<cmd>lua require('telescope.builtin').treesitter()<CR>" { :noremap true })
(nvim.set_keymap "n" "<Space>fh" "<cmd>lua require('telescope.builtin').help_tags()<CR>" { :noremap true })

; Quickhl <Space>mでハイライト、<Space>Mでクリア
(nvim.set_keymap "n" "<Space>m" "<Plug>(quickhl-manual-this)"  { :noremap false })
(nvim.set_keymap "x" "<Space>m" "<Plug>(quickhl-manual-this)"  { :noremap false })
(nvim.set_keymap "n" "<Space>M" "<Plug>(quickhl-manual-reset)" { :noremap false })
(nvim.set_keymap "x" "<Space>M" "<Plug>(quickhl-manual-reset)" { :noremap false })

; Glow
(nvim.set_keymap "n" "<Space>p" ":Glow<CR>" { :noremap true })
(nvim.set_keymap "x" "<Space>p" ":Glow<CR>" { :noremap true })

; シンタックスハイライトの有効化
; Neovimはデフォルトで有効化されるはずだがそうならないファイルがある？
(nvim.ex.syntax "on")
; undoファイルを用意
(set nvim.bo.undofile true)
; デフォルトの折りたたみ方式をmarkerに
(set nvim.wo.foldmethod "marker")
; undoの最大値
(set nvim.o.undolevels 1024)
(set nvim.o.undoreload 8192)
; スワップファイルとバックアップファイルを消す
(set nvim.o.swapfile false)
(set nvim.o.backup false)
(set nvim.o.writebackup false)
; デフォルトのインデント設定(最初のバッファに設定すると引き継がれる？)
(set nvim.bo.tabstop 4)
(set nvim.bo.shiftwidth 4)
(set nvim.bo.expandtab false)
(set nvim.bo.autoindent true)
; guiと同じく2^24色使えるように(一応)
(set nvim.o.termguicolors true)
; 行番号をrelativenumberで表示
(set nvim.wo.number true)
(set nvim.wo.relativenumber true)
; コマンド欄(下の2行)を2行に設定
(set nvim.o.cmdheight 2)
; 検索中のハイライトの有効化
(set nvim.o.hls true)
; 不可視文字を可視化
(set nvim.wo.list true)
(set nvim.o.listchars "tab:»-,trail:-,eol:↲,extends:»,precedes:«,nbsp:%")
; Coc推奨設定(わからん)
(set nvim.o.hidden true)
(set nvim.o.updatetime 300)
; カラースキーム
(colorscheme "otynium")
; treesitterの設定
(treesitter.setup {:ensure_installed "maintained" ; 自動で全部インストール
                   :highlight {:enable true ; ハイライトの有効化
                               :disable []}
                   :indent {:enable true}}) ; インデントの有効化(微妙っぽい)
