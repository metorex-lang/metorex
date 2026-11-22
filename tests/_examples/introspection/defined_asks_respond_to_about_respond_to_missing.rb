class RespondToFalse
  def respond_to?(name, include_all = false)
    p [:respond_to?, name, include_all]
    false
  end

  def respond_to_missing?(name, include_all)
    p [:missing, name, include_all]
    true
  end
end

class RespondToTrue
  def respond_to?(name, include_all = false)
    p [:respond_to?, name, include_all]
    true
  end

  def respond_to_missing?(name, include_all)
    p [:missing, name, include_all]
    true
  end
end

class RespondToOneParameter
  def respond_to?(name)
    false
  end
end

class Plain
  def respond_to_missing?(name, include_all)
    p [:missing, name, include_all]
    name == :yes
  end
end
p defined?(RespondToFalse.new.an_undefined_method)
p defined?(RespondToTrue.new.an_undefined_method)
p defined?(Plain.new.yes)
p defined?(Plain.new.no)

require "stringio"
captured = StringIO.new
$stderr = captured
Warning[:deprecated] = true
p defined?(RespondToOneParameter.new.an_undefined_method)
Warning[:deprecated] = false
p defined?(RespondToOneParameter.new.an_undefined_method)
$stderr = STDERR
puts captured.string.gsub(/^.*?:(\d+): /, 'line \1: ')
