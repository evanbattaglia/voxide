#!/usr/bin/env bash
# Usage:
# v A MYSTRING
# v A mycommit MYSTRING
if [[ -z "$2" ]]; then
  commit=HEAD
else
  commit=${1:-HEAD}
  shift
fi
git show "$commit" | diff2additionsqf.rb "$@"
