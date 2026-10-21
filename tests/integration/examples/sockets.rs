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

#[test]
fn a_connection_with_a_time_limit_to_a_closed_port_is_refused() {
    let output = run_example("sockets/connect_timeout_refused.rb");
    assert_eq!(output, "Errno::ECONNREFUSED\nErrno::ECONNREFUSED\n");
}

#[test]
fn accept_waits_for_a_client_process_to_connect() {
    let output = run_example("sockets/accept_waits_for_a_client.rb");
    assert_eq!(
        output,
        "\"ping\"\n\"Socket is not connected - shutdown(2)\"\n"
    );
}

#[test]
fn a_socket_s_gets_takes_a_separator_and_a_limit() {
    let output = run_example("sockets/gets_with_a_limit.rb");
    assert_eq!(
        output,
        "\"* OK ready\\r\\n\"\n\"* 2 \"\n\"EXISTS\\r\\n\"\n\"A001 OK done\\r\\n\"\nnil\n\"* OK\"\n\" ready\\r\\n\"\n"
    );
}

#[test]
fn socket_tcp_tries_each_address_the_host_answers_on() {
    let output = run_example("sockets/socket_tcp_tries_each_address.rb");
    assert_eq!(output, "\"* OK ready\\r\\n\"\ntrue\nErrno::ECONNREFUSED\n");
}
