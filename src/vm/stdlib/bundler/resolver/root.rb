# frozen_string_literal: true

Kernel.require "bundler/resolver/package"

module Bundler
  class Resolver
    #
    # Represents the Gemfile from the resolver's perspective. It's the root
    # package and Gemfile entries depend on it.
    #
    class Root < Package
      def initialize(name)
        @name = name
      end

      def meta?
        true
      end

      def root?
        true
      end
    end
  end
end
