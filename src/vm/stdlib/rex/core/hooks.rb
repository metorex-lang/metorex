module Rex
  module Core
    module Hooks
      SCOPE_NAMES = {
        each: :example, example: :example,
        all: :context, context: :context,
        suite: :suite
      }.freeze

      Hook = Struct.new(:block, :metadata) do
        def applies_to?(target_metadata)
          Metadata.filter_applies?(metadata, target_metadata)
        end
      end

      # The hooks one group or the configuration holds, by kind and scope.
      class Collection
        def initialize
          @hooks = Hash.new { |hash, key| hash[key] = [] }
        end

        # `before` hooks run in the order written and `after` hooks in
        # reverse, so `before` appends and `after` prepends unless the
        # registration names the other end.
        def register(kind, scope, metadata, block, position = nil)
          scope = Hooks.scope_named(scope)
          hook = Hook.new(block, metadata)
          position ||= kind == :after ? :prepend : :append
          if position == :prepend
            @hooks[[kind, scope]].unshift(hook)
          else
            @hooks[[kind, scope]] << hook
          end
          hook
        end

        def for(kind, scope)
          @hooks[[kind, scope]]
        end

        def empty?(kind, scope)
          @hooks[[kind, scope]].empty?
        end
      end

      def self.scope_named(scope)
        scope = :each if scope.nil?
        SCOPE_NAMES.fetch(scope) do
          raise ArgumentError, "#{scope.inspect} is not a supported hook scope. Supported scopes are: :example, :each, :context, :all, :suite"
        end
      end

      # Splits a hook's arguments into its scope and its metadata, the way
      # `before(:each, :slow)` and `before(slow: true)` both read.
      def self.scope_and_metadata(arguments)
        scope = SCOPE_NAMES.key?(arguments.first) ? arguments.shift : :each
        [scope, Metadata.build_hash_from(arguments)]
      end
    end
  end
end
