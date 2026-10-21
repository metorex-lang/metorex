internal = Encoding.default_internal
internal_name = internal.nil? ? "none" : internal.name
puts("source #{__ENCODING__.name}")
puts("external #{Encoding.default_external.name}")
puts("internal #{internal_name}")
