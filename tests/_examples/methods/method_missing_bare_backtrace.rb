require_relative "method_missing_bare_runner"

class Checker
  include BarePredicates

  def check = acount
end

begin
  Checker.new.check
rescue NameError => error
  puts error.backtrace.first(3).map { |line| line.sub(/\A.*\//, "") }
end

class ExplicitChecker
  include BarePredicates

  def check = self.acount
end

begin
  ExplicitChecker.new.check
rescue NoMethodError => error
  puts error.backtrace.first(2).map { |line| line.sub(/\A.*\//, "") }
end
