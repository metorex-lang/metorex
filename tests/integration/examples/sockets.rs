use super::run_example;

#[test]
fn test_sockets_accept_and_echo_execution() {
    let output = run_example("sockets/accept_and_echo.rb");
    assert_eq!(output, "\"goodbye\"\n\"hello\"\n");
}

#[test]
fn test_sockets_accept_handles_execution() {
    let output = run_example("sockets/accept_handles.rb");
    assert_eq!(output, "[\"hello\", false]\n");
}
