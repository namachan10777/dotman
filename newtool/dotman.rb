#!/usr/bin/ruby

require 'rbconfig'
require 'fileutils'

def path_expand(path)
  File.expand_path(`echo -n #{path}`)
end

def test(path)
  # TODO: エラー処理
  File.exist?(path_expand(path))
end

# OS判別
os = begin
  host_os = RbConfig::CONFIG['host_os']
  case host_os
  when /mswin|msys|mingw|cygwin|bccwin|wince|emc/
    :windows
  when /darwin|mac os/
    :macosx
  when /linux/
    :linux
  when /solaris|bsd/
    :unix
  else
    :unknown
  end
end

unless test('$HOME/.cargo/bin/rustup')
  system("sh -c \"curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh\"")
end
puts '✅ rustup'

ENV['XDG_CONFIG_HOME'] = path_expand('$HOME/.config')

# TODO: root required installation
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

filecp.each do |pkg, dest_dict|
  if dest_dict.key?(:merge) && dest_dict[:merge]
    puts "✘ skip. merge required #{pkg}"
  elsif dest_dict.key?(:choose) && dest_dict[:choose]
    src = "#{__dir__}/pkgs/#{pkg}/#{dest_dict[:choose]}"
    dest = path_expand(dest_dict[os])
    FileUtils.cp(src, dest)
    puts "✅ #{pkg}"
  else
    src = "#{__dir__}/pkgs/#{pkg}"
    dest = path_expand(dest_dict[os])
    FileUtils.rm_rf(dest) if File.exists?(dest)
    FileUtils.cp_r(src, dest)
    puts "✅ #{pkg}"
  end
end
