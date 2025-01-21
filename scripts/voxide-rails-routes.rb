#!/usr/bin/env ruby
require 'shellwords'
require 'tempfile'

# This script:
# 1. Extracts Rails routes from the Rails app using `rails runner`
#      assumes rails container running in "web" under docker compose!
# 2. Saves the routes in a cache file so we don't have to run `rails runner` every time
# 3. Presents the routes in fzf
# 4. Opens the controller file in the editor at the action definition

Dir.chdir `git rev-parse --show-toplevel`.chomp

# Service name in docker-compose.yml we use to get routes from Rails (rails runner)
DOCKER_COMPOSE_CONTAINER_NAME="web"

def ensure_rails_routes_cache_loaded(cache_file)
  return if File.exist?(cache_file)
  Tempfile.open("voxide-rails-routes") do |f|
    f.write <<~RUBY
      Rails.application.routes.routes.each do |route|
        # Get the HTTP verb and path
        http_method = route.verb
        path = route.path.spec.to_s

        # Get the controller and action (optional)
        requirements = route.defaults
        controller = requirements[:controller]
        action = requirements[:action]

        puts [http_method, path, controller, action].join("\t")
      end
    RUBY
    f.flush
    system "cat #{f.path.shellescape} | docker compose exec -T #{DOCKER_COMPOSE_CONTAINER_NAME} rails runner - 2> >(grep -v ^Running.via.Spring >&2) > #{cache_file.shellescape}"
  end
end

git_repo_dir = `git rev-parse --show-toplevel`.chomp
cache_file = File.join(git_repo_dir, ".voxide-rails-routes")
ensure_rails_routes_cache_loaded(cache_file)
cmd = "fzf -n 1,2 --with-nth 1,2 --multi < #{cache_file.shellescape}"
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
