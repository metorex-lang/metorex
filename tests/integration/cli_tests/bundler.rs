// `metorex bundle install` and `metorex bundle exec`, against a gem server
// the test starts.

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Stdio};

use super::metorex_cmd;

/// The gem server in `bundler/gem_server.rb`, stopped when dropped.
struct GemServer {
    child: Child,
    port: String,
}

impl GemServer {
    fn start(examples: &Path) -> GemServer {
        let mut child = metorex_cmd()
            .arg(examples.join("bundler/gem_server.rb"))
            .stdout(Stdio::piped())
            .spawn()
            .expect("the gem server starts");
        let mut port = String::new();
        BufReader::new(child.stdout.take().expect("the server's output is piped"))
            .read_line(&mut port)
            .expect("the server prints its port");
        GemServer {
            child,
            port: port.trim().to_string(),
        }
    }
}

impl Drop for GemServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// A directory of the app's own under the temporary directory, holding a copy
/// of `bundler/app`, so the lockfile and the bundle stay out of the checkout.
fn app_copy(examples: &Path) -> PathBuf {
    let app = std::env::temp_dir().join(format!("metorex_bundler_app_{}", std::process::id()));
    std::fs::create_dir_all(&app).expect("the temporary directory is writable");
    for name in ["Gemfile", "totals.rb"] {
        std::fs::copy(examples.join("bundler/app").join(name), app.join(name))
            .expect("the app's files copy");
    }
    app
}

fn bundle(app: &Path, server: &GemServer, words: &[&str]) -> std::process::Output {
    metorex_cmd()
        .current_dir(app)
        .arg("bundle")
        .args(words)
        .env("GEM_SERVER", format!("http://127.0.0.1:{}", server.port))
        .env("HOME", app)
        .env("BUNDLE_PATH", app.join("bundle"))
        .env_remove("BUNDLE_GEMFILE")
        .env_remove("RUBYOPT")
        .output()
        .expect("bundle runs")
}

#[test]
fn bundle_install_installs_a_gem_from_a_server_and_bundle_exec_runs_with_it() {
    let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join(crate::common::EXAMPLES_DIR);
    let server = GemServer::start(&examples);
    let app = app_copy(&examples);

    let installed = bundle(&app, &server, &["install"]);
    let printed = String::from_utf8_lossy(&installed.stdout);
    assert!(
        installed.status.success(),
        "{printed}{}",
        String::from_utf8_lossy(&installed.stderr)
    );
    assert!(
        printed.contains("Installing ledger_totals 1.0.0"),
        "{printed}"
    );

    let ran = bundle(&app, &server, &["exec", "./totals.rb"]);
    assert_eq!(
        String::from_utf8_lossy(&ran.stdout),
        "1320\n",
        "{}",
        String::from_utf8_lossy(&ran.stderr)
    );
}
