$blocks = []
def later(&block) = $blocks << block
load File.join(__dir__, "examples.rb")
$blocks.each(&:call)
