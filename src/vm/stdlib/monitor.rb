# A monitor: a lock a fiber may take more than once, and the condition
# variables that wait on it. The lock underneath is a Thread::Mutex, so a
# fiber entering a monitor another one holds waits for it to be let go.

class Monitor
  def initialize
    @mon_mutex = Thread::Mutex.new
    @mon_owner = nil
    @mon_count = 0
  end

  def try_enter
    unless @mon_owner.equal?(Fiber.current)
      return false unless @mon_mutex.try_lock
      @mon_owner = Fiber.current
      @mon_count = 0
    end
    @mon_count += 1
    true
  end

  def enter
    unless @mon_owner.equal?(Fiber.current)
      @mon_mutex.lock
      @mon_owner = Fiber.current
      @mon_count = 0
    end
    @mon_count += 1
    nil
  end

  def exit
    mon_check_owner
    @mon_count -= 1
    if @mon_count == 0
      @mon_owner = nil
      @mon_mutex.unlock
    end
    nil
  end

  def synchronize
    enter
    begin
      yield
    ensure
      self.exit
    end
  end

  def mon_locked?
    @mon_mutex.locked?
  end

  def mon_owned?
    @mon_mutex.locked? && @mon_owner.equal?(Fiber.current)
  end

  # Raised on anyone who tries to leave a monitor they are not holding.
  def mon_check_owner
    unless @mon_owner.equal?(Fiber.current)
      raise ThreadError, "current fiber not owner"
    end
    nil
  end

  # Hands the lock back for the whole depth it is held to while waiting on
  # `cond`, and takes it again to that depth afterwards.
  def wait_for_cond(cond, timeout)
    count = @mon_count
    @mon_owner = nil
    @mon_count = 0
    begin
      cond.wait(@mon_mutex, timeout) ? true : false
    ensure
      @mon_owner = Fiber.current
      @mon_count = count
    end
  end

  def new_cond
    ::MonitorMixin::ConditionVariable.new(self)
  end

  alias try_mon_enter try_enter
  alias mon_try_enter try_enter
  alias mon_enter enter
  alias mon_exit exit
  alias mon_synchronize synchronize
end

module MonitorMixin
  # Something to wait on while the monitor is held. Waiting hands the lock
  # back for the duration and takes it again afterwards.
  class ConditionVariable
    def wait(timeout = nil)
      @monitor.mon_check_owner
      @monitor.wait_for_cond(@cond, timeout)
    end

    def wait_while
      while yield
        wait
      end
    end

    def wait_until
      until yield
        wait
      end
    end

    def signal
      @monitor.mon_check_owner
      @cond.signal
    end

    def broadcast
      @monitor.mon_check_owner
      @cond.broadcast
    end

    private

    def initialize(monitor)
      @monitor = monitor
      @cond = Thread::ConditionVariable.new
    end
  end

  def self.extend_object(object)
    super(object)
    object.__send__(:mon_initialize)
  end

  def mon_try_enter
    @mon_data.try_enter
  end
  alias try_mon_enter mon_try_enter

  def mon_enter
    @mon_data.enter
  end

  def mon_exit
    mon_check_owner
    @mon_data.exit
  end

  def mon_locked?
    @mon_data.mon_locked?
  end

  def mon_owned?
    @mon_data.mon_owned?
  end

  def mon_synchronize(&block)
    @mon_data.synchronize(&block)
  end
  alias synchronize mon_synchronize

  def new_cond
    unless defined?(@mon_data)
      mon_initialize
      @mon_initialized_by_new_cond = true
    end
    ConditionVariable.new(@mon_data)
  end

  private

  def initialize(...)
    super
    mon_initialize
  end

  def mon_initialize
    if defined?(@mon_data)
      if defined?(@mon_initialized_by_new_cond)
        return
      elsif @mon_data_owner_object_id == object_id
        raise ThreadError, "already initialized"
      end
    end
    @mon_data = ::Monitor.new
    @mon_data_owner_object_id = object_id
  end

  def mon_check_owner
    @mon_data.mon_check_owner
  end
end
