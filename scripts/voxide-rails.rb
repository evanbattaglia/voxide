#!/usr/bin/env ruby

# TODO this is not very portable really.
# probably need support for submodes


DIRS = {
  m: 'app/models/',
  c: 'app/controllers/',
  a: 'app/',
  u: 'ui/',# TODO this is specific to our project
  us: 'ui/shared/', # TODO this is specific to our project
}
submode = ARGV[0]
if submode == 'r'
  # TODO broken because different TSV columns for fzf
  system 'rails-routes-cached'
elsif (dir = DIRS[submode.to_sym])
  ARGV[0] = dir
  ARGV[1] ||= ''
  require_relative 'voxide-file-acronym.rb'
else
  raise "unknown submode, expected one of #{DIRS.keys.join(', ')} or 'r'"
end
