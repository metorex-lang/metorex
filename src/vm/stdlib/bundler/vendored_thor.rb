# frozen_string_literal: true

module Bundler
  def self.require_thor_actions
    Kernel.require "bundler/vendor/thor/lib/thor/actions"
  end
end
Kernel.require "bundler/vendor/thor/lib/thor"
