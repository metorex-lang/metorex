# A class defined while another thread is part-way through a method written
# in a block run as a class body lands where it is written, not in the class
# the other thread's method keeps its constants in, as Active Record's
# transaction classes do while a Rails initializer's thread waits.
class Base; end

class Railtie
  def self.initializer(queue)
    Thread.new { Base.class_exec { queue.pop } }
  end
end

queue = Queue.new
waiting = Railtie.initializer(queue)
Thread.pass until waiting.status == "sleep"

module Ledger
  module Adapters
    autoload :TransactionManager, File.join(__dir__, "transaction")
  end
end
p Ledger::Adapters::TransactionManager::NULL.class
p Railtie.constants
queue << :done
waiting.join
