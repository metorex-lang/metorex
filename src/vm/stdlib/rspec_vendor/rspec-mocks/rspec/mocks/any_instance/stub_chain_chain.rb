module Rex
  module Mocks
    module AnyInstance
      # @private
      class StubChainChain < StubChain
        def initialize(*args)
          super
          @expectation_fulfilled = false
        end

      private

        def create_message_expectation_on(instance)
          ::Rex::Mocks::StubChain.stub_chain_on(instance, *@expectation_args, &@expectation_block)
        end

        def invocation_order
          EmptyInvocationOrder
        end
      end
    end
  end
end
