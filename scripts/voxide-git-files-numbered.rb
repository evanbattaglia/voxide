#!/usr/bin/env ruby

require 'io/console' # STDIN.getch
require 'shellwords'

class String
def black;          "\e[30m#{self}\e[0m" end
def red;            "\e[31m#{self}\e[0m" end
def green;          "\e[32m#{self}\e[0m" end
def brown;          "\e[33m#{self}\e[0m" end
def blue;           "\e[34m#{self}\e[0m" end
def magenta;        "\e[35m#{self}\e[0m" end
def cyan;           "\e[36m#{self}\e[0m" end
def gray;           "\e[37m#{self}\e[0m" end

def bg_black;       "\e[40m#{self}\e[0m" end
def bg_red;         "\e[41m#{self}\e[0m" end
def bg_green;       "\e[42m#{self}\e[0m" end
def bg_brown;       "\e[43m#{self}\e[0m" end
def bg_blue;        "\e[44m#{self}\e[0m" end
def bg_magenta;     "\e[45m#{self}\e[0m" end
def bg_cyan;        "\e[46m#{self}\e[0m" end
def bg_gray;        "\e[47m#{self}\e[0m" end

def bold;           "\e[1m#{self}\e[22m" end
def italic;         "\e[3m#{self}\e[23m" end
def underline;      "\e[4m#{self}\e[24m" end
def blink;          "\e[5m#{self}\e[25m" end
def reverse_color;  "\e[7m#{self}\e[27m" end
end


SCRIPT_DIR=File.dirname(__FILE__)
f = ARGV[0] or raise "Usage: voxide-git-files-numbered.rb <outputfile>"
# hmm, ruby calling ruby, inefficient...
output = `#{SCRIPT_DIR.shellescape}/voxide-git-files #{ARGV[1..-1].map(&:shellescape).join(' ')}`
exit 1 unless $?.success?
lines = output.split("\n")

lookup = ('1'..'9').to_a + ('a'..'z').to_a
rev_lookup = lookup.each_with_index.map{|val, i| [val, i]}.to_h

output.split("\n").zip(lookup).each do |line, letter|
  pretty_filename = line.split("\t")[1]
  puts "#{pretty_filename} #{" #{letter}\u00A0".bg_cyan.black.bold.italic}"
end

print "? "
ch = $stdin.getch.tap { |char| exit(1) if char == "\u0003" }
n = rev_lookup[ch]
# remove color codes and fifrst mpart "com", "loc", etc.) .... yeah, we should really bring vgfiles into this script and clean this up...
line = lines[n.to_i]
exit 1 unless line
file = line.split("\t")[0]
File.open(f, 'a') { |f| f.puts file }
puts
