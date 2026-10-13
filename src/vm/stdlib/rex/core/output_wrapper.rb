module Rex
  module Core
    class OutputWrapper
      attr_accessor :output

      def initialize(output)
        @output = output
      end

      def respond_to?(name, priv=false)
        output.respond_to?(name, priv)
      end

      def method_missing(name, *args, &block)
        output.__send__(name, *args, &block)
      end

      IO.instance_methods(false).each do |method|
        define_method(method) do |*args, &block|
          output.__send__(method, *args, &block)
        end
      end
    end
  end
end
