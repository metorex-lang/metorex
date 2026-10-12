# frozen_string_literal: false
# -*- ruby -*-

require "rubygems/vendor/optparse/lib/optparse"
require "rubygems/vendor/uri/lib/uri"

Gem::OptionParser.accept(Gem::URI) {|s,| Gem::URI.parse(s) if s}
