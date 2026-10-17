require "shellwords"

puts Shellwords.split("deploy --to staging").length
