class Recorder
  def self.method_added(name)
    location = caller_locations(1, 1)[0]
    puts "#{name} #{location.lineno} #{location.label}"
  end
end

Class.new(Recorder) do
  def in_a_block
  end

  marker = 1; def after_a_statement
  end
end

class Named < Recorder
  def in_a_class_body
  end
end

class Maker
  def make
    line_before = nil
    Class.new(Recorder) do
      line_before = __LINE__
      def in_a_method_block
      end
    end
    line_before
  end
end
puts Maker.new.make
