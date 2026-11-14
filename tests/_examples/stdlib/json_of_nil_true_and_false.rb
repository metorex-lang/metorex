require "json"

p nil.to_json, true.to_json, false.to_json
p nil.method(:to_json).owner
p({ body_field: nil }.to_json)
