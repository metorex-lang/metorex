require "coverage"
Coverage.start(branches: true)
require_relative "ledger"
Coverage.result.each_value do |report|
  report[:branches].each { |condition, arms| puts "#{condition.inspect} => #{arms.inspect}" }
end
