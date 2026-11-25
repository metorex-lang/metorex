module Emitter
  def initialize
    @listeners = Hash.new { |hash, key| hash[key] = [] }
  end
end

class Driver
  include Emitter

  def self.validate(options, allowed)
    options.keys.each do |key|
      raise ArgumentError, "unknown option #{key}" unless allowed.include?(key)
    end
  end

  def initialize(socket, options = {})
    super()
    Driver.validate(options, [:masking, :protocols])
    @socket = socket
    @options = options
    @queue = []
  end
end

class Framed < Driver
  def initialize(socket, options = {})
    super
    @extensions = [socket]
    @stage = 0
  end

  def described
    [@socket, @extensions, @stage]
  end
end

built = Queue.new
threads = 20.times.map do |number|
  Thread.new do
    200.times { |round| built << Framed.new("socket #{number}-#{round}", protocols: ["chat"]) }
  end
end
threads.each(&:join)

all = []
all << built.pop until built.empty?
wrong = all.reject { |held| held.described == [held.described[0], [held.described[0]], 0] }
p all.size
p wrong.size
p wrong.first&.instance_variables
