module Rex
  module Core
    module MetadataFilter
      class << self
        def apply?(predicate, filters, metadata)
          filters.__send__(predicate) { |k, v| filter_applies?(k, v, metadata) }
        end

        def filter_applies?(key, filter_value, metadata)
          silence_metadata_example_group_deprecations do
            return location_filter_applies?(filter_value, metadata) if key == :locations
            return id_filter_applies?(filter_value, metadata)       if key == :ids
            return filters_apply?(key, filter_value, metadata)      if Hash === filter_value

            meta_value = metadata.fetch(key) { return false }

            return true if TrueClass === filter_value && meta_value
            return proc_filter_applies?(key, filter_value, metadata) if Proc === filter_value
            return filter_applies_to_any_value?(key, filter_value, metadata) if Array === meta_value

            filter_value === meta_value || filter_value.to_s == meta_value.to_s
          end
        end

        def silence_metadata_example_group_deprecations
          Rex::Support.thread_local_data[:silence_metadata_example_group_deprecations] = true
          yield
        ensure
          Rex::Support.thread_local_data.delete(:silence_metadata_example_group_deprecations)
        end

      private

        def filter_applies_to_any_value?(key, value, metadata)
          metadata[key].any? { |v| filter_applies?(key, v,  key => value) }
        end

        def id_filter_applies?(rerun_paths_to_scoped_ids, metadata)
          scoped_ids = rerun_paths_to_scoped_ids.fetch(metadata[:rerun_file_path]) { return false }

          Metadata.ascend(metadata).any? do |meta|
            scoped_ids.include?(meta[:scoped_id])
          end
        end

        def location_filter_applies?(locations, metadata)
          Metadata.ascend(metadata).any? do |meta|
            file_path = meta[:absolute_file_path]
            line_num  = meta[:line_number]

            locations[file_path].any? do |filter_line_num|
              line_num == Rex.world.preceding_declaration_line(file_path, filter_line_num)
            end
          end
        end

        def proc_filter_applies?(key, proc, metadata)
          case proc.arity
          when 0 then proc.call
          when 2 then proc.call(metadata[key], metadata)
          else proc.call(metadata[key])
          end
        end

        def filters_apply?(key, value, metadata)
          subhash = metadata[key]
          return false unless Hash === subhash || HashImitatable === subhash
          value.all? { |k, v| filter_applies?(k, v, subhash) }
        end
      end
    end

    module FilterableItemRepository
      # How many times any repository gained or lost an item, which a list
      # built from repositories is kept under.
      def self.changes
        @changes ||= 0
      end

      def self.changed
        @changes = changes + 1
      end

      class UpdateOptimized
        attr_reader :items_and_filters

        def initialize(applies_predicate)
          @applies_predicate = applies_predicate
          @items_and_filters = []
        end

        def append(item, metadata)
          FilterableItemRepository.changed
          @items_and_filters << [item, metadata]
        end

        def prepend(item, metadata)
          FilterableItemRepository.changed
          @items_and_filters.unshift [item, metadata]
        end

        def delete(item, metadata)
          FilterableItemRepository.changed
          @items_and_filters.delete [item, metadata]
        end

        def items_for(request_meta)
          @items_and_filters.each_with_object([]) do |(item, item_meta), to_return|
            to_return << item if item_meta.empty? ||
                                 MetadataFilter.apply?(@applies_predicate, item_meta, request_meta)
          end
        end

        unless [].respond_to?(:each_with_object)
          undef items_for
          def items_for(request_meta)
            @items_and_filters.inject([]) do |to_return, (item, item_meta)|
              to_return << item if item_meta.empty? ||
                                   MetadataFilter.apply?(@applies_predicate, item_meta, request_meta)
              to_return
            end
          end
        end
      end

      class QueryOptimized < UpdateOptimized
        alias find_items_for items_for
        private :find_items_for

        def initialize(applies_predicate)
          super
          @applicable_keys   = Set.new
          @proc_keys         = Set.new
          @memoized_lookups  = Hash.new do |hash, applicable_metadata|
            hash[applicable_metadata] = find_items_for(applicable_metadata)
          end
        end

        def append(item, metadata)
          super
          handle_mutation(metadata)
        end

        def prepend(item, metadata)
          super
          handle_mutation(metadata)
        end

        def delete(item, metadata)
          super
          reconstruct_caches
        end

        def items_for(metadata)
          applicable_metadata = applicable_metadata_from(metadata)

          if applicable_metadata.any? { |k, _| @proc_keys.include?(k) }
            find_items_for(applicable_metadata)
          else
            @memoized_lookups[applicable_metadata]
          end
        end

      private

        def reconstruct_caches
          @applicable_keys.clear
          @proc_keys.clear
          @items_and_filters.each do |_item, metadata|
            handle_mutation(metadata)
          end
        end

        def handle_mutation(metadata)
          @applicable_keys.merge(metadata.keys)
          @proc_keys.merge(proc_keys_from metadata)
          @memoized_lookups.clear
        end

        def applicable_metadata_from(metadata)
          MetadataFilter.silence_metadata_example_group_deprecations do
            @applicable_keys.inject({}) do |hash, key|
              hash[key] = metadata[key] if metadata.key?(key) || key == :example_group
              hash
            end
          end
        end

        def proc_keys_from(metadata)
          metadata.each_with_object([]) do |(key, value), to_return|
            to_return << key if Proc === value
          end
        end

        unless [].respond_to?(:each_with_object)
          undef proc_keys_from
          def proc_keys_from(metadata)
            metadata.inject([]) do |to_return, (key, value)|
              to_return << key if Proc === value
              to_return
            end
          end
        end
      end
    end
  end
end
