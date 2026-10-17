use super::run_example;

#[test]
fn test_bytecode_literals_and_names_execution() {
    let output = run_example("bytecode/literals_and_names.rb");
    assert_eq!(
        output,
        concat!(
            "[1, 2.5, 123456789012345678901234567890, nil, true, false, :name, \"text\", /a+b/i, 1..3, 1...3, 19]\n",
            "[3, 3, 2, true, true]\n",
            "{a: 1, \"b\" => 2, c: 3}\n",
            "\"2 items, symbol [1, 2] 1.5\"\n",
            "[4, 5, 6]\n",
        )
    );
}

#[test]
fn test_bytecode_hash_key_order_execution() {
    let output = run_example("bytecode/hash_key_order.rb");
    assert_eq!(
        output,
        "[\"value 1\", \"value 2\", \"hash first\", \"hash second\"]\n"
    );
}

#[test]
fn test_bytecode_assignments_execution() {
    let output = run_example("bytecode/assignments.rb");
    assert_eq!(
        output,
        concat!(
            "[1, :replaced, false, 1, 2, 7, 5, 1024]\n",
            "[2, 3]\n",
            "[6, \"first\", [:a, :b, 3], {count: 1}, nil, 3, 1]\n",
            "[3, 6, 4, 1, 2]\n",
            "[2, 1, 1, [2, 3], 4, 5, 6, [7, 8], 9, 10, \"ten\", [:x, :y], [3, 4], 3, 4]\n",
            "[5, 5, 7, 6]\n",
        )
    );
}

#[test]
fn test_bytecode_constant_or_assignments_execution() {
    let output = run_example("bytecode/constant_or_assignments.rb");
    assert_eq!(output, "[3, 6, 4, 5, 1]\n[9, 8]\n");
}

#[test]
fn test_bytecode_calls_execution() {
    let output = run_example("bytecode/calls.rb");
    assert_eq!(
        output,
        concat!(
            "[[1, :default, [], :none, {}, :no_block], [1, 2, [3, 4], :none, {}, :no_block], [1, :default, [], :named, {}, :no_block], [1, :default, [], :given, {extra: 1}, :no_block], [1, 2, [], :none, {}, 10], [1, :default, [], :none, {}, \"1\"]]\n",
            "[[1, 2], :finished, 100, [2, 3, 4]]\n",
            "[nil, 4, nil]\n",
            "11\n",
            "[12, 1.5, \"007\", [], 0]\n",
            "[:negated, :positive, 3, true, 8, 1, true]\n",
            "[true, 3, true, false]\n",
            "[\"constant\", nil, \"method\", nil]\n",
        )
    );
}

#[test]
fn test_bytecode_spread_subscripts_execution() {
    let output = run_example("bytecode/spread_subscripts.rb");
    assert_eq!(
        output,
        "[[2, 3], [1, :replaced, 4], :pair]\n[1, :both, 4, 5]\n"
    );
}

#[test]
fn test_bytecode_branches_and_loops_execution() {
    let output = run_example("bytecode/branches_and_loops.rb");
    assert_eq!(
        output,
        concat!(
            "[:large, :even, :positive]\n",
            "[:medium, :odd, :positive]\n",
            "[:small, :odd, :positive]\n",
            "[[2, 4, 6], 0, 3, :carried]\n",
            "[2, 20]\n",
            "[6, 3]\n",
            "[:integer, :letter, :range, :other]\n",
            "[[2, 0, 6, 8], 6]\n",
            "[:matched, 2, :right]\n",
        )
    );
}

#[test]
fn test_bytecode_blocks_and_returns_execution() {
    let output = run_example("bytecode/blocks_and_returns.rb");
    assert_eq!(
        output,
        concat!(
            "[[1, nil], [2, 3]]\n",
            "3\n",
            "\"no block given (yield)\"\n",
            "[:negative, nil]\n",
            "20\n",
            "[8, :after]\n",
            "[[3, 6], [5, 6]]\n",
        )
    );
}

#[test]
fn test_bytecode_definitions_execution() {
    let output = run_example("bytecode/definitions.rb");
    assert_eq!(
        output,
        concat!(
            "[\"you\", [], 1, {}, nil]\n",
            "[\"ann\", [1], 2, {extra: 3}, :block]\n",
            "\"hi!\"\n",
            "[\"Plain\", \"Outer::Inner\", 3, Struct]\n",
            "[:spoken, :twin, :later]\n",
        )
    );
}

#[test]
fn test_bytecode_supers_execution() {
    let output = run_example("bytecode/supers.rb");
    assert_eq!(
        output,
        concat!(
            "[10, 10, \"HELLO ANN.\", \"!nna olleh\"]\n",
            "[[[1, 2, 9], :given], [[], :none]]\n",
            ":broken\n",
        )
    );
}

#[test]
fn test_bytecode_exceptions_execution() {
    let output = run_example("bytecode/exceptions.rb");
    assert_eq!(
        output,
        concat!(
            "[:splatted, [:body, Custom, :ensure], nil]\n",
            "[:listed, [:body, \"argument\", :ensure], nil]\n",
            "[:listed, [:body, \"invalid value for Integer(): \\\"x\\\"\", :ensure], nil]\n",
            "[:splatted, [:body, KeyError, :ensure], nil]\n",
            "[:else_value, [:body, :else, :ensure], nil]\n",
            "3\n",
            "[\"ensure 1\", 2, \"ensure 2\", \"ensure 3\", :loop_ensure]\n",
            "[:returned, [:ensured]]\n",
            "[:method_level, \"passes\"]\n",
            "[\"second\", \"first\"]\n",
            "20\n",
            "nil\n",
        )
    );
}

#[test]
fn test_bytecode_pattern_matching_execution() {
    let output = run_example("bytecode/pattern_matching.rb");
    assert_eq!(
        output,
        concat!(
            "[:number, :number, :early, :zed, :other]\n",
            "[:array, 3, [4, 5]]\n",
            "[:array, 1, [:needle, :after]]\n",
            "[:find, [:after]]\n",
            "[:hash, \"ann\"]\n",
            "[:array, 1, [2]]\n",
            "[:alternative, :two]\n",
            ":nothing\n",
            "[NoMatchingPatternError, \"7: String === 7 does not return true\"]\n",
            "[:deconstruct]\n",
        )
    );
}

#[test]
fn test_bytecode_begin_blocks_execution() {
    let output = run_example("bytecode/begin_blocks.rb");
    assert_eq!(
        output,
        concat!(
            "first statement\n",
            "[:begin_one, :begin_two]\n",
            "end two\n",
            "end one\n",
        )
    );
}

#[test]
fn test_bytecode_runtime_code_execution() {
    let output = run_example("bytecode/runtime_code.rb");
    assert_eq!(
        output,
        concat!(
            "2\n",
            "[2, \"main\"]\n",
            ":left\n",
            "12\n",
            "10\n",
            "15\n",
            "30\n",
            "6\n",
            "4\n",
            "6\n",
            "20\n",
        )
    );
}

#[test]
fn test_bytecode_fibers_and_threads_execution() {
    let output = run_example("bytecode/fibers_and_threads.rb");
    assert_eq!(
        output,
        concat!(
            "2\n",
            "[1, 5, 10]\n",
            "116\n",
            "[\"ab\", \"c19\"]\n",
            "[33, 63]\n",
        )
    );
}

#[test]
fn test_bytecode_tracing_execution() {
    let output = run_example("bytecode/tracing.rb");
    assert_eq!(
        output,
        concat!(
            "[[:line, 14], [:call, 2], [:line, 3], [:b_call, 3], [:line, 4], [:line, 6], ",
            "[:return, 7], [:line, 15], [:class, 15], [:line, 18], [:raise, 18], [:line, 21]]\n",
            "[[\"call\", 2, :tally], [\"line\", 3, :tally]]\n",
            "33\n",
        )
    );
}

#[test]
fn test_bytecode_long_lists_execution() {
    let output = run_example("bytecode/long_lists.rb");
    assert_eq!(
        output,
        concat!(
            "[300, 0]\n",
            "[300, 300]\n",
            "[300, 0]\n",
            "[301, 0]\n",
            "[0, 300]\n",
            "45150\n",
            "300\n",
            "300\n",
            "300\n",
            "[1392, \"<1><2><3>\", \"><299><300>\"]\n",
            "300\n",
            "300\n",
            "45151\n",
            "300\n",
            "300\n",
        )
    );
}
