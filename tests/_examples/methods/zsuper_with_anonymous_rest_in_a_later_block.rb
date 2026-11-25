class Recorder
  def initialize
    @seen = []
  end

  attr_reader :seen

  def add(channel, subscriber, on_success)
    @seen << [channel, subscriber]
    on_success&.call
  end
end

class Deferred < Recorder
  def initialize
    @pending = []
    super()
  end

  def add(*)
    @pending << -> { super }
  end

  def run
    @pending.each(&:call)
  end
end

deferred = Deferred.new
deferred.add("news", :reader, -> { puts "subscribed" })
p deferred.seen
deferred.run
p deferred.seen

class Threaded < Recorder
  def add(*)
    Thread.new { super }.join
  end
end

threaded = Threaded.new
threaded.add("sports", :viewer, nil)
p threaded.seen
