# Copies the files a gem's Rakefile generates with Ragel, Racc, or
# oedipus_lex from the released gem of the same version, which the gem's
# bundle installs, into the copy of the gem a test file runs in. It runs in
# that copy, whose directory is named after the gem.
require "bundler/setup"
require "fileutils"

VERSION_FILES = {
  "parser" => "lib/parser/version.rb",
  "regexp_parser" => "lib/regexp_parser/version.rb",
  "rubocop-ast" => "lib/rubocop/ast/version.rb"
}.freeze

GENERATED_FILES = {
  "parser" => %w[lexer-F0 lexer-F1 lexer-strings ruby18 ruby19 ruby20 ruby21 ruby22 ruby23 ruby24 ruby25
                 ruby26 ruby27 ruby30 ruby31 ruby32 ruby33 ruby34 macruby rubymotion].map { |name| "lib/parser/#{name}.rb" },
  "regexp_parser" => %w[lib/regexp_parser/scanner.rb],
  "rubocop-ast" => %w[lib/rubocop/ast/node_pattern/lexer.rex.rb lib/rubocop/ast/node_pattern/parser.racc.rb]
}.freeze

gem_name = File.basename(Dir.pwd)
released = Gem.loaded_specs.fetch(gem_name)
checkout_version = File.read(VERSION_FILES.fetch(gem_name))[/(?:VERSION|STRING) = '([^']+)'/, 1]
unless released.version.to_s == checkout_version
  abort "#{gem_name}'s bundle has #{released.version} and the checkout is #{checkout_version}"
end

GENERATED_FILES.fetch(gem_name).each do |path|
  FileUtils.cp(File.join(released.full_gem_path, path), path)
end
