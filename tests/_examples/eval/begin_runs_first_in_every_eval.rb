module Checker
  def self.complete?(code)
    catch(:valid) do
      eval("BEGIN { throw :valid }\n#{code}")
    end
    true
  end
end

$runs = []
p Checker.complete?("$runs << :first")
p Checker.complete?("$runs << :second")
p $runs
