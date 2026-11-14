require "time"

p Date._strptime("20150830T123600Z", "%Y%m%dT%H%M%S%Z")
p Date._strptime("20150830", "%Y%m%d")
p Date._strptime("2015083012", "%Y%m%d%H")
p Date._strptime("123456", "%Y")
p Time.strptime("20150830T123600Z", "%Y%m%dT%H%M%S%Z")
p Date._parse("20160101T112233Z")
p Date._parse("20160101T1122+0900")
p Date._parse("20160101T11")
p Date._parse("0101T1122")
p Time.parse("20160101T112233Z")
p Date._parse("2015-08-30T12:36:00Z").sort.to_h
p Date._parse("2015-08-30T1236")
p Time.parse("2015-08-30T12:36:00Z")
