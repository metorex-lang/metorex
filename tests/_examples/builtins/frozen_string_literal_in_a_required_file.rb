# The frozen_string_literal comment of a required file freezes the String
# literals written in it, a `class << self` body's among them.
require_relative "support/frozen_schemes"
p FrozenSchemes::Top.frozen?
p FrozenSchemes.singleton_class.const_get(:Reserved).frozen?
# A library Ruby carries is read under its own comment the same way.
require "uri"
p URI.const_get(:Schemes).singleton_class.const_get(:ReservedChars).frozen?
