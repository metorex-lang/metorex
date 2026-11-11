module Outer
  require_relative "inner"
end
p Tools::NAME, respond_to?(:helper_path, true)
