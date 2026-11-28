# A block written in a module and run in the class that includes it reads
# constants from the modules it was written in, even when the class's name
# is the name of one of the block's modules.
module ActiveModel
  module Naming
    def model_name = "named by ActiveModel::Naming"
  end

  module API
    def self.included(base)
      base.class_eval do
        extend ActiveModel::Naming
      end
    end
  end
end

module Strategies
  class ActiveModel
    class Model
      include ::ActiveModel::API
    end
  end
end

puts Strategies::ActiveModel::Model.model_name
