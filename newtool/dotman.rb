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

filecp = {
  'alacritty' => {
    macos: '$HOME/.config/alacritty',
    linux: '$XDG_CONFIG_HOME/alacritty'
  }
}

filecp.each do |pkg, dest_dict|
  src = "#{__dir__}/pkgs/#{pkg}"
  dest = path_expand(dest_dict[os])
  FileUtils.cp_r(src, dest)
  puts "✅ #{pkg}"
end
