# frozen_string_literal: true

if ENV['RUBY_DEBUG_LAZY']
  require "debug/prelude"
else
  require "debug/session"
  return unless defined?(DEBUGGER__)
  DEBUGGER__::start no_sigint_hook: true, nonstop: true
end
