obj = Object.new
def obj.to_str; "blah" end
re = /blah/
p(re =~ obj)
p(re.match(obj))
p(re === obj)
p(re.match?(obj))
x = re =~ obj
p x
