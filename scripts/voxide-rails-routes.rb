#!/usr/bin/env ruby
Dir.chdir `git rev-parse --show-toplevel`.chomp

cmd = "rails-routes-cached | fzf -n 1,2 --with-nth 1,2 --multi"
if initial_search = ARGV.first
  require "shellwords"
  cmd += " --query #{initial_search.shellescape}"
end
tsv = `#{cmd}`.lines.map { _1.chomp.split(/\t/) }

source_locations = tsv.map do |method, path, cont, act|
  cont_file = "app/controllers/#{cont}_controller.rb"
  line_no = File.read(cont_file).lines.each_with_index do |line, i|
    if line =~ /def #{Regexp.escape(act)}(\s*$|\()/
      break i + 1
    end
  end

  if line_no
    "#{cont_file}:#{line_no}:#{method} #{path}"
  else
    $stderr.puts "WARNING: no line number found for #{cont}##{act}" if line_no.nil?
  end
end

if source_locations.any?
  puts source_locations
else
  $stderr.puts "No source locations found"
  exit 1
end
