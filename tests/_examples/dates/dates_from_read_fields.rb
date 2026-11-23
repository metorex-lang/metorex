require "date"

p Date.strptime("2001-02-03", "%Y-%m-%d")
p Date.strptime("2001-034", "%Y-%j")
p Date.strptime("2001-W05-6", "%G-W%V-%u")
p Date.strptime("2001 04 6", "%Y %U %w")
p Date.strptime("2001 05 6", "%Y %W %u")
p Date.strptime("sat3feb01", "%a%d%b%y")
p Date.strptime("2001-02-03x", "%F")
p Date.strptime("1470272280", "%s")
p DateTime.strptime("2001-02-03T04:05:06+07:00", "%Y-%m-%dT%H:%M:%S%z")
p DateTime.strptime("03-02-2001 04:05:06 PM", "%d-%m-%Y %I:%M:%S %p")
p DateTime.strptime("2001-W05-6T04:05:06+07:00", "%G-W%V-%uT%H:%M:%S%z")
p DateTime.strptime("2001 04 6 04 05 06 +7", "%Y %U %w %H %M %S %z")
p DateTime.strptime("-1", "%s")
p DateTime.strptime("-1000", "%Q")
p DateTime.strptime("sat3feb014pm+7", "%a%d%b%y%H%p%z")
p DateTime.strptime("1470272280.123456789 +0530", "%s.%N %z")
p DateTime.strptime("2001-02-03 23:59:60", "%F %T")
p DateTime.strptime("2001-02-03 24:00:00", "%F %T")
p Date.parse("2001-02-03")
p Date.parse("2001-W05-6")
p DateTime.parse("2001-02-03T04:05:06+07:00")
p DateTime.parse("3rd Feb 2001 04:05:06 PM")
p Date.iso8601("2001-02-03")
p DateTime.iso8601("2001-02-03T04:05:06.5+01:00")
p DateTime.rfc2822("Sat, 3 Feb 2001 04:05:06 +0700")
p DateTime.httpdate("Sat, 03 Feb 2001 04:05:06 GMT")
%w[%H:%M 10:00].then do |format, text|
  begin
    Date.strptime(text, format)
  rescue Date::Error => error
    p error.message
  end
end
begin
  Date.strptime("2001-02-30", "%F")
rescue Date::Error => error
  p error.message
end
