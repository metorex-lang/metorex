require "matrix"
require "time"
require "ostruct"

# The matrix, time and ostruct libraries name some methods twice with
# `alias`, which a Method for either name reports.
p Matrix.instance_method(:map).original_name
same = Matrix.instance_method(:collect) == Matrix.instance_method(:map)
p same
same = Matrix.method(:I) == Matrix.method(:identity)
p same
same = Vector.instance_method(:norm) == Vector.instance_method(:magnitude)
p same
square = Matrix[[1, 2], [3, 4]]
p square.t
p square.det
same = Time.method(:rfc822) == Time.method(:rfc2822)
p same
same = Time.method(:iso8601) == Time.method(:xmlschema)
p same
epoch = Time.at 0
p epoch.utc.rfc822
same = OpenStruct.instance_method(:to_s) == OpenStruct.instance_method(:inspect)
p same
