#!/usr/bin/ruby

require 'yaml'
require 'psych'
require 'optparse'

Version = '0.0.1'.freeze # rubocop:disable Naming/ConstantName

config_file_name = nil
opt = OptionParser.new
opt.on('-c FILE') { |f| config_file_name = f }
opt.parse!(ARGV)

if config_file_name.nil?
  warn(opt.help)
  exit!
end

config = begin
  config_file = File.open(config_file_name, 'r')
  YAML.load_file(config_file)
rescue Psych::SyntaxError => e
  warn("SyntaxError at #{e.line}:#{e.column}.")
  config_file.close
  exit!
rescue Errno::ENOENT, Errno::EPERM
  warn("Cannot read file #{config_file_name}")
  config_file.close
  exit!
end
config_file.close

pp config
