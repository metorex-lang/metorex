# A module path an `include` names reads each segment through the
# ancestors of the namespace before it, as a scoped constant read does.
module Markup
  module Parsers
    module Helpers; end
    class Markup
      include ::Markup
      include Markup::Parsers::Helpers
    end
  end
end
p Markup::Parsers::Markup.ancestors.take(3)
