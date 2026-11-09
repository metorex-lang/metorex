calls = 0
Dir.singleton_class.alias_method(:original_pwd, :pwd)
Dir.define_singleton_method(:pwd) do
  calls += 1
  original_pwd
end
File.expand_path("notes.txt")
File.absolute_path("notes.txt")
p calls
p File.expand_path("notes.txt") == File.join(Dir.original_pwd, "notes.txt")
p Dir.respond_to?(:__working_directory__)
