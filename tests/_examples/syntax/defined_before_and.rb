# `defined?` without parentheses takes the one operand before `and` or
# `or`, so the rest of the line is the other side of `and` or `or`. An
# instance variable main holds is defined at the top level, nil or not.
@parent = nil
found = (defined? @parent and @parent)
p found
found = (defined? @parent or :unreached)
p found
found = (defined? @missing and :unreached)
p found
p(defined? @parent && @parent)
:removed if defined? @parent and @parent
p :reached
