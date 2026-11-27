# A missing file is named on its own, with no other file suggested, even
# when one in the same directory has a similar name.
begin
  require_relative "require_relative_missing_file_messag"
rescue LoadError => error
  puts error.message.sub(__dir__, "DIR")
  p error.path.end_with?("require_relative_missing_file_messag")
end
