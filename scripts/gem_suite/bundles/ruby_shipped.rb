# The versions Ruby 4.0.7 ships of the gems a suite loads beside the one it
# tests, as default or bundled gems, which metorex carries at the same
# versions. A Gemfile names the ones its suite reaches with `shipped`, so
# MRI and metorex load the same files of each.
SHIPPED_VERSIONS = {
  "base64" => "0.3.0", "bigdecimal" => "4.0.1", "date" => "3.5.1", "drb" => "2.2.3", "erb" => "6.0.7",
  "io-console" => "0.8.2", "irb" => "1.16.0", "json" => "2.18.0", "logger" => "1.7.0",
  "minitest" => "6.0.0", "mutex_m" => "0.3.0", "net-http" => "0.9.1", "net-protocol" => "0.2.2",
  "ostruct" => "0.6.3", "power_assert" => "3.0.1", "pp" => "0.6.3", "prettyprint" => "0.2.0",
  "prism" => "1.8.1", "psych" => "5.3.1", "rbs" => "3.10.0", "rdoc" => "7.0.4", "readline" => "0.0.4",
  "reline" => "0.6.3", "rexml" => "3.4.4", "stringio" => "3.2.0", "test-unit" => "3.7.5", "timeout" => "0.6.0",
  "tsort" => "0.2.0", "uri" => "1.1.1"
}.freeze

def shipped(*names)
  group :ruby_shipped do
    names.each { |name| gem name, SHIPPED_VERSIONS.fetch(name) }
  end
end
