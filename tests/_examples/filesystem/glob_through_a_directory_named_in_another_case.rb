require "fileutils"
require "tmpdir"

# A volume that treats names differing only in case as one name, as macOS
# volumes do, matches a glob without regard to case and answers the names as
# the directory holds them. A case-sensitive volume matches none of these.
Dir.mktmpdir do |root|
  FileUtils.mkdir_p(File.join(root, "ab", "inner"))
  File.write(File.join(root, "ab", "inner", "file"), "x")
  folds_case = File.exist?(File.join(root, "AB"))
  expected = folds_case ? ["/ab/inner/file"] : []

  ["AB/inner/*", "AB/inner/fil?", "A?/inner/*", "AB/inner/file", "ab/INNER/FILE"].each do |pattern|
    found = Dir.glob("#{root}/#{pattern}").map { |path| path.delete_prefix(root) }
    puts "#{pattern}: #{found == expected}"
  end
  p Dir.glob("#{root}/ab/inner/*").map { |path| path.delete_prefix(root) }
end
