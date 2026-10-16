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
