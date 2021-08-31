#!/usr/bin/ruby

require 'rbconfig'
require 'fileutils'
require 'optparse'
require 'erb'

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
    break if Dir.exist?(dir)

    Dir.mkdir(dir)
  end
  FileUtils.cp_r(src, dest)
end

# OS判別
@os = begin
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
    filecp_merge(pkg, cfg[@os])
  elsif cfg.key?(:choose) && cfg[:choose]
    filecp_choose(pkg, cfg[:choose], cfg[@os])
  else
    filecp_clean(pkg, cfg[@os])
  end
  return unless cfg.key?(:erb)

  erb_target_file_name = "#{cfg[@os]}/#{cfg[:erb]}"
  file_post_erb(erb_target_file_name, cfg[:erb_hash])
end

def src_has_new_file?(pkg, dest)
  enumerate_cp_pairs(pkg, dest).each do |pair|
    src_mtime = File::Stat.new(pair[:src])
    dest_mtime = File::Stat.new(pair[:dest])
    return true if src_mtime > dest_mtime
  end
  false
end

def make_filecp_cond(pkg, cfg)
  lambda do
    return false if cfg[@os].nil?

    src = cfg.key?(:choose) && cfg[:choose] ? "#{__dir__}/pkgs/#{pkg}/#{cfg[:choose]}" : "#{__dir__}/pkgs/#{pkg}"
    dest = path_expand(cfg[@os])
    if cfg.key?(:choose)
      src_stat = File::Stat.new(src)
      dest_stat = File.file?(dest) ? File::Stat.new(dest) : File::Stat.new("#{dest}/#{cfg[:choose]}")
      return src_stat.mtime > dest_stat.mtime
    end
    return src_has_new_file?(pkg, dest)
  end
end

def filecp_to_install_task(pkg, cfg)
  {
    name: pkg,
    cond: make_filecp_cond(pkg, cfg),
    hook: lambda do
      return false if cfg[@os].nil?

      filecp_install(pkg, cfg)
    end
  }
end

if $PROGRAM_NAME == __FILE__

  @is_root = Process.euid.zero? and Process.uid.zero?

  # ~/.dotfileにデフォルトのターゲットを保存しておく
  dotfile_path = path_expand('$HOME/.dotfile')

  target_str = File.open(dotfile_path).read if File.readable?(dotfile_path)

  opt = OptionParser.new
  opt.on('-t TARGET', '--target TARGET') do |t|
    target_str = t
  end
  @verbose = false
  opt.on('-v', '--verbose') { @verbose = true }
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

  set_xdg_config_home = {
    name: 'set $XDG_CONFIG_HOME',
    cond: -> do ENV['XDG_CONFIG_HOME'].nil? end,
    hook: lambda do
      ENV['XDG_CONFIG_HOME'] = path_expand('$HOME/.config')
    end
  }

  rustup_install = {
    name: 'rustup install',
    cond: lambda do
      return !test('$HOME/.cargo/bin/rustup')
    end,
    hook: lambda do
      system("curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh")
    end
  }

  fisher_install = {
    name: 'fisher install',
    cond: lambda do
      return !test('$HOME/.config/fish/functions/fisher.fish')
    end,
    hook: lambda do
      system('fish -c "curl -sL https://git.io/fisher | source && fisher install jorgebucaran/fisher"')
    end
  }

  filecp_common = {
    'alacritty' => {
      macos: '$HOME/.config/alacritty',
      linux: '$XDG_CONFIG_HOME/alacritty',
      erb: 'alacritty.yml',
      erb_hash: {
        alacritty_font_size: 13
      }
    },
    'fish' => {
      macos: '$HOME/.config/fish',
      linux: '$XDG_CONFIG_HOME/fish',
      merge: true
    },
    'gpg' => {
      macos: '$HOME/.gnupg/gpg.conf',
      linux: '$HOME/.gnupg/gpg.conf',
      choose: 'gpg.conf'
    },
    'latexmk' => {
      macos: '$HOME/.latexmkrc',
      linux: '$HOME/.latexmkrc',
      choose: 'latexmkrc'
    },
    'lazygit' => {
      macos: '$HOME/.config/jesseduffield/lazygit',
      linux: '$HOME/.config/jesseduffield/lazygit'
    },
    'neovim' => {
      macos: '$HOME/.config/nvim',
      linux: '$HOME/.config/nvim'
    },
    'npm' => {
      macos: '$HOME/.npmrc',
      linux: '$HOME/.npmrc',
      choose: 'npmrc'
    },
    'paru' => {
      macos: '$HOME/.config/paru',
      linux: '$HOME/.config/paru'
    },
    'sway' => {
      macos: '$HOME/.config/sway',
      linux: '$XDG_CONFIG_HOME/sway'
    },
    'tig' => {
      macos: '$HOME/.tigrc',
      linux: '$HOME/.tigrc',
      choose: 'tigrc'
    },
    'vscode' => {
      macos: '$HOME/.config/Code/User',
      linux: '$XDG_CONFIG_HOME/Code/User',
      merge: true
    }
  }

  filecp_priv_gitconfig = {
    'git' => {
      macos: '$HOME/.gitconfig',
      linux: '$HOME/.gitconfig',
      choose: 'gitconfig'
    },
    'ssh' => {
      macos: '$HOME/.ssh',
      linux: '$HOME/.ssh',
      merge: true
    }
  }

  filecp_ckpd_gitconfig = {
    'git.priv' => {
      macos: '$HOME/.gitconfig.priv',
      linux: '$HOME/.gitconfig.priv',
      choose: 'gitconfig'
    },
    'git.ckpd' => {
      macos: '$HOME/.gitconfig.ckpd',
      linux: '$HOME/.gitconfig.ckpd',
      choose: 'gitconfig'
    },
    'ssh' => {
      macos: '$HOME/.ssh/config.priv',
      linux: '$HOME/.ssh/config.priv',
      choose: 'config'
    }
  }

  filecp_priv_root = {
    'autofs' => {
      macos: nil,
      linux: '/etc/autofs',
      merge: true
    },
    'fcitx5' => {
      macos: nil,
      linux: '/usr/share/fcitx5',
      merge: true
    },
    'iptables' => {
      macos: nil,
      linux: '/etc/iptables',
      merge: true
    },
    'networkmanager' => {
      macos: nil,
      linux: '/etc/NetworkManager',
      merge: true
    },
    'sshd' => {
      macos: nil,
      linux: '/etc/ssh',
      merge: true
    },
    'systemd' => {
      macos: nil,
      linux: '/etc/systemd/',
      merge: true
    },
    'udev' => {
      macos: nil,
      linux: '/etc/udev/rules.d',
      merge: true
    }
  }

  tasks = []
  if @is_root
    case target
    when :priv
      tasks = filecp_priv_root.map do |pkg, cfg|
        filecp_to_install_task(pkg, cfg)
      end
    when :ckpd
      tasks = []
    end

  else
    tasks = [set_xdg_config_home, rustup_install]
    case target
    when :priv
      filecp = filecp_common.merge(filecp_priv_gitconfig)
      tasks += filecp.map do |pkg, cfg|
        filecp_to_install_task(pkg, cfg)
      end
    when :ckpd
      filecp = filecp_common.merge(filecp_ckpd_gitconfig)
      tasks += filecp.map do |pkg, cfg|
        filecp_to_install_task(pkg, cfg)
      end
    end
    tasks.push fisher_install
  end

  tasks.each do |task|
    if task[:cond].call
      puts "✅ #{task[:name]}"
      task[:hook].call
    elsif @verbose
      puts "➡ #{task[:name]}"
    end
  end
end
