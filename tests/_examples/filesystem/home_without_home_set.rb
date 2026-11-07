require "etc"

ENV["HOME"] = ""
p Dir.home
ENV.delete("HOME")
p Dir.home == Dir.home(Etc.getlogin || ENV["USER"])
p File.expand_path("~") == Dir.home
