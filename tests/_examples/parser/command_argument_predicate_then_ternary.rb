module Forking; end
module Subprocess; end

class Runner
  def self.forking_env?
    false
  end

  include forking_env? ? Forking : Subprocess
end
p Runner.include?(Subprocess)

def choose(flag) = flag
def ready? = true
p choose ready? ? :yes : :no
p(choose ready? ? 1 : 2)
module Loud; end
helper = Object.new
def helper.loud? = true
which = Object.new
which.extend helper.loud? ? Loud : Subprocess
p which.is_a?(Loud)
