use metorex::lexer::Lexer;
use metorex::parser::Parser;

/// A program of many methods, each with a block that reads back over its
/// own tokens, long enough that the parser passes several chunks of tokens.
fn many_methods() -> String {
    (0..2_000)
        .map(|number| {
            format!("def method_{number}(value)\n  [value].map {{ it + {number} }}\nend\n")
        })
        .collect()
}

#[test]
fn a_parser_reading_from_the_lexer_reads_what_one_given_every_token_reads() {
    let source = many_methods();

    let streamed = Parser::lexing(Lexer::new(&source))
        .parse()
        .expect("the program parses");

    assert_eq!(
        Some(streamed),
        Parser::new(Lexer::new(&source).tokenize()).parse().ok()
    );
}

#[test]
fn a_parser_reading_from_the_lexer_reports_an_error_past_the_first_chunk() {
    let source = format!("{}def broken(\n", many_methods());

    assert!(Parser::lexing(Lexer::new(&source)).parse().is_err());
}
