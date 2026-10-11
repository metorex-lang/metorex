module Warning
  def self.warn(message, category: nil)
    $stdout.print("warned: #{message.sub(/\A.*?warning: /, "")}")
  end
end
$VERBOSE = true

rates = [4, 9]
p(rates.fetch(0) { :missing })
p(rates.fetch(5) { |index| "no rate at #{index}" })
p(rates.fetch(1, :fallback) { :block })
p(rates.fetch(7, :fallback) { :block })
p(rates.fetch(7, :fallback))
