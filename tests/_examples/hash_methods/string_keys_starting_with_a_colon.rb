# A String key that starts with a colon stays a String, apart from the
# Symbol spelled the same way.
hash = {
  '::ffff:192.0.2.1' => :address,
  ':name' => :string,
  name: :symbol,
}
p hash.keys
p hash.keys.map(&:class)
p hash[':name'], hash[:name], hash['::ffff:192.0.2.1']
hash.each { |key, value| p [key, value] }
p hash.key?(':name'), hash.key?(:':name')
