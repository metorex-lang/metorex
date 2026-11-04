module Shoulda
  module Matchers
    module ActiveModel
      # @private
      module Qualifiers
      end
    end
  end
end

require "shoulda/matchers/active_model/qualifiers/allow_nil"
require "shoulda/matchers/active_model/qualifiers/allow_blank"
require "shoulda/matchers/active_model/qualifiers/ignore_interference_by_writer"
require "shoulda/matchers/active_model/qualifiers/ignoring_interference_by_writer"
