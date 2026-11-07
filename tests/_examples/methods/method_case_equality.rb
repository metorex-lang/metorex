def starts_with_ruby?(text)
  text&.start_with?("ruby")
end

checker = method(:starts_with_ruby?)
p checker === "ruby-jwt", checker === "python"
p(case "ruby-gems" when checker then :matched else :missed end)
p(case "go" when *[checker] then :matched else :missed end)
p 5.method(:<) === 7
unbound = Integer.instance_method(:even?)
p unbound.bind(4) === nil rescue p $!.class
