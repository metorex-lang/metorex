# What a gem's metadata writes: a sequence at its key's own indentation, a
# sequence opened inside another on one line, a tag naming the class of a
# mapping under a key and of one in a sequence, and a timestamp in UTC.
require "yaml"

class Version
  attr_reader :version
end

text = <<~YAML
  requirements:
  - - ">="
    - !ruby/object:Version
      version: '0'
  required: !ruby/object:Version
    version: 1.0.0
  date: 1980-01-02 00:00:00.000000000 Z
  offset: 2026-10-21 08:30:00 +05:00
YAML
read = YAML.unsafe_load(text)
p read["requirements"].map { |(operator, version)| [operator, version.version] }
p read["required"].version
p read["date"]
p read["date"].utc?
p read["offset"]
