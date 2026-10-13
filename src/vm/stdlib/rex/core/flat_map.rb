module Rex
  module Core
    module FlatMap
      if [].respond_to?(:flat_map)
        def flat_map(array, &block)
          array.flat_map(&block)
        end
      else
        def flat_map(array, &block)
          array.map(&block).flatten(1)
        end
      end

      module_function :flat_map
    end
  end
end
