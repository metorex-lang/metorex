require_relative "loaded_file_runner"

puts LoadedFileRunner.load_and_report(File.join(__dir__, "loaded_file_raises.rb"))

begin
  load File.join(__dir__, "loaded_file_raises.rb")
rescue NameError => error
  puts error.backtrace.first(2).map { |line| line.sub(/\A.*\//, "") }
end
