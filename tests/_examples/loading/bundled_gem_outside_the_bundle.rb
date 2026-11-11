require "tmpdir"
Dir.mktmpdir do |dir|
  gemfile = File.join(dir, "Gemfile")
  File.write(gemfile, "")
  ENV["BUNDLE_GEMFILE"] = gemfile
  require "bundler/setup"
  begin
    require "rake"
    puts "loaded rake"
  rescue LoadError => error
    puts error.message
  end
  require "json"
  puts JSON.generate([1])
end
