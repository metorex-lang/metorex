// Examples of RubyVM::InstructionSequence, which compiles through a parser
// written in Ruby. They run in a test binary of their own so no one binary
// takes long enough under coverage to reach tarpaulin's time limit.

use super::run_example;

/// The expected output of both `ruby_vm/instruction_sequences` variants.
const INSTRUCTION_SEQUENCES_OUTPUT: &str = concat!(
    "<RubyVM::InstructionSequence:<compiled>@<compiled>:1>\n",
    "[\"<compiled>\", \"<compiled>\", \"<compiled>\", \"<compiled>\", 1]\n",
    "21\n",
    "[[1, :line], [2, :line]]\n",
    "== disasm: #<ISeq:<compiled>@<compiled>:1 (1,0)-(2,9)>\n",
    "0000 LoadInteger to: 0, value: 4            (   1)[Li]\n",
    "0001 LoadInteger to: 1, value: 5            (   1)\n",
    "0002 Multiply to: 0, left: 0, right: 1      (   1)\n",
    "0003 SetLocal from: 0, slot: 0  ; total     (   1)\n",
    "0004 GetLocal to: 0, slot: 0  ; total       (   2)[Li]\n",
    "0005 LoadInteger to: 1, value: 1            (   2)\n",
    "0006 Add to: 0, left: 0, right: 1           (   2)\n",
    "0007 Return from: 0                         (   2)\n",
    "[1, :RUBY_EVENT_LINE, [:LoadInteger, 0, 4], [:LoadInteger, 1, 5], [:Multiply, 0, 0, 1], [:SetLocal, 0, 0], 2, :RUBY_EVENT_LINE, [:GetLocal, 0, 0], [:LoadInteger, 1, 1], [:Add, 0, 0, 1], [:Return, 0]]\n",
    "[\"<main>\", true, true, 1]\n",
    "[<RubyVM::InstructionSequence:<compiled>@report.rb:12>, \"report.rb\", \"/reports/report.rb\", 12, 42]\n",
    "[\"area\", \"area\", 19, true]\n",
    "0000 GetIdentifier to: 0, name: 0  ; width  (  19)[Li]\n",
    "0001 GetIdentifier to: 1, name: 1  ; height (  19)\n",
    "0002 Multiply to: 0, left: 0, right: 1      (  19)\n",
    "0003 Return from: 0                         (  19)\n",
    "\"Not a toplevel InstructionSequence\"\n",
    "\"block in <main>\"\n",
    "\"block (2 levels) in builder\"\n",
    "\"block in <module:Reports>\"\n",
    "nil\n",
    "[[\"a\", 1], [\"<class:B>\", 2], [\"block in <compiled>\", 3]]\n",
    "21\n",
    "\"<compiled>:1: syntax error, unexpected end-of-input\"\n",
    "\"<compiled>:1: syntax error, unexpected end-of-input\\ntotal = price *\\n               ^\\n\"\n",
    "\"loop.rb:1: Invalid break\\nbreak if ready\\n^~~~~\\n\"\n",
    "\"allocator undefined for RubyVM::InstructionSequence\"\n",
    "0\n",
);

#[test]
fn test_ruby_vm_instruction_sequences_execution() {
    let output = run_example("ruby_vm/instruction_sequences.rb");
    assert_eq!(output, INSTRUCTION_SEQUENCES_OUTPUT);
}

#[test]
fn test_ruby_vm_instruction_sequences_no_parens_execution() {
    let output = run_example("ruby_vm/instruction_sequences_no_parens.rb");
    assert_eq!(output, INSTRUCTION_SEQUENCES_OUTPUT);
}

#[test]
fn an_instruction_sequence_lists_the_patterns_its_code_holds() {
    let output = run_example("ruby_vm/instruction_sequence_lists_its_patterns.rb");
    assert_eq!(
        output,
        "[:check, [/\\AAUTH=/i]]\n[:labeled, [/\\ABODY\\[/in]]\n[:as_value, [/\\ABINARY\\[/in]]\n[:words, [/ /]]\n[:args, [/\\ACHARSET\\b/i, /\\ACHARSET\\z/i]]\n"
    );
}
