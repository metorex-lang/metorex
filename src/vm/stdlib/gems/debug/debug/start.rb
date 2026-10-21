# frozen_string_literal: true

require "debug/session"
return unless defined?(DEBUGGER__)
DEBUGGER__.start no_sigint_hook: false
