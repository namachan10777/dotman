#!/usr/bin/ruby

require 'yaml'
require 'optparse'

Version = '0.0.1'.freeze # rubocop:disable Naming/ConstantName

config_file = nil
opt = OptionParser.new
opt.on('-c FILE') { |f| config_file = f }
opt.parse!(ARGV)

print(opt.help) if config_file.nil?
