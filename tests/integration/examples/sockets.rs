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

#[test]
fn a_tls_client_and_server_shake_hands_verify_and_echo() {
    let output = run_example("sockets/tls_handshake.rb");
    assert_eq!(
        output,
        "\"/C=JP/ST=Shimane/O=Ruby Core Team/OU=Ruby Test/CN=localhost\"\n\"/C=JP/ST=Shimane/L=Matz-e city/O=Ruby Core Team/CN=Ruby Test CA/emailAddress=security@ruby-lang.org\"\n0\n0\n\"sha256WithRSAEncryption\"\n2024-01-01 11:47:23 UTC\n2033-12-29 11:47:23 UTC\ntrue\ntrue\ntrue\ntrue\n\"TLSv1.3\"\n\"TLS_AES_128_GCM_SHA256\"\n0\n\"/C=JP/ST=Shimane/O=Ruby Core Team/OU=Ruby Test/CN=localhost\"\n1\n\"echo: a001 NOOP\\n\"\n\"\"\n:served\n\"hostname \\\"127.0.0.1\\\" does not match the server certificate\"\n\"SSL_connect returned=1 errno=0 state=error: certificate verify failed (unable to get local issuer certificate)\"\n[OpenSSL::SSL::SSLError, \"SSL_accept returned=1 errno=0 state=error: tlsv1 alert unknown ca (SSL alert number 48)\"]\n20\n\"echo: plain\\n\"\n:served\n"
    );
}

#[test]
fn sockets_write_through_by_default() {
    let output = run_example("sockets/socket_sync_mode.rb");
    assert_eq!(output, "true\ntrue\ntrue\n[true, true]\n");
}

#[test]
fn a_tls_server_greets_a_client_connected_through_socket_tcp() {
    let output = run_example("sockets/tls_greeting_after_accept.rb");
    assert_eq!(
        output,
        "\"* OK test server\\r\\n\"\n\"* BYE terminating connection\\r\\n\"\nnil\n\"RUBY0001 LOGOUT\\r\\n\"\n"
    );
}

#[test]
fn net_imap_reads_a_greeting_over_tls() {
    let output = run_example("sockets/imap_over_tls.rb");
    assert_eq!(
        output,
        "\"test server\"\nfalse\n\"RUBY0001 LOGOUT\\r\\n\"\n"
    );
}

#[test]
fn tls_runs_over_a_unix_socket_pair_with_a_nonblocking_connect() {
    let output = run_example("sockets/tls_over_a_unix_socket_pair.rb");
    assert_eq!(
        output,
        ":wait_readable\n0\n[\"/C=JP/ST=Shimane/O=Ruby Core Team/OU=Ruby Test/CN=localhost\", \"/C=JP/ST=Shimane/L=Matz-e city/O=Ruby Core Team/CN=Ruby Test CA/emailAddress=security@ruby-lang.org\"]\n\"220 ready\\n\"\n\"QUIT\\n\"\n"
    );
}

#[test]
fn net_smtp_sends_a_sender_outside_ascii_with_smtputf8() {
    let output = run_example("sockets/smtp_sender_outside_ascii.rb");
    assert_eq!(
        output,
        "[\"EHLO localhost\\r\\n\", \"MAIL FROM:<rené@example.com> SMTPUTF8\\r\\n\", \"RCPT TO:<foo@example.com>\\r\\n\", \"DATA\\r\\n\", \"message\\r\\n\", \".\\r\\n\", \"QUIT\\r\\n\"]\n"
    );
}

#[test]
fn net_imap_reads_a_literal_within_max_response_size() {
    let output = run_example("sockets/imap_literal_within_max_response_size.rb");
    assert_eq!(output, "1234\n");
}

#[test]
fn a_socket_reads_a_literal_after_a_line_into_a_buffer() {
    let output = run_example("sockets/socket_reads_a_literal_after_a_line.rb");
    assert_eq!(
        output,
        "\"* 1 FETCH (BODY[] {12345}\\r\\n\"\n12345\ntrue\n12345\n\")\\r\\n\"\n\"RUBY0001 OK done\\r\\n\"\n"
    );
}

#[test]
fn a_socket_reads_lines_by_separator_and_chomps_them() {
    let output = run_example("sockets/readline_with_separator_and_chomp.rb");
    assert_eq!(output, "\"<fo\"\n\"o>\"\n\"<bar\"\n\"\"\n\"lin\"\n");
}

#[test]
fn listeners_open_on_one_ephemeral_port_and_survive_handing_on_by_descriptor() {
    let output = run_example("sockets/tcp_server_sockets_on_an_ephemeral_port.rb");
    assert_eq!(output, "false\n1\ntrue\nSocket\ntrue\nTCPServer\ntrue\n");
}

#[test]
fn readpartial_fills_the_buffer_it_is_handed_with_what_gets_left_behind() {
    let output = run_example("sockets/readpartial_fills_a_buffer_after_gets.rb");
    assert_eq!(output, "\"ok\"\ntrue\n#<Encoding:BINARY (ASCII-8BIT)>\n");
}

#[test]
fn a_timeout_ends_a_socket_read_that_is_still_waiting() {
    let output = run_example("sockets/timeout_ends_a_read_that_is_waiting.rb");
    assert_eq!(
        output,
        "[:gets, \"execution expired\"]\n[:readpartial, \"execution expired\"]\n\"late\\n\"\n"
    );
}

#[test]
fn tcp_server_sockets_listen_on_every_address_of_the_host_on_one_port() {
    let output = run_example("sockets/tcp_server_sockets_listen_on_every_address.rb");
    assert_eq!(
        output,
        concat!(
            "128\n",
            "[[\"::\", true], [\"0.0.0.0\", false]]\n",
            "1\n",
            "[true, true]\n",
            "[[\"127.0.0.1\", false]]\n",
            "1\n",
            "[true]\n",
            "[[\"::1\", true], [\"127.0.0.1\", false]]\n",
            "1\n",
            "[true, true]\n",
        )
    );
}

#[test]
fn read_nonblock_at_the_end_of_the_stream_empties_the_buffer_and_raises_eof() {
    let output = run_example("sockets/read_nonblock_at_the_end_of_the_stream.rb");
    assert_eq!(output, "\"ab\"\n\"ab\"\nEOFError\n\"\"\nnil\nnil\nnil\n");
}

#[test]
fn a_nonblocking_connect_confirmed_through_so_error_writes_and_reads() {
    let output = run_example("sockets/nonblocking_connect_confirmed_through_so_error.rb");
    assert_eq!(output, ":waiting\n0\n4\n\"ping\"\n\"pong\"\n");
}

#[test]
fn a_thread_waiting_in_accept_gets_an_io_error_when_another_thread_closes_the_listener() {
    let output = run_example("sockets/accept_ends_when_another_thread_closes_the_listener.rb");
    assert_eq!(output, "[IOError, \"stream closed in another thread\"]\n");
}

#[test]
fn a_socket_reads_lines_through_readline_and_gets_with_chomp() {
    let output = run_example("sockets/socket_readline_and_gets_with_chomp.rb");
    assert_eq!(output, "\"first\\n\"\n\"second\"\n\"third\"\nnil\n");
}

#[test]
fn a_tls_client_reads_with_read_nonblock_and_waits_in_select() {
    let output = run_example("sockets/tls_read_nonblock_with_select.rb");
    assert_eq!(
        output,
        "\"HTTP/1.1 200 OK\\r\\nContent-Length: 2\\r\\n\\r\\nok\"\n\"GET / HTTP/1.1\\r\\n\"\n"
    );
}

#[test]
fn the_plain_socket_under_a_server_tls_socket_is_readable_once_the_client_writes() {
    let output = run_example("sockets/tls_raw_socket_readable_after_accept.rb");
    assert_eq!(output, "[true, \"GET / HTTP/1.1\\r\\n\"]\n");
}

#[test]
fn a_unix_family_socket_connects_to_a_path_once() {
    let output = run_example("sockets/unix_socket_connect_and_connect_nonblock.rb");
    assert_eq!(
        output,
        "0\n\"hello\"\nErrno::EISCONN\n0\n\"hello\"\nErrno::EISCONN\n"
    );
}

#[test]
fn a_tls_server_requiring_a_client_certificate_refuses_a_client_without_one() {
    let output = run_example("sockets/tls_server_requires_a_client_certificate.rb");
    assert_eq!(
        output,
        concat!(
            "OpenSSL::SSL::SSLError\nOpenSSL::SSL::SSLError\n",
            "\"hello /C=JP/ST=Shimane/O=Ruby Core Team/OU=Ruby Test/CN=localhost\\n\"\ntrue\n",
        )
    );
}

#[test]
fn a_tls_server_answers_with_the_context_its_servername_callback_chooses() {
    let output = run_example("sockets/tls_server_chooses_a_context_by_server_name.rb");
    assert_eq!(
        output,
        concat!(
            "[\"excon\", \"/CN=excon\", \"ok\\n\"]\n",
            "[\"other\", \"/C=JP/ST=Shimane/O=Ruby Core Team/OU=Ruby Test/CN=localhost\", \"ok\\n\"]\n",
            "[nil, nil]\n",
            "[[OpenSSL::SSL::SSLSocket, \"excon\"], [OpenSSL::SSL::SSLSocket, \"other\"]]\n",
        )
    );
}

#[test]
fn ip_address_list_names_the_address_of_every_interface() {
    let output = run_example("sockets/ip_address_list_lists_every_interface_address.rb");
    assert_eq!(output, "true\ntrue\ntrue\ntrue\n");
}

#[test]
fn an_ipv6_address_keeps_the_interface_it_names_after_a_percent_sign() {
    let output = run_example("sockets/addrinfo_with_an_ipv6_scope.rb");
    assert_eq!(output, "#<Addrinfo: fe80::1%lo0>\n\"fe80::1%lo0\"\ntrue\n");
}

#[test]
fn write_takes_several_strings() {
    let output = run_example("sockets/write_takes_several_strings.rb");
    assert_eq!(output, "17\n\"HTTP/1.1 200 OK\\r\\n\"\n");
}
