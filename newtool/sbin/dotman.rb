#!/usr/bin/ruby

require "yaml"
require "optparse"

Version = "0.0.1"

config_file = nil
opt = OptionParser.new
opt.on('-c FILE1') {|f| config_file = f}
opt.parse!(ARGV)

if config_file.nil? then
    print(opt.help)
end