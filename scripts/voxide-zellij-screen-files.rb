#!/usr/bin/env ruby
require 'tempfile'
require 'shellwords'

def git_repo_root
  @git_repo_root ||= `git rev-parse --show-toplevel 2>/dev/null`.chomp
end

def add_to_set_if_exists(set, f, extra_re)
  f = f.gsub(/^~\//, ENV['HOME']+"/").strip
  return unless extra_re.nil? || extra_re.match?(f)
  return if f.strip.empty?

  # TODO: really should find a way to utilize the transforms, like voxide should apply the transforms to find files that exist but then show in the filter (FZF) the untransformed filename
  if File.exist?(f) || File.exist?("#{git_repo_root}/#{f}")
    set << f
  end
end

Tempfile.open('zelij-screen-files') do |f|
  system "zellij ac dump-screen #{f.path.shellescape}"
  screen = File.read(f.path)
  regex = %r{[~a-zA-Z0-9_./-]+/[a-zA-Z0-9_./-]+}
  extra_re = ARGV[0]&.then { Regexp.new(_1) }
  set = Set.new
  screen.lines.each do |l|
    add_to_set_if_exists(set, l, extra_re)
    l.scan(regex) do |match|
      add_to_set_if_exists(set, match, extra_re)
    end
  end
  puts set.to_a.join("\n")
end

