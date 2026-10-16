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
