#!/usr/bin/ruby

require 'rbconfig'
require 'fileutils'
require 'optparse'
require 'erb'

$verbose = true

def replace_env_vars(path, dict)
  base = path
  m = path.match(/\$[a-zA-Z0-9_]+/)
  while !m.nil? && !m[0].empty?
    base.gsub!(m[0], dict[m[0][1..]])
    path = m.post_match
    m = path.match(/\$[a-zA-Z0-9_]+/)
  end
  base
end

def path_expand(path)
  File.expand_path(replace_env_vars(path, ENV))
end

def test(path)
  File.exist?(path_expand(path))
end

def enumerate_dirs_likes_grow_up(path)
  enumerated = [path]
  until path == '/'
    path = File.dirname(path)
    enumerated.push(path)
  end
  enumerated.reverse
end

# パスは正規化すること！
def cp_rec(src, dest)
  enumerate_dirs_likes_grow_up(File.dirname(dest)).each do |dir|
    Dir.mkdir(dir) unless Dir.exist?(dir)
  end
  FileUtils.cp_r(src, dest)
end

# OS判別
OS = begin
  host_os = RbConfig::CONFIG['host_os']
  case host_os
  when /mswin|msys|mingw|cygwin|bccwin|wince|emc/
    :windows
  when /darwin|mac os/
    :macos
  when /linux/
    :linux
  when /solaris|bsd/
    :unix
  else
    :unknown
  end
end
OS.freeze

def enumerate_cp_pairs(pkg, dest_dir)
  src = "#{__dir__}/pkgs/#{pkg}"
  Dir.glob('**/*', base: src).map do |file|
    { src: File.expand_path("#{__dir__}/pkgs/#{pkg}/#{file}"), dest: path_expand("#{dest_dir}/#{file}") }
  end
end

def filecp_merge(pkg, dest_dir)
  enumerate_cp_pairs(pkg, dest_dir).each do |pair|
    cp_rec(pair[:src], pair[:dest])
  end
end

def filecp_choose(pkg, choose, dest_dir)
  src = "#{__dir__}/pkgs/#{pkg}/#{choose}"
  dest = path_expand(dest_dir)
  cp_rec(src, dest)
end

def filecp_clean(pkg, dest_dir)
  src = "#{__dir__}/pkgs/#{pkg}"
  dest = path_expand(dest_dir)
  FileUtils.rm_rf(dest) if File.exist?(dest)
  cp_rec(src, dest)
end

def file_post_erb(erb_target_file_name, erb_hash)
  template = ERB.new(File.read(erb_target_file_name))
  File.write(erb_target_file_name, template.result_with_hash(erb_hash))
end

def filecp_install(pkg, cfg)
  # TODO: Windows-compatible
  if cfg.key?(:merge) && cfg[:merge]
    filecp_merge(pkg, cfg[OS])
  elsif cfg.key?(:choose) && cfg[:choose]
    filecp_choose(pkg, cfg[:choose], cfg[OS])
  else
    filecp_clean(pkg, cfg[OS])
  end
  return unless cfg.key?(:erb)

  erb_target_file_name = "#{cfg[OS]}/#{cfg[:erb]}"
  file_post_erb(erb_target_file_name, cfg[:erb_hash])
end

def src_has_new_file?(pkg, dest)
  enumerate_cp_pairs(pkg, dest).each do |pair|
    next unless File.file?(pair[:src]) || File.file?(pair[:dest])
    return true unless File.exist?(pair[:dest])

    src_mtime = File::Stat.new(pair[:src])
    dest_mtime = File::Stat.new(pair[:dest])
    return true if src_mtime > dest_mtime
  end
  false
end

def make_filecp_cond(pkg, cfg)
  lambda do
    return false if cfg[OS].nil?

    src = cfg.key?(:choose) && cfg[:choose] ? "#{__dir__}/pkgs/#{pkg}/#{cfg[:choose]}" : "#{__dir__}/pkgs/#{pkg}"
    dest = path_expand(cfg[OS])
    if cfg.key?(:choose)
      src_stat = File::Stat.new(src)
      dest_stat = File.file?(dest) ? File::Stat.new(dest) : File::Stat.new("#{dest}/#{cfg[:choose]}")
      return src_stat.mtime > dest_stat.mtime
    end
    return src_has_new_file?(pkg, dest)
  end
end

# 汎用のインストールクラス
class InstallUnit
  def initialize(name)
    @name = name
    @cond = @hook = nil
  end

  def cond(&cond_blk)
    @cond = cond_blk
  end

  def hook(&hook_blk)
    @hook = hook_blk
  end

  def execute
    raise 'at least cond or hook are undefined' if @cond.nil? || @hook.nil?

    if @cond.call
      puts "✅ #{@name}"
      @hook.call
    elsif $verbose
      puts "➡ #{@name}"
    end
  end
end

# ディレクトリコピー用
class FileCopyUnit < InstallUnit
  def initialize(name)
    super(name)
    @cfg = {}
  end

  def setup_task(pkg, cfg)
    @cond = make_filecp_cond(pkg, cfg)
    @hook = lambda do
      return false if cfg[OS].nil?

      filecp_install(pkg, cfg)
    end
  end

  def macos(path)
    @cfg.merge!({ macos: path })
    setup_task(@name, @cfg)
  end

  def linux(path)
    @cfg.merge!({ linux: path })
    setup_task(@name, @cfg)
  end

  def choose(path)
    @cfg.merge!({ choose: path })
    setup_task(@name, @cfg)
  end

  def merge(flag)
    @cfg.merge!({ merge: flag })
    setup_task(@name, @cfg)
  end

  def erb(path, hash)
    @cfg.merge!({ erb: path, erb_hash: hash })
    setup_task(@name, @cfg)
  end
end

# for cargo tools
class RustToolInstallUnit < InstallUnit
  def initialize(name)
    super("rust #{name}")
  end

  def bin(bin_name)
    @cond = -> do !test("$HOME/.cargo/bin/#{bin_name}") end
    @hook = lambda do
      cargo = path_expand('$HOME/.cargo/bin/cargo')
      system("#{cargo} install #{@name}")
    end
  end
end

# インストールユニットの集合（主にDSLの都合）
class InstallUnits
  def initialize
    @tasks = []
  end

  def unwrap_tasks
    @tasks
  end

  def merge!(units)
    @tasks += units.unwrap_tasks
    self
  end

  def merge(units)
    new_units = InstallUnits.new
    new_units.merge!(self)
    new_units.merge!(units)
  end

  def general_task(name, &blk)
    unit = InstallUnit.new(name)
    unit.instance_eval(&blk)
    @tasks << unit
  end

  def file_copy(name, &blk)
    unit = FileCopyUnit.new(name)
    unit.instance_eval(&blk)
    @tasks << unit
  end

  def rust_tool(name, bin)
    unit = RustToolInstallUnit.new(name)
    unit.bin(bin)
    @tasks << unit
  end

  def install
    @tasks.each(&:execute)
  end
end

def tasks(_name, &blk)
  units = InstallUnits.new
  units.instance_eval(&blk)
  units
end

if $PROGRAM_NAME == __FILE__

  IS_ROOT = Process.euid.zero? and Process.uid.zero?
  IS_ROOT.freeze

  # ~/.dotfileにデフォルトのターゲットを保存しておく
  dotfile_path = path_expand('$HOME/.dotfile')

  target_str = File.open(dotfile_path).read if File.readable?(dotfile_path)

  opt = OptionParser.new
  opt.on('-t TARGET', '--target TARGET') do |t|
    target_str = t
  end
  $verbose = false
  opt.on('-v', '--verbose') { $verbose = true }
  opt.parse(ARGV)

  target = case target_str
           when /cookpad|ckpd/
             :ckpd
           when /private|priv/
             :priv
           end

  if target.nil?
    warn(opt.help)
    exit!
  end

  File.open(dotfile_path, 'w+').write(target) if File.world_writable?(dotfile_path) || !File.exist?(dotfile_path)

  ############################################################################
  ######################## ↓↓↓ USER CONFIGURATION ↓↓↓ ########################
  ############################################################################

  pre_file_copy_tasks = tasks 'pre file copy' do
    general_task 'set XDG_CONFIG_HOME' do
      cond { ENV['XDG_CONFIG_HOME'].nil? }
      hook { ENV['XDG_CONFIG_HOME'] = path_expand('$HOME/.config') }
    end

    general_task 'rustup' do
      cond { !test('$HOME/.cargo/bin/rustup') && !test('$HOME/.rustup') }
      hook { system("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh") }
    end
  end

  post_file_copy_tasks = tasks 'post file copy' do
    general_task 'fisher' do
      cond { !test('$HOME/.config/fish/functions/fisher.fish') }
      hook { system('fish -c "curl -sL https://git.io/fisher | source && fisher install jorgebucaran/fisher"') }
    end
  end

  file_copy_tasks_common = tasks 'file copy common' do
    file_copy 'alacritty' do
      macos '$HOME/.config/alacritty'
      linux '$XDG_CONFIG_HOME/alacritty'
      erb('alacritty.yml', {
            alacritty_font_size: 13,
            alacritty_opacity: OS == :macos ? 0.9 : 0.7
          })
    end
    file_copy 'fish' do
      macos '$HOME/.config/fish'
      linux '$XDG_CONFIG_HOME/fish'
      merge true
    end
    file_copy 'gpg' do
      macos '$HOME/.gnupg/gpg.conf'
      linux '$HOME/.gnupg/gpg.conf'
      choose 'gpg.conf'
    end
    file_copy 'latexmk' do
      macos '$HOME/.latexmkrc'
      linux '$HOME/.latexmkrc'
      choose 'latexmkrc'
    end
    file_copy 'lazygit' do
      macos '$HOME/.config/jesseduffield/lazygit'
      linux '$HOME/.config/jesseduffield/lazygit'
    end
    file_copy 'neovim' do
      macos '$HOME/.config/nvim'
      linux '$HOME/.config/nvim'
    end
    file_copy 'npm' do
      macos '$HOME/.npmrc'
      linux '$HOME/.npmrc'
      choose 'npmrc'
    end
    file_copy 'paru' do
      macos '$HOME/.config/paru'
      linux '$HOME/.config/paru'
    end
    file_copy 'sway' do
      macos '$HOME/.config/sway'
      linux '$XDG_CONFIG_HOME/sway'
    end
    file_copy 'tig' do
      macos '$HOME/.tigrc'
      linux '$HOME/.tigrc'
      choose 'tigrc'
    end
    file_copy 'vscode' do
      macos '$HOME/.config/Code/User'
      linux '$XDG_CONFIG_HOME/Code/User'
      merge true
    end

    file_copy 'waybar' do
      linux '$XDG_CONFIG_HOME/waybar'
    end
  end

  file_copy_tasks_priv_gitconfig = tasks 'file copy private gitconfig' do
    file_copy 'git' do
      macos '$HOME/.gitconfig'
      linux '$HOME/.gitconfig'
      choose 'gitconfig'
    end
    file_copy 'ssh' do
      macos '$HOME/.ssh'
      linux '$HOME/.ssh'
      merge true
    end
  end

  file_copy_tasks_ckpd_gitconfig = tasks 'file copy cookpad gitconfig' do
    file_copy 'git.priv' do
      macos '$HOME/.gitconfig.priv'
      linux '$HOME/.gitconfig.priv'
      choose 'gitconfig'
    end
    file_copy 'git.ckpd' do
      macos '$HOME/.gitconfig.ckpd'
      linux '$HOME/.gitconfig.ckpd'
      choose 'gitconfig'
    end
    file_copy 'ssh' do
      macos '$HOME/.ssh/config.priv'
      linux '$HOME/.ssh/config.priv'
      choose 'config'
    end
  end

  file_copy_tasks_priv_root = tasks 'file copy private in root' do
    file_copy 'autofs' do
      macos nil
      linux '/etc/autofs'
      merge true
    end
    file_copy 'fcitx5' do
      macos nil
      linux '/usr/share/fcitx5'
      merge true
    end
    file_copy 'iptables' do
      macos nil
      linux '/etc/iptables'
      merge true
    end
    file_copy 'networkmanager' do
      macos nil
      linux '/etc/NetworkManager'
      merge true
    end
    file_copy 'sshd' do
      macos nil
      linux '/etc/ssh'
      merge true
    end
    file_copy 'systemd' do
      macos nil
      linux '/etc/systemd/'
      merge true
    end
    file_copy 'udev' do
      macos nil
      linux '/etc/udev/rules.d'
      merge true
    end
    # TODO: generate wallpaper
    file_copy 'wallpaper' do
      linux '/opt/wallpaper'
    end
  end

  cargo_tools = tasks 'install tools by cargo' do
    rust_tool('bandwhich', 'bandwhich')
    rust_tool('bat', 'bat')
    rust_tool('bingrep', 'bingrep')
    rust_tool('cargo-edit', 'cargo-add')
    rust_tool('cargo-fuzz', 'cargo-fuzz')
    rust_tool('cross', 'cross')
    rust_tool('csview', 'csview')
    rust_tool('diskonaut', 'diskonaut')
    rust_tool('fd-find', 'fd')
    rust_tool('git-delta', 'delta')
    rust_tool('git-interactive-rebase-tool', 'interactive-rebase-tool')
    rust_tool('gping', 'gping')
    rust_tool('ht', 'ht')
    rust_tool('hyperfine', 'hyperfine')
    rust_tool('lsd', 'lsd')
    rust_tool('onefetch', 'onefetch')
    rust_tool('pastel', 'pastel')
    rust_tool('procs', 'procs')
    rust_tool('ripgrep', 'rg')
    rust_tool('silicon', 'silicon')
    rust_tool('skim', 'sk')
    rust_tool('tokei', 'tokei')
    rust_tool('topgrade', 'topgrade')
    rust_tool('xsv', 'xsv')
  end

  targets = []
  if IS_ROOT
    case target
    when :priv
      targets = file_copy_tasks_priv_root
    when :ckpd
      return
    end

  else
    targets = pre_file_copy_tasks.merge(cargo_tools)
    case target
    when :priv
      targets = targets.merge(file_copy_tasks_common).merge(file_copy_tasks_priv_gitconfig)

    when :ckpd
      targets = targets.merge(file_copy_tasks_common).merge(file_copy_tasks_ckpd_gitconfig)
    end
    targets.merge(post_file_copy_tasks)
  end

  targets.install
end
