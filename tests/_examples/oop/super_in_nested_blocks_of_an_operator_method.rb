# super with an argument, two blocks deep inside an operator method, calls
# the parent's operator method with that argument.
class Pool
  attr_reader :items

  def initialize
    @items = []
    @mutex = Mutex.new
  end

  def <<(item)
    @items << item
  end

  def with_mutex
    @mutex.synchronize { yield }
  end
end

class BatchPool < Pool
  def <<(work)
    work = [work] unless work.is_a?(Array)
    with_mutex do
      work.each { |item| super item }
    end
  end
end

pool = BatchPool.new
pool << 1
pool << [2, 3]
p pool.items
