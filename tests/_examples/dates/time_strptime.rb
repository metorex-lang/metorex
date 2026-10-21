# Time.strptime reads a time in the format given, as net-imap reads a
# message's internal date.
require "time"

held = Time.strptime("17-Jul-1996 02:44:25 -0700", "%d-%b-%Y %H:%M:%S %z")
p held
p held.utc_offset
p Time.strptime("2026-10-21 08:30:15 +0000", "%Y-%m-%d %H:%M:%S %z").utc?
p Time.strptime("1792800000", "%s").to_i
p Time.strptime("2026-10-21 UTC", "%Y-%m-%d %Z").utc?
begin
  Time.strptime("bad", "%Y-%m-%d")
rescue ArgumentError => error
  p error.message
end

# Date._strptime names the seconds since the epoch and the zone as written.
p Date._strptime("1792800000", "%s")
p Date._strptime("2026-10-21 UTC", "%Y-%m-%d %Z")
p Date.strptime("1792800000", "%s").to_s
p DateTime.strptime("1792800000", "%s").to_s
