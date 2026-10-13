module Rex
  module Core
    # The groups a run registered, the filters that pick which examples run,
    # and the reporter they report to.
    class World
      attr_reader :example_groups, :location_filters, :description_filters
      attr_accessor :reporter, :wants_to_quit, :non_example_failure

      def initialize
        @example_groups = []
        @top_level_indexes = Hash.new(0)
        @location_filters = {}
        @description_filters = []
        @wants_to_quit = false
        @filtered_examples = {}
        @focus_applies = nil
        @non_example_failure = false
      end

      def register(group)
        @example_groups << group
        group
      end

      def next_top_level_index(file_path)
        @top_level_indexes[file_path] += 1
      end

      def add_location_filter(path, line_numbers)
        (@location_filters[File.expand_path(path)] ||= []).concat(line_numbers)
      end

      def add_description_filter(pattern)
        @description_filters << pattern
      end

      def description_filter
        @description_filters.empty? ? nil : Regexp.union(*@description_filters)
      end

      def add_id_filter(path, scoped_ids)
        ((@id_filters ||= {})[File.expand_path(path)] ||= []).concat(scoped_ids)
      end

      def all_examples
        @example_groups.flat_map(&:descendants).flat_map(&:examples)
      end

      def example_count(groups = @example_groups)
        groups.flat_map(&:descendant_filtered_examples).size
      end

      def filtered_examples_for(group)
        @filtered_examples[group] ||= group.examples.select { |example| runs?(example) }
      end

      def runs?(example)
        metadata = example.metadata
        return false unless location_allows?(example)
        return false unless description_allows?(example)
        return false unless id_allows?(example)

        configuration = Rex.configuration
        exclusion = configuration.exclusion_filter
        return false if !exclusion.empty? && Metadata.filter_applies_to_any?(exclusion, metadata)

        inclusion = inclusion_filter
        return true if inclusion.empty?

        Metadata.filter_applies_to_any?(inclusion, metadata)
      end

      # The inclusion filter in force: the configured one, plus the
      # `filter_run_when_matching` one when some example matches it.
      def inclusion_filter
        @inclusion_filter ||= begin
          filter = Rex.configuration.inclusion_filter.dup
          when_matching = Rex.configuration.when_matching_filter
          unless when_matching.empty?
            if all_examples.any? { |example| location_allows?(example) && description_allows?(example) && Metadata.filter_applies_to_any?(when_matching, example.metadata) }
              filter.update(when_matching)
            end
          end
          if !filter.empty? && Rex.configuration.run_all_when_everything_filtered &&
             all_examples.none? { |example| Metadata.filter_applies_to_any?(filter, example.metadata) }
            filter = {}
          end
          filter
        end
      end

      def location_allows?(example)
        return true if @location_filters.empty?

        lines = @location_filters[File.expand_path(example.file_path)]
        return true if lines.nil?

        lines.any? { |line| declaration_for(example.file_path, line)&.then { |found| covers?(found, example) } }
      end

      def id_allows?(example)
        return true if @id_filters.nil?

        ids = @id_filters[File.expand_path(example.file_path)]
        return true if ids.nil?

        ids.any? { |id| example.scoped_id == id || example.scoped_id.start_with?("#{id}:") }
      end

      def description_allows?(example)
        return true if @description_filters.empty?

        @description_filters.any? do |pattern|
          pattern.is_a?(Regexp) ? pattern.match?(example.full_description) : example.full_description.include?(pattern)
        end
      end

      # The example or group written at or nearest above `line` in `path`.
      def declaration_for(path, line)
        expanded = File.expand_path(path)
        candidates = []
        @example_groups.flat_map(&:descendants).each do |group|
          candidates << group if File.expand_path(group.file_path) == expanded && group.declaration_line <= line
          group.examples.each do |example|
            candidates << example if File.expand_path(example.file_path) == expanded && example.line_number <= line
          end
        end
        candidates.max_by { |candidate| candidate.is_a?(Example) ? candidate.line_number : candidate.declaration_line }
      end

      def covers?(declaration, example)
        if declaration.is_a?(Example)
          declaration.equal?(example)
        else
          example.example_group.parent_groups.include?(declaration)
        end
      end

      # A filter as RSpec announces it: a Proc without its address, with the
      # project directory written as `.`, and without ` (lambda)`.
      def describe_filter(filter)
        filter.inspect.gsub(/0x[0-9a-f]+@?/, "").gsub(File.expand_path("."), ".").gsub(" (lambda)", "")
      end

      def announce_filters
        announcements = []
        inclusion = inclusion_filter.dup
        inclusion[:locations] = @location_filters.transform_keys { |path| Rex::Core.relative_path(path) } unless @location_filters.empty?
        inclusion[:full_description] = Regexp.union(*@description_filters) unless @description_filters.empty?
        announcements << "include #{describe_filter(inclusion)}" unless inclusion.empty?
        exclusion = Rex.configuration.exclusion_filter
        announcements << "exclude #{describe_filter(exclusion)}" unless exclusion.empty?
        if !announcements.empty? && !Rex.configuration.silence_filter_announcements
          Rex.configuration.output_stream.puts "Run options: #{announcements.join(' ')}"
        end
        if example_count.zero?
          message = if announcements.empty?
                      "No examples found."
                    else
                      "\nAll examples were filtered out"
                    end
          Rex.configuration.output_stream.puts message
        end
      end
    end

    class World
      def ordered_example_groups
        Rex.configuration.ordering_registry.fetch(:global).order(example_groups)
      end
    end

    module Ordering
      # The order the examples and groups were defined in.
      class Identity
        def order(items)
          items
        end
      end

      # Sorts by a Jenkins hash of the seed and each item's id, so one seed
      # gives RSpec's order for the same suite.
      class Random
        MAX_32_BIT = 4_294_967_295

        def initialize(configuration)
          @configuration = configuration
          @used = false
        end

        def used?
          @used
        end

        def order(items)
          @used = true
          seed = @configuration.seed.to_s
          items.sort_by { |item| jenkins_hash_digest(seed + item.id) }
        end

        private

        def jenkins_hash_digest(string)
          hash = 0
          string.each_byte do |byte|
            hash += byte
            hash &= MAX_32_BIT
            hash += ((hash << 10) & MAX_32_BIT)
            hash &= MAX_32_BIT
            hash ^= hash >> 6
          end
          hash += ((hash << 3) & MAX_32_BIT)
          hash &= MAX_32_BIT
          hash ^= hash >> 11
          hash += ((hash << 15) & MAX_32_BIT)
          hash & MAX_32_BIT
        end
      end

      # The most recently modified file first.
      class RecentlyModified
        def order(items)
          items.sort_by { |item| -File.mtime(item.metadata[:absolute_file_path]).to_i }
        end
      end

      class Custom
        def initialize(callable)
          @callable = callable
        end

        def order(items)
          @callable.call(items)
        end
      end

      # An ordering named before it is registered, looked up when it is used.
      class Delayed
        def initialize(registry, name)
          @registry = registry
          @name = name
        end

        def order(items)
          strategy.order(items)
        end

        private

        def strategy
          @strategy ||= begin
            raise "Undefined ordering strategy #{@name.inspect}" unless @registry.has_strategy?(@name)

            @registry.fetch(@name)
          end
        end
      end

      class Registry
        def initialize(manager)
          @strategies = {}
          register(:random, Random.new(manager))
          register(:recently_modified, RecentlyModified.new)
          identity = Identity.new
          register(:defined, identity)
          register(:global, identity)
        end

        def fetch(name, &fallback)
          @strategies.fetch(name, &fallback)
        end

        def has_strategy?(name)
          @strategies.key?(name)
        end

        def register(name, strategy)
          @strategies[name] = strategy
        end

        def used_random_seed?
          @strategies[:random].used?
        end
      end

      # The seed and the global ordering. A value forced from the command
      # line wins over one the configuration sets later.
      class ConfigurationManager
        attr_reader :seed, :ordering_registry

        def initialize
          @ordering_registry = Registry.new(self)
          @seed = rand(0xFFFF)
          @seed_forced = false
          @order_forced = false
        end

        def seed_used?
          ordering_registry.used_random_seed?
        end

        def seed=(seed)
          return if @seed_forced

          register_ordering(:global, ordering_registry.fetch(:random))
          @seed = seed.to_i
        end

        def order=(type)
          order, seed = type.to_s.split(":")
          @seed = seed.to_i if seed
          ordering_name = if order.include?("rand")
                            :random
                          elsif order == "defined"
                            :defined
                          elsif order == "recently-modified"
                            :recently_modified
                          else
                            order.to_sym
                          end
          strategy = if ordering_registry.has_strategy?(ordering_name)
                       ordering_registry.fetch(ordering_name)
                     else
                       Delayed.new(ordering_registry, ordering_name)
                     end
          register_ordering(:global, strategy)
        end

        def force(hash)
          if hash.key?(:seed)
            self.seed = hash[:seed]
            @seed_forced = true
            @order_forced = true
          elsif hash.key?(:order)
            self.order = hash[:order]
            @order_forced = true
          end
        end

        def register_ordering(name, strategy = Custom.new(proc { |list| yield list }))
          return if @order_forced && name == :global

          ordering_registry.register(name, strategy)
        end
      end
    end
  end

  def self.world
    @world ||= Core::World.new
  end

  def self.world=(world)
    @world = world
  end
end
