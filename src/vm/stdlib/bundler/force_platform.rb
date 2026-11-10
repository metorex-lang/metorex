# frozen_string_literal: true

module Bundler
  module ForcePlatform
    # The `:force_ruby_platform` value used by dependencies for resolution, and
    # by locked specifications for materialization is `false` by default, except
    # for TruffleRuby. TruffleRuby generally needs to force the RUBY platform
    # variant unless the name is explicitly allowlisted.
    #
    # Metorex loads no library compiled for MRI, so it takes the RUBY platform
    # variant of every gem and builds its extensions, except a gem whose
    # library it provides itself, where the variant installed is never loaded.

    def default_force_ruby_platform
      return !Gem.provided_by_metorex?(name) if Gem.respond_to?(:provided_by_metorex?)
      return false unless RUBY_ENGINE == "truffleruby"

      !Gem::Platform::REUSE_AS_BINARY_ON_TRUFFLERUBY.include?(name)
    end
  end
end
