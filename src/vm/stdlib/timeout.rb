# A block given a limit on how long it may run.
module Timeout
  # Raised inside the block when no class was given, so that a
  # `rescue StandardError` there does not stop it on its way out.
  class ExitException < Exception
    def exception(*)
      self
    end
  end

  class Error < RuntimeError
    def self.catch(*arguments, &block)
      made = new(*arguments)
      made.instance_variable_set(:@thread, nil)
      block.call(made) unless block.nil?
      made
    end

    def exception(*arguments)
      self
    end
  end

  # Runs the block, and answers what it answers. A limit of nil or zero puts
  # no limit on it at all.
  def self.timeout(sec, klass = nil, message = nil, &block)
    return block.call(sec) if sec.nil? || sec == 0
    unless sec.is_a?(Numeric)
      raise TypeError, "no implicit conversion of #{sec.class} into Integer"
    end
    if sec < 0
      raise ArgumentError, "Timeout sec must be a non-negative number"
    end
    text = message.nil? ? "execution expired" : message
    # A fiber that is not blocking leaves the limit to its scheduler.
    scheduler = Fiber.current_scheduler
    if !scheduler.nil? && scheduler.respond_to?(:timeout_after)
      return scheduler.timeout_after(sec, klass.nil? ? Timeout::Error : klass, text, &block)
    end
    raised = klass.nil? ? ExitException.new(text) : klass
    __timeout_open__ sec, raised, text
    begin
      block.call sec
    rescue ExitException => exited
      raise Timeout::Error, text if exited.equal?(raised)
      raise
    ensure
      __timeout_close__
    end
  end

  def timeout(sec, klass = nil, message = nil, &block)
    Timeout.timeout(sec, klass, message, &block)
  end
end

module Kernel
  def timeout(sec, klass = nil, message = nil, &block)
    Timeout.timeout(sec, klass, message, &block)
  end
  module_function :timeout
end

TimeoutError = Timeout::Error
