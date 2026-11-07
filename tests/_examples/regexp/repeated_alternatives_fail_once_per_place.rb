subject = "a" * 90
p subject.match?(/\A(?:(?:a|aa)*x|(?:a|aa)*)\z/)
p subject.match?(/\A(?:(?:a|aa)+)+x/)
p (subject + "y") =~ /(?:(?:a|aa)*)*y/

payload = ";" + ("a=1;" * 2000) + "!"
pattern = /\A(?:|;(?<foo>(?:(?:[a-zA-Z0-9\-\.\_\~]|%[a-fA-F0-9][a-fA-F0-9])*=(?:[a-zA-Z0-9\-\.\_\~]|%[a-fA-F0-9][a-fA-F0-9])*?|(?:[a-zA-Z0-9\-\.\_\~]|%[a-fA-F0-9][a-fA-F0-9])*))?;?(?<bar>(?:(?<![a-zA-Z0-9\-\.\_\~])(?:(?:[a-zA-Z0-9\-\.\_\~]|%[a-fA-F0-9][a-fA-F0-9])|=)*(?:;(?:(?:[a-zA-Z0-9\-\.\_\~]|%[a-fA-F0-9][a-fA-F0-9])|=)*)*)?);?(?<baz>))\z/
p payload.match(pattern)
matched = (";" + ("a=1;" * 2000)).match(pattern)
p matched[:foo], matched[:bar].length, matched[:baz]

p "aaab".match(/(a|ab)*b/)[0], "abab".match(/(?:a|ab)*?b/)[0]
p "xaaay"[/a+?y/], "aaa".scan(/a|aa/)
