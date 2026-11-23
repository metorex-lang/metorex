pattern = /\A((?:gmt|utc?)?[-+]\d+(?:[,.:]\d+(?::\d+)?)?|(?-i:[[:alpha:].\s]+)(?:standard|daylight)\s+time\b|(?-i:[[:alpha:]]+)(?:\s+dst)?\b)/i
p pattern.match("GMT-0700 (PDT)")[1]
p pattern.match("Eastern Standard Time rest")[1]
p pattern.match("EST dst x")[1]
p pattern.match("-08:00:00")[1]
p pattern.match("1234")
