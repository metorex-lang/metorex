# A `when` list may open with an index of a scoped constant, as RubyGems
# compares a path with the directories RbConfig names.
module Settings
  PATHS = { "site" => "/srv/site", "vendor" => "/srv/vendor" }.freeze
end

def placement(path)
  case path
  when Settings::PATHS["site"], Settings::PATHS["vendor"]
    :installed
  else
    :elsewhere
  end
end

p placement("/srv/vendor")
p placement("/home/ledger")
