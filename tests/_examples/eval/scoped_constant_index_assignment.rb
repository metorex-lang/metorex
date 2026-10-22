module Outer
  class Scheme
    class << self
      def []=(key, value)
        puts "set #{key}=#{value}"
      end
    end
  end
end

held = Outer::Scheme
held["local"] = 0
Outer::Scheme["inverted"] = 1
::Outer::Scheme["top"] = 2
