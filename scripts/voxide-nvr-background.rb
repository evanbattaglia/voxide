#!/usr/bin/env ruby
require 'shellwords'
cmd =
  if ARGV.first == '-q'
    "-cc 'copen | cfirst' -q #{ARGV[1].shellescape}"
  else
    args = ARGV.map do |fn|
      "-cc #{"badd #{fn}".shellescape}"
    end.join(" ")
  end
system "nvr #{cmd}"
