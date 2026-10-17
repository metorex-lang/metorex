# A method written on a reopened core class is the one the class reports.
class Hash
  def fetch(key) = :fetched
end
p Hash.instance_method(:fetch).owner
p Hash.instance_method(:fetch).source_location.last
p({a: 1}.method(:fetch).call(:a))
