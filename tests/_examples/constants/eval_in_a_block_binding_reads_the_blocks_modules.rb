# Code evaluated in a block's binding finds the constants of the modules the
# block was written in, an autoloaded one among them.
module Bukkits
  autoload :User, File.join(__dir__, "eval_in_a_block_binding", "user")
end

def measure(expression, &block)
  before = eval(expression, block.binding)
  block.call
  [before, eval(expression, block.binding)]
end

module Bukkits
  class UsersControllerTest
    def run_it
      measure("User.count") { :destroyed }
    end
  end
end

p Bukkits::UsersControllerTest.new.run_it
