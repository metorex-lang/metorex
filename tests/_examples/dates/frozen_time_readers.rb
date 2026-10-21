# A frozen Time answers its readers, and `utc` on one already in UTC
# answers it unchanged, as RubyGems reads the build date.
stamp = Time.at(1_792_800_000).utc.freeze
p stamp.utc.equal?(stamp)
p stamp.to_a[3..5]
p Time.utc(*stamp.utc.to_a[3..5].reverse)
