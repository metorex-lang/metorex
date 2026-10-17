# begin, rescue, else, ensure, raise, retry, and throw as a body compiles
# them, printed so the bytecode engine's answers can be compared with the
# tree walker's and Ruby's.

class Custom < StandardError
end

ERRORS = [KeyError, Custom]

def rescued(kind)
  log = []
  value = begin
    log << :body
    raise Custom, "custom" if kind == :custom
    raise ArgumentError, "argument" if kind == :argument
    Integer("x") if kind == :internal
    {}.fetch(:missing) if kind == :key
    :finished
  rescue *ERRORS => error
    log << error.class
    :splatted
  rescue ArgumentError, TypeError => error
    log << error.message
    :listed
  rescue => error
    log << error.class
    :standard
  else
    log << :else
    :else_value
  ensure
    log << :ensure
  end
  [value, log, $!]
end

def retried
  attempts = 0
  begin
    attempts += 1
    raise "again" if attempts < 3
    attempts
  rescue
    retry
  end
end

def ensure_on_exits
  log = []
  [1, 2, 3].each do |number|
    begin
      next if number == 1
      break if number == 3
      log << number
    ensure
      log << "ensure #{number}"
    end
  end
  while true
    begin
      break
    ensure
      log << :loop_ensure
    end
  end
  log
end

def returns_through_ensure(log)
  begin
    return :returned
  ensure
    log << :ensured
  end
end

def unrescued
  begin
    raise Custom, "passes"
  rescue ArgumentError
    :wrong
  end
rescue Custom => error
  [:method_level, error.message]
end

def caused
  begin
    begin
      raise "first"
    rescue
      raise ArgumentError, "second"
    end
  rescue => error
    [error.message, error.cause&.message]
  end
end

def thrown
  catch(:done) do
    [1, 2].each { |number| throw :done, number * 10 if number == 2 }
    :not_thrown
  end
end

p rescued(:custom)
p rescued(:argument)
p rescued(:internal)
p rescued(:key)
p rescued(:none)
p retried
p ensure_on_exits
log = []
p [returns_through_ensure(log), log]
p unrescued
p caused
p thrown
p $!
