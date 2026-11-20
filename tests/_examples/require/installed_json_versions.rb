# An installed json gem of the version metorex carries is left to the carried
# copy. Another version is built for metorex and loaded, and one whose
# extensions do not build is left to the carried copy too. An installed
# fiddle is built and loaded in place of the part of it metorex carries.
%w[2.7.1 2.18.3 2.6.0].each do |version|
  shown = IO.popen([Gem.ruby, "-e", <<~RUBY], err: File::NULL, &:read)
    gem "json", "= #{version}"
    require "json"
    carried = $LOADED_FEATURES.grep(/json\\.rb\\z/).all? { |feature| feature.start_with?("<metorex>") }
    p [JSON::VERSION, carried, defined?(JSON::Ext::PARSER_BUILT) ? true : false]
  RUBY
  print shown
end
print IO.popen([Gem.ruby, "-e", <<~RUBY], err: File::NULL, &:read)
  gem "fiddle"
  require "fiddle"
  p [Fiddle::VERSION, Fiddle::BUILD_RUBY_PLATFORM == RUBY_PLATFORM]
RUBY
