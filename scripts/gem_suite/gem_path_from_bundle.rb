# Puts the bundle's gems on GEM_PATH before Bundler loads. railties' generator
# tests run a generated app's bin/rails inside Bundler.with_unbundled_env,
# which removes BUNDLE_PATH but keeps GEM_PATH, so the app resolves its
# Gemfile from the bundle.
require "rbconfig"

bundle_gems = File.join(ENV.fetch("BUNDLE_PATH"), "ruby", RbConfig::CONFIG["ruby_version"])
ENV["GEM_PATH"] = [bundle_gems, *Gem.path].uniq.join(File::PATH_SEPARATOR)
