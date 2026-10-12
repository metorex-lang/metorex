// Code samples published on metorex.org.

use super::run_example;

#[test]
fn test_site_home_snippet_execution() {
    let expected = "#<struct Point x=4, y=6>\ny leads by 2\n";
    let output = run_example("site/home.rb");
    assert_eq!(output, expected);
}

#[test]
fn test_site_install_hello_execution() {
    let expected = "Hello, world!\n[2, 4, 6]\n";
    let output = run_example("site/install_hello.rb");
    assert_eq!(output, expected);
}
