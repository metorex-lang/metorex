// The numeric classes and the arithmetic they answer.

use super::super::run_example;
use super::*;
#[test]
fn test_operators_coverage_execution() {
    let output = run_example("builtins/operators_coverage.rb");
    assert_eq!(
        output,
        "9223372036854775806\n13835058055282163709\n-9223372036854775808\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\n1\n7\n6\ntrue\nfalse\ntrue\ntrue\ntrue\ntrue\n"
    );
}

#[test]
fn test_operators_coverage_no_parens_execution() {
    let output = run_example("builtins/operators_coverage_no_parens.rb");
    assert_eq!(
        output,
        "9223372036854775806\n13835058055282163709\n-9223372036854775808\ntrue\ntrue\ntrue\ntrue\ntrue\ntrue\n1\n7\n6\n"
    );
}

#[test]
fn test_integer_iteration_execution() {
    // `upto` and `downto` answer an Enumerator without a block, so the example
    // collects it before inspecting.
    let expected = "1\n2\n3\n3\n2\n1\n[1, 2, 3]\n[3, 2, 1]\n3/2\n1\n1.5\n3\n2\n2\n2/1\n";
    let output = run_example("builtins/integer_iteration.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_rational_execution() {
    let expected = "1/2\n1\n2\n1/2\n3/5\n7/1\n1/3\n1/2\n5/1\n3/2\n5/6\n1/3\n1/4\n2/1\ntrue\n0.5\n2\n(1/2)\ntrue\n13/25\n13/15\n3/4\n3/5\n3/1\n1/2\ntrue\ntrue\nfalse\ndivided by 0\ncan't convert nil into Rational\nnil\n";
    let output = run_example("builtins/rational.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_integer_bits_operations_execution() {
    let expected = concat!(
        "[8, 34359738368, 2, -2]\n",
        "[2, 10]\n",
        "[1606938044258990275541962092341162602522202993782792835301376, -1, 0]\n",
        "[-6, -1, 0]\n",
        "[8, 9, 0, 0]\n",
        "[1, 3, 2]\n",
        "42\n",
        "true\n"
    );
    let output = run_example("builtins/integer_bits/operations.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_integer_bits_operations_parens_execution() {
    let expected = concat!(
        "[8, 34359738368, 2, -2]\n",
        "[2, 10]\n",
        "[1606938044258990275541962092341162602522202993782792835301376, -1, 0]\n",
        "[-6, -1, 0]\n",
        "[8, 9, 0, 0]\n",
        "[1, 3, 2]\n",
        "42\n",
        "true\n"
    );
    let output = run_example("builtins/integer_bits/operations_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn one_bit_of_a_machine_word_integer_reads_its_twos_complement_form() {
    let expected = concat!(
        "[1, 0, 1, 0]\n",
        "[0, 1, 0, 1, 1, 1, 1]\n",
        "[1, 0, 0]\n",
        "[0, 0]\n",
        "[0, 1]\n",
        "[:written, 0]\n"
    );
    let output = run_example("builtins/integer_bits/single_bit.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_big_integers_execution() {
    let expected = concat!(
        "[18446744073709551616, 1267650600228229401496703205376]\n",
        "Integer\n",
        "true\n",
        "[9223372036854775808, 18446744073709551614, -9223372036854775809]\n",
        "42\n",
        "0\n",
        "Integer\n",
        "true\n",
        "1\n",
        "[-18446744073709551616, 1, 3, 18446744073709551616]\n",
        "18446744073709551616\n",
        "18446744073709551616\n",
        "true\n",
        "18446744073709551617\n",
        "65\n",
        "[18446744073709551, 616]\n",
        "4294967296\n",
        "-18446744073709551617\n",
        "Integer\n",
        "340282366920938463463374607431768211456\n",
        "18446744073709551616\n",
        "3\n",
        "true\n",
        "false\n",
        "true\n"
    );
    let output = run_example("builtins/big_integers/arithmetic.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_big_integers_parens_execution() {
    let expected = concat!(
        "[18446744073709551616, 1267650600228229401496703205376]\n",
        "Integer\n",
        "true\n",
        "[9223372036854775808, 18446744073709551614, -9223372036854775809]\n",
        "42\n",
        "0\n",
        "Integer\n",
        "true\n",
        "1\n",
        "[-18446744073709551616, 1, 3, 18446744073709551616]\n",
        "18446744073709551616\n",
        "18446744073709551616\n",
        "true\n",
        "18446744073709551617\n",
        "65\n",
        "[18446744073709551, 616]\n",
        "4294967296\n",
        "-18446744073709551617\n",
        "Integer\n",
        "340282366920938463463374607431768211456\n",
        "18446744073709551616\n",
        "3\n",
        "true\n",
        "false\n",
        "true\n"
    );
    let output = run_example("builtins/big_integers/arithmetic_parens.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_builtins_numeric_parts_execution() {
    let output = run_example("builtins/numeric_parts.rb");
    assert_eq!(output, NUMERIC_PARTS_OUTPUT);
}

#[test]
fn test_builtins_numeric_parts_no_parens_execution() {
    let output = run_example("builtins/numeric_parts_no_parens.rb");
    assert_eq!(output, NUMERIC_PARTS_OUTPUT);
}

#[test]
fn test_builtins_comparing_values_execution() {
    let output = run_example("builtins/comparing_values.rb");
    assert_eq!(output, COMPARING_VALUES_OUTPUT);
}

#[test]
fn test_builtins_comparing_values_no_parens_execution() {
    let output = run_example("builtins/comparing_values_no_parens.rb");
    assert_eq!(output, COMPARING_VALUES_OUTPUT);
}

#[test]
fn test_builtins_optimized_redefinition_execution() {
    let output = run_example("builtins/optimized_redefinition.rb");
    assert_eq!(output, OPTIMIZED_REDEFINITION_OUTPUT);
}

#[test]
fn test_builtins_optimized_redefinition_parens_execution() {
    let output = run_example("builtins/optimized_redefinition_parens.rb");
    assert_eq!(output, OPTIMIZED_REDEFINITION_OUTPUT);
}

#[test]
fn test_builtins_complex_powers_execution() {
    let output = run_example("builtins/complex_powers.rb");
    assert_eq!(output, COMPLEX_POWERS_OUTPUT);
}

#[test]
fn test_builtins_complex_powers_no_parens_execution() {
    let output = run_example("builtins/complex_powers_no_parens.rb");
    assert_eq!(output, COMPLEX_POWERS_OUTPUT);
}

#[test]
fn test_builtins_numbers_written_out_execution() {
    let output = run_example("builtins/numbers_written_out.rb");
    assert_eq!(output, NUMBERS_WRITTEN_OUT_OUTPUT);
}

#[test]
fn test_builtins_numbers_written_out_no_parens_execution() {
    let output = run_example("builtins/numbers_written_out_no_parens.rb");
    assert_eq!(output, NUMBERS_WRITTEN_OUT_OUTPUT);
}

/// The expected output of both `builtins/operators_by_name` variants.
const OPERATORS_BY_NAME: &str = concat!(
    "Integer +: TypeError: nil can't be coerced into Integer\n",
    "Integer <<: TypeError: no implicit conversion of nil into Integer\n",
    "Integer &: TypeError: true can't be coerced into Integer\n",
    "Integer []: TypeError: no implicit conversion of nil into Integer\n",
    "Float &: NoMethodError: undefined method '&' for an instance of Float\n",
    "Float <: ArgumentError: comparison of Float with :sym failed\n",
    "Rational +: TypeError: :sym can't be coerced into Rational\n",
    "Rational <: ArgumentError: comparison of Rational with String failed\n",
    "Rational |: NoMethodError: undefined method '|' for an instance of Rational\n",
    "Complex +: TypeError: String can't be coerced into Complex\n",
    "Complex **: TypeError: Object can't be coerced into Complex\n",
    "Complex <: NoMethodError: undefined method '<' for an instance of Complex\n",
    "Complex %: NoMethodError: undefined method '%' for an instance of Complex\n",
    "String +: TypeError: no implicit conversion of nil into String\n",
    "String *: TypeError: no implicit conversion from nil to integer\n",
    "String *: TypeError: no implicit conversion of String into Integer\n",
    "String []: TypeError: no implicit conversion of Symbol into Integer\n",
    "String <<: TypeError: no implicit conversion of true into String\n",
    "String =~: TypeError: type mismatch: String given\n",
    "String =~: NoMethodError: undefined method '=~' for an instance of Array\n",
    "String &: NoMethodError: undefined method '&' for an instance of String\n",
    "Array +: TypeError: no implicit conversion of nil into Array\n",
    "Array -: TypeError: no implicit conversion of String into Array\n",
    "Array &: TypeError: no implicit conversion of true into Array\n",
    "Array *: TypeError: no implicit conversion from nil to integer\n",
    "Array ^: NoMethodError: undefined method '^' for an instance of Array\n",
    "Hash <: TypeError: no implicit conversion of nil into Hash\n",
    "Hash +: NoMethodError: undefined method '+' for an instance of Hash\n",
    "Symbol <: false\n",
    "Symbol <: ArgumentError: comparison of Symbol with 1 failed\n",
    "Symbol -@: NoMethodError: undefined method '-@' for an instance of Symbol\n",
    "Symbol []: TypeError: no implicit conversion from nil to integer\n",
    "NilClass +: NoMethodError: undefined method '+' for nil\n",
    "TrueClass <: NoMethodError: undefined method '<' for true\n",
    "Object -: NoMethodError: undefined method '-' for an instance of Object\n",
    "Range +: NoMethodError: undefined method '+' for an instance of Range\n",
    "Time -: TypeError: can't convert String into an exact number\n",
    "Integer =~: NoMethodError: undefined method '=~' for an instance of Integer\n",
    "[false, false, false, false, true, false, false]\n",
);

#[test]
fn test_builtins_operators_by_name_execution() {
    let output = run_example("builtins/operators_by_name.rb");
    assert_eq!(output, OPERATORS_BY_NAME);
}

#[test]
fn test_builtins_operators_by_name_no_parens_execution() {
    let output = run_example("builtins/operators_by_name_no_parens.rb");
    assert_eq!(output, OPERATORS_BY_NAME);
}

/// The expected output of both `builtins/numeric_coerce` variants.
const NUMERIC_COERCE: &str = concat!(
    "[3.0, 2.0]\n",
    "[4.0, 2.0]\n",
    "can't define singleton method \"unit\" for Measure\n",
    "[3.0, 2.0]\n",
    "true\n",
);

#[test]
fn test_builtins_numeric_coerce_execution() {
    let output = run_example("builtins/numeric_coerce.rb");
    assert_eq!(output, NUMERIC_COERCE);
}

#[test]
fn test_builtins_numeric_coerce_no_parens_execution() {
    let output = run_example("builtins/numeric_coerce_no_parens.rb");
    assert_eq!(output, NUMERIC_COERCE);
}

/// The expected output of both `builtins/numeric_edges` variants.
const NUMERIC_EDGES_OUTPUT: &str = "true\nfalse\n\"not a real\"\n(0+0.0i)\n(-2+0.0i)\n\"bignum too big to convert into 'long'\"\n\"bignum too big to convert into 'long'\"\ntrue\nfalse\nFrozenError\n";

#[test]
fn test_builtins_numeric_edges_execution() {
    let output = run_example("builtins/numeric_edges.rb");
    assert_eq!(output, NUMERIC_EDGES_OUTPUT);
}

#[test]
fn test_builtins_numeric_edges_no_parens_execution() {
    let output = run_example("builtins/numeric_edges_no_parens.rb");
    assert_eq!(output, NUMERIC_EDGES_OUTPUT);
}

#[test]
fn math_error_and_gamma_functions_answer_the_c_library_digits() {
    let output = run_example("builtins/math_error_function_digits.rb");
    assert_eq!(
        output,
        concat!(
            "0.32862675945912734\n3.0577097964381646e-06\n0.0029794666563329845\n-0.0\n",
            "11.631728396567448\n9.51350769866873\n",
            "[13.940625219403763, 1]\n[-0.05624371649767412, -1]\n",
        )
    );
}

#[test]
fn float_divmod_floors_toward_the_divisor_sign_with_infinite_divisors() {
    let output = run_example("builtins/float_divmod_by_infinity.rb");
    assert_eq!(
        output,
        concat!(
            "[-1, Infinity]\n[0, -4.2]\n[0, 0.0]\n[0, 0.0]\n[0, 4.2]\n[-1, -Infinity]\n",
            "[0, -0.0]\n[0, -0.0]\n[3, 1.5]\n[-4, 0.5]\n[-4, -0.5]\n[3, 0.10000000000000003]\n",
            "[0, 6.0]\n\"Infinity\"\n\"NaN\"\n",
        )
    );
}

#[test]
fn integer_quo_divides_a_rational_by_any_number_and_floats_meet_rationals_as_floats() {
    let output = run_example("builtins/integer_quo_by_any_number.rb");
    assert_eq!(
        output,
        concat!(
            "(1/2)\n(0/1)\n(3/590295810358705651712)\n((3/2)-(3/2)*i)\n(7/2)\n3.5\n",
            "\"divided by 0\"\n\"String can't be coerced into Rational\"\n",
            "(1.5+0i)\n((3/2)+0i)\n(6.0+0i)\n(1.5+0i)\n(1.5+(1/4)*i)\n",
            "((3/2)+1i)\n((1/4)-(1/4)*i)\n((3/2)-(3/2)*i)\n(1-(1/4)*i)\n",
            "1.5\n6.0\n5.0\n1.0\n4.0\n",
        )
    );
}

#[test]
fn a_negative_base_raised_by_a_half_turn_answers_an_exact_zero_real_part() {
    let output = run_example("builtins/negative_base_fractional_power.rb");
    assert_eq!(
        output,
        concat!(
            "(0.0+2.8284271247461903i)\n(0.0-22.627416997969522i)\n(0.0-0.3535533905932738i)\n",
            "(0.0+32.0i)\n(0.0+2.0i)\n(0.0+2.0i)\n(0.0+34359738368.0i)\n",
        )
    );
}

#[test]
fn small_float_to_s_uses_an_exponent() {
    let output = run_example("numbers/small_float_to_s_uses_an_exponent.rb");
    assert_eq!(
        output,
        concat!(
            "\"1.8156028367527268e-06\"\n",
            "\"2.35e-06\"\n",
            "\"0.0001\"\n",
            "\"1.0e-05\"\n",
            "\"1.0e+16\"\n",
            "\"1.0e+15\"\n",
            "-6\n",
            "true\n",
        )
    );
}

#[test]
fn test_upto_and_downto_walk_past_the_machine_word() {
    let output = run_example("numbers/upto_and_downto_past_the_machine_word.rb");
    assert_eq!(
        output,
        concat!(
            "9223372036854775806 9223372036854775807 9223372036854775808 9223372036854775809 \n",
            "-9223372036854775807 -9223372036854775808 -9223372036854775809 \n",
            "18446744073709551616 18446744073709551615 \n",
            "[18446744073709551615, 18446744073709551616]\n",
            "2\n",
            "4\n",
            "0\n",
            "[18446744073709551617, 18446744073709551616]\n",
            "18446744073709551618\n",
        )
    );
}
