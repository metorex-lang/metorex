require "tmpdir"
Dir.mktmpdir do |root|
  first = File.join(root, "first")
  second = File.join(root, "second")
  Dir.mkdir first
  Dir.mkdir second
  File.write(File.join(first, "lib_in_first.rb"), "puts 'loaded from first'\n")
  File.write(File.join(second, "lib_in_second.rb"), "puts 'loaded from second'\n")
  link = File.join(root, "link")
  File.symlink(first, link)
  $LOAD_PATH.unshift link
  require "lib_in_first"
  File.unlink link
  File.symlink(second, link)
  begin
    require "lib_in_second"
  rescue LoadError => e
    puts e.message
  end
  $LOAD_PATH.push root
  require "lib_in_second"
  p $LOADED_FEATURES.grep(/lib_in_/).map { |feature| File.basename(File.dirname(feature)) }
end
