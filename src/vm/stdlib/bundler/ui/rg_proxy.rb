# frozen_string_literal: true

Kernel.require "bundler/ui"
require "rubygems/user_interaction"

module Bundler
  module UI
    class RGProxy < ::Gem::SilentUI
      def initialize(ui)
        @ui = ui
        super()
      end

      def say(message)
        @ui&.debug(message)
      end
    end
  end
end
