#!/usr/bin/ruby

require 'rbconfig'
require 'fileutils'

# TODO: select task sets by option

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

# パスは正規化すること！
def cp_rec(src, dest)
  # FIXME: ad-hoc
  system("mkdir -p #{File.dirname(dest)}")
  FileUtils.cp_r(src, dest)
end

# OS判別
os = begin
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

if $PROGRAM_NAME == __FILE__

  unless test('$HOME/.cargo/bin/rustup')
    system("sh -c \"curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh\"")
  end
  puts '✅ rustup'

  ENV['XDG_CONFIG_HOME'] = path_expand('$HOME/.config')

  # TODO: root required installation
  # TODO: hooks for alacritty
  filecp = {
    'alacritty' => {
      macos: '$HOME/.config/alacritty',
      linux: '$XDG_CONFIG_HOME/alacritty'
    },
    'fish' => {
      macos: '$HOME/.config/fish',
      linux: '$XDG_CONFIG_HOME/fish'
    },
    'git' => {
      macos: '$HOME/.config/.gitconfig',
      linux: '$HOME/.config/.gitconfig',
      choose: 'gitconfig'
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
      macos: '$HOME/.config/neovim',
      linux: '$HOME/.config/neovim'
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
    'ssh' => {
      macos: '$HOME/.ssh',
      linux: '$HOME/.ssh',
      merge: true
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

  # TODO: Windows-compatible
  # TODO: mkdir -p
  # TODO: remove choose
  filecp.each do |pkg, dest_dict|
    if dest_dict.key?(:merge) && dest_dict[:merge]
      src = "#{__dir__}/pkgs/#{pkg}"
      Dir.glob('**/*', base: src).each do |file|
        cp_rec(File.expand_path("#{__dir__}/pkgs/#{pkg}/#{file}"), path_expand("#{dest_dict[os]}/#{file}"))
      end
    elsif dest_dict.key?(:choose) && dest_dict[:choose]
      src = "#{__dir__}/pkgs/#{pkg}/#{dest_dict[:choose]}"
      dest = path_expand(dest_dict[os])
      cp_rec(src, dest)
    else
      src = "#{__dir__}/pkgs/#{pkg}"
      dest = path_expand(dest_dict[os])
      FileUtils.rm_rf(dest) if File.exist?(dest)
      cp_rec(src, dest)
    end
    puts "✅ #{pkg}"
  end
end
