require "pathname"

ran = []
answer = Dir.chdir(Pathname.new("/")) do
  ran << Dir.pwd
  :from_block
end
p ran, answer
