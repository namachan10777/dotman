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

def filecp_merge(pkg, dest_dir)
  src = "#{__dir__}/pkgs/#{pkg}"
  Dir.glob('**/*', base: src).each do |file|
    cp_rec(File.expand_path("#{__dir__}/pkgs/#{pkg}/#{file}"), path_expand("#{dest_dir}/#{file}"))
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
  if cfg.key?(:erb)
    erb_target_file_name = "#{cfg[@os]}/#{cfg[:erb]}"
    file_post_erb(erb_target_file_name, cfg[:erb_hash])
  end
  puts "✅ #{pkg}"
end

def filecp_to_install_task(pkg, cfg)
  {
    cond: lambda do
      src = cfg.key?(:choose) && cfg[:choose] ? "#{__dir__}/pkgs/#{pkg}/#{cfg[:choose]}" : "#{__dir__}/pkgs/#{pkg}"
      dest = path_expand(cfg[@os])
      src_stat = File::Stat.new(src)
      dest_stat = File::Stat.new(dest)
      return src_stat.mtime > dest_stat.mtime
    end,
    hook: lambda do
      filecp_install(pkg, cfg)
    end
  }
end

if $PROGRAM_NAME == __FILE__

  opt = OptionParser.new
  target = nil
  opt.on('--target TARGET') do |t|
    case t
    when /cookpad|ckpd/
      target = :ckpd
    when /private|priv/
      target = :priv
    end
  end
  opt.parse(ARGV)

  if target.nil?
    warn(opt.help)
    exit!
  end

  set_xdg_config_home = {
    cond: -> do return true end,
    hook: lambda do
      ENV['XDG_CONFIG_HOME'] = path_expand('$HOME/.config')
    end
  }

  rustup_install = {
    cond: lambda do
      return !test('$HOME/.cargo/bin/rustup')
    end,
    hook: lambda do
      system("sh -c \"curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh\"")
      puts '✅ rustup'
    end
  }

  # TODO: root required installation
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

  tasks = []
  case target
  when :priv
    tasks = [set_xdg_config_home, rustup_install]
    filecp = filecp_common.merge(filecp_priv_gitconfig)
    tasks += filecp.map do |pkg, cfg|
      filecp_to_install_task(pkg, cfg)
    end
  when :ckpd
    tasks = [set_xdg_config_home, rustup_install]
    filecp = filecp_common.merge(filecp_ckpd_gitconfig)
    tasks += filecp.map do |pkg, cfg|
      filecp_to_install_task(pkg, cfg)
    end
  end

  tasks.each do |task|
    task[:hook].call if task[:cond].call
  end
end
