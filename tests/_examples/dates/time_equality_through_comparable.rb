# Time's == comes from Comparable, so a <=> the program writes decides it.
class Time
  alias_method :ordered_against, :<=>

  def <=>(other)
    other.is_a?(String) ? 0 : ordered_against(other)
  end
end

moment = Time.at(0)
p moment == "1970-01-01"
p({ "at" => moment } == { "at" => "1970-01-01" })
p [moment] == ["1970-01-01"]
p moment == Time.at(0)
p moment == 0
p Time.instance_method(:==).owner
