#!/usr/bin/env ruby
require 'shellwords'

def path_fragment_acronym(frag)
  frag.gsub(/([a-z])([A-Z])/, '\1 \2').gsub(/[\/_.-]/, ' \0').split.map do |word|
    if word[0] == '-' || word[0] == '_'
      word[1]
    else
      word[0]
    end
  end.join.downcase
end

def score_match(acronym, needle)
  # example: app/models/context_external_tool.rb => AMcetR
  if needle == acronym
    # "AMcetR"
    return 100
  elsif acronym.match(/^#{Regexp.escape(needle)}[A-Z]$/)
    # "AMcet"
    return 90
  elsif acronym.match(/([A-Z]|\/)#{Regexp.escape(needle)}$/)
    # "McetR", "cetR"
    return 80
  elsif acronym.match(/([A-Z]|\/)#{Regexp.escape(needle)}[A-Z]$/)
    # "Mcet", "cet"
    return 70
  elsif acronym.gsub(/[A-Z]*$/, '').downcase.end_with?(needle)
    # mcet
    return 60 - acronym.length
    # TODO: mer should match MerR over XmeR (my/ex_rb.rb over x/my_ex.rb )?
    # but still not sure this is exactly right....
  elsif acronym.downcase.end_with?(needle)
    return 50 - acronym.length
    # cetr, mcetr, amcetr
    # TODO cetr, mcetr
    # but etr should be lower. etr would match EtR better than cetR
  elsif acronym.include?(needle)
    # "Mce", "ce"
    return 40 - acronym.length
  elsif acronym.downcase.include?(needle.downcase)
    # "Mce", "ce"
    return 30 - acronym.length
  end
end

def acronymize_string(s)
  *dirs, file = s.chomp.split("/")
  file, *extensions = file.split(".")

  dirs.map { |d| path_fragment_acronym(d) }.join.upcase +
    path_fragment_acronym(file) +
    extensions.map { |e| path_fragment_acronym(e) }.join.upcase
end

class TopResultFinder
  attr_reader :matches, :max_score, :needle

  def initialize(needle)
    @max_score = 0
    @matches = []
    @needle = needle
  end

  def process(match, item)
    score = score_match(match, needle)

    if score && score > @max_score
      @max_score = score
      @matches = [[match, item]]
    elsif score == @max_score
      @matches << [match, item]
    end
  end

  def finish
    matches.each do |(m, i)|
      puts [m, i].join("\t")
    end
  end
end

class Printer
  def process(match, item) = puts([match, item].join("\t"))
  def finish = nil
end

pre_search, search =
  case ARGV.count
  when 2
    [ARGV[0], ARGV[1]]
  when 0..1
    [nil, ARGV[0]]
  else
    raise "Usage: #{$0} [pre_search] [search]"
  end

files_raw =
  if pre_search
    if File.directory?(pre_search)
      `fd -t f . #{pre_search.shellescape}`
    else
      `fd -t f -p #{pre_search.shellescape}`
    end
  else
    `fd -t f .`
  end

processor =
  if search
    TopResultFinder.new(search)
  else
    Printer.new
  end

files_raw.split("\n").each do |s|
  s = s.chomp
  acronym = acronymize_string(s)
  processor.process(acronym, s)
end

processor.finish
