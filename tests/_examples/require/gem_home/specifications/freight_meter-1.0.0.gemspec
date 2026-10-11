# -*- encoding: utf-8 -*-
# stub: freight_meter 1.0.0 ruby lib

Gem::Specification.new do |s|
  s.name = "freight_meter".freeze
  s.version = "1.0.0".freeze
  s.require_paths = ["lib".freeze]
  s.authors = ["Metorex".freeze]
  s.summary = "Freight charges computed in C".freeze
  s.extensions = ["ext/freight_meter/extconf.rb".freeze]
  s.add_runtime_dependency(%q<tariff_table>.freeze, [">= 1.0".freeze, "< 3.0".freeze])
  s.add_runtime_dependency(%q<prism>.freeze, [">= 1.0".freeze])
end
