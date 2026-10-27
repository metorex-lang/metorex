# A block sets `$~` in the frame it was written in, while a method it was
# handed to runs it, and the method's own match stays its own.
def run_given
  yield
  $~
end

inside = run_given { "order 42" =~ /\d+/ }
p inside
p $~ && $~[0]

def match_and_run
  "code A7" =~ /[A-Z]\d/
  yield
  $~[0]
end

p match_and_run { "zone 9" =~ /\d/ }
p $~[0]
