module Rex
  module Core
    class Set
      include Enumerable

      def initialize(array=[])
        @values = {}
        merge(array)
      end

      def empty?
        @values.empty?
      end

      def <<(key)
        @values[key] = true
        self
      end

      def delete(key)
        @values.delete(key)
      end

      def each(&block)
        @values.keys.each(&block)
        self
      end

      def include?(key)
        @values.key?(key)
      end

      def merge(values)
        values.each do |key|
          @values[key] = true
        end
        self
      end

      def clear
        @values.clear
        self
      end
    end
  end
end
