require "matrix"
require "time"
require "ostruct"

# The matrix, time and ostruct libraries name some methods twice with
# `alias`, which a Method for either name reports.
p Matrix.instance_method(:map).original_name
p Matrix.instance_method(:collect) == Matrix.instance_method(:map)
p Matrix.method(:I) == Matrix.method(:identity)
p Vector.instance_method(:norm) == Vector.instance_method(:magnitude)
p Matrix[[1, 2], [3, 4]].t
p Matrix[[1, 2], [3, 4]].det
p Time.method(:rfc822) == Time.method(:rfc2822)
p Time.method(:iso8601) == Time.method(:xmlschema)
p Time.at(0).utc.rfc822
p OpenStruct.instance_method(:to_s) == OpenStruct.instance_method(:inspect)
