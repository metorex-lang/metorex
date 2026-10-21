class Reporter
  def notify(event, *args)
    args.size
  end
end

module Enforce
  def notify(event, *args)
    return super if caller_locations(1, 1).first.label =~ /publish/
    return super if %i[start stop].include?(event)

    raise "#{event.inspect} is not listed"
  end
end
Reporter.prepend(Enforce)

reporter = Reporter.new
total = 0
20_000.times { total += reporter.notify(:stop, 1, 2) }
p total
