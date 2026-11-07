def pick(throw: false, raise: false)
  (throw && :throw || raise && :raise)
end
p pick, pick(raise: true), pick(throw: true)
def again(raise: false)
  raise
end
p again(raise: 3)
def positional(key, raise)
  raise ? [key, raise] : key
end
p positional(:a, false), positional(:b, :loud)
