def inner(text)
  text =~ /(\w+)-(\d+)/
  $2
end

def outer(tag, parts)
  case tag
  when /^!ruby\/exception:?(.*)?$/
    seen = parts.map { |part| inner(part) }
    [$1, seen]
  end
end

p outer("!ruby/exception:Net::IMAP::ResponseParseError", ["invoice-1042", "order-7"])

def through_block
  "alpha beta" =~ /(beta)/
  [1].each { "gamma" =~ /(gam)/ }
  $1
end
p through_block

def caller_match
  "x-1" =~ /(x)/
  callee_match
  $1
end

def callee_match
  "y" =~ /(y)/
end
p caller_match
p callee_match && $~ && $~[0]

def make_blocks
  [proc { "invoice-1042" =~ /(\d+)/ }, proc { $1 }]
end

setter, reader = make_blocks
setter.call
p reader.call
p $1
p "wawa".to_enum(:scan, /./).map { |o| $& }
def make = proc { "wawa".to_enum(:scan, /./).map { |o| $& } }
body = make
p body.call
def runner(block) = block.call
p runner(body)
