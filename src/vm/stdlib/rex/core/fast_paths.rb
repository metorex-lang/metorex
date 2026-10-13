module Rex
  module Core
    # The guards Rex's fast paths run behind. A fast path stands in for a
    # group of Rex's own methods, and runs only while `config.fast_paths` is
    # on, which the fast path checks, and each of those methods still
    # answers on the objects it reads as it did when Rex loaded. A redefined method, or one stubbed on a single
    # object, sends the work back through the methods themselves.
    module FastPaths
      # Per example group, the hooks of the group and the groups around it,
      # flattened in the order they run, which `Hooks::HookCollections`
      # builds.
      FLATTENED_HOOKS = {}.compare_by_identity

      # A guard over the methods named `names` of `owner`.
      class Guard
        def initialize(owner, names)
          @originals = names.to_h { |name| [name, owner.instance_method(name)] }
          # Per object checked: the method state the check ran under and
          # what it answered. Held aside, so the objects carry nothing a
          # fast path added.
          @checked = {}.compare_by_identity
        end

        # Whether each of `receivers` still answers the guarded methods with
        # Rex's own.
        def holds?(receivers)
          state = __method_state__
          receivers.all? { |receiver| answers_as_rex?(receiver, state) }
        end

        private

        def answers_as_rex?(receiver, state)
          checked_under, held = @checked[receiver]
          return held if checked_under == state

          held = @originals.all? { |name, original| receiver.method(name).unbind == original }
          @checked[receiver] = [state, held]
          held
        end
      end

      # The methods flattened hooks stand in for: each group's own walk
      # through its parents' hooks, as Rex loaded them.
      HOOKS_GUARD = Guard.new(
        Hooks::HookCollections,
        %i[run_owned_hooks_for matching_hooks_for owner_parent_groups example_hook_entries]
      )

      def self.hooks_guard
        HOOKS_GUARD
      end

      # The methods `let` reads past while a value is stored: the example's
      # memo and the memo's lookup, as Rex defined them.
      __guard_fast_path__(MemoizedHelpers, :__memoized)
      __guard_fast_path__(MemoizedHelpers::ThreadsafeMemoized, :fetch_or_store)
      __guard_fast_path__(MemoizedHelpers::NonThreadSafeMemoized, :fetch_or_store)

      # The steps of filling an example's metadata, which
      # `__populate_example_metadata__` takes in one go, as Rex defined them.
      %i[populate ensure_valid_user_keys populate_location_attributes build_description_from
         description_separator build_scoped_id_for metadata user_metadata block
         description_args].each do |name|
        __guard_fast_path__(Metadata::HashPopulator, name)
      end
      %i[full_description described_class].each do |name|
        __guard_fast_path__(Metadata::ExampleHash, name)
      end

      # The initializers `__new_example_group__` stands in for when it builds
      # an example group instance, as Rex defined them.
      __guard_fast_path__(ExampleGroup, :initialize)
      __guard_fast_path__(MemoizedHelpers, :initialize)
      __guard_fast_path__(MemoizedHelpers, :__init_memoized)
      __guard_fast_path__(MemoizedHelpers::ThreadsafeMemoized, :initialize)
      __guard_fast_path__(MemoizedHelpers::NonThreadSafeMemoized, :initialize)
      __guard_fast_path__(Support::ReentrantMutex, :initialize)

      # The matchers `__expect_to__` matches natively, the methods of each it
      # stands in for, and the steps of `to` it reads past, as Rex defined
      # them.
      built_in = Rex::Matchers::BuiltIn
      __guard_fast_matcher__(:eq, built_in::Eq, :matches?, :match, :expected, :supports_value_expectations?)
      __guard_fast_matcher__(:equal, built_in::Equal, :matches?, :match, :expected, :supports_value_expectations?)
      __guard_fast_matcher__(:be_truthy, built_in::BeTruthy, :matches?, :match, :supports_value_expectations?)
      __guard_fast_matcher__(:be_nil, built_in::BeNil, :matches?, :match, :supports_value_expectations?)
      __guard_fast_matcher__(:match, built_in::Match, :matches?, :match, :expected, :values_match?,
                             :supports_value_expectations?)
      __guard_fast_matcher__(:include, built_in::Include, :matches?, :check_actual?, :check_expected_count?,
                             :perform_match, :excluded_from_actual, :actual_collection_includes?,
                             :supports_value_expectations?)
      __guard_fast_path__(Rex::Expectations::ExpectationTarget, :enforce_value_expectation)
      __guard_fast_path__(Rex::Expectations::ExpectationTarget, :with_source_location)
      __guard_fast_path_class_method__(Rex::Expectations::PositiveExpectationHandler, :handle_matcher)
      __guard_fast_path_class_method__(Rex::Expectations::PositiveExpectationHandler, :verb)
      __guard_fast_path_class_method__(Rex::Expectations::ExpectationHelper, :check_message)
      __guard_fast_path_class_method__(Rex::Matchers, :last_matcher=)
      __guard_fast_path_class_method__(Rex::Matchers, :last_expectation_verb=)

      # What verifying and taking down an example's mock space walks, which
      # `__mock_space_idle__` lets an example with no doubles skip.
      __guard_fast_path__(Rex::Mocks::Space, :verify_all)
      __guard_fast_path__(Rex::Mocks::Space, :reset_all)
    end
  end
end
