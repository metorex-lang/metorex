module Rex
  module Core
    module Ordering
      class Identity
        def order(items)
          items
        end
      end

      class Random
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
          hash &= MAX_32_BIT
          hash
        end

        MAX_32_BIT = 4_294_967_295
      end

      class RecentlyModified
        def order(list)
          list.sort_by { |item| -File.mtime(item.metadata[:absolute_file_path]).to_i }
        end
      end

      class Custom
        def initialize(callable)
          @callable = callable
        end

        def order(list)
          @callable.call(list)
        end
      end

      class Delayed
        def initialize(registry, name)
          @registry = registry
          @name = name
        end

        def order(list)
          strategy.order(list)
        end

        private

        def strategy
          @strategy ||= lookup_strategy
        end

        def lookup_strategy
          raise "Undefined ordering strategy #{@name.inspect}" unless @registry.has_strategy?(@name)
          @registry.fetch(@name)
        end
      end

      class Registry
        def initialize(configuration)
          @configuration = configuration
          @strategies    = {}

          register(:random, Random.new(configuration))
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

        def register(sym, strategy)
          @strategies[sym] = strategy
        end

        def used_random_seed?
          @strategies[:random].used?
        end
      end

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
          order, seed = type.to_s.split(':')
          @seed = seed.to_i if seed

          ordering_name = if order.include?('rand')
                            :random
                          elsif order == 'defined'
                            :defined
                          elsif order == 'recently-modified'
                            :recently_modified
                          else
                            order.to_sym
                          end

          if ordering_name
            strategy =
              if ordering_registry.has_strategy?(ordering_name)
                ordering_registry.fetch(ordering_name)
              else
                Delayed.new(ordering_registry, ordering_name)
              end

            register_ordering(:global, strategy)
          end
        end

        def force(hash)
          if hash.key?(:seed)
            self.seed = hash[:seed]
            @seed_forced  = true
            @order_forced = true
          elsif hash.key?(:order)
            self.order = hash[:order]
            @order_forced = true
          end
        end

        def register_ordering(name, strategy=Custom.new(Proc.new { |l| yield l }))
          return if @order_forced && name == :global
          ordering_registry.register(name, strategy)
        end
      end
    end
  end
end
