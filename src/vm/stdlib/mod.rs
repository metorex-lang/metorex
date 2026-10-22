// The libraries metorex ships with, held in the binary so `require` finds
// them without a directory on the load path.

mod bundler_libraries;
mod gem_libraries;
mod prism_libraries;
mod rex_libraries;
mod rspec_libraries;
mod rubygems_libraries;
pub(crate) mod source_digest;
use bundler_libraries::BUNDLER_LIBRARIES;
use gem_libraries::GEM_LIBRARIES;
use prism_libraries::PRISM_LIBRARIES;
use rex_libraries::REX_LIBRARIES;
use rspec_libraries::RSPEC_LIBRARIES;
use rubygems_libraries::RUBYGEMS_LIBRARIES;

/// The file a library metorex carries says its code was written in. A
/// bundled gem's file sits in a directory of that gem's own, as it does in
/// MRI's gem directory, so a gem that reads its own directory from a path
/// does not take another gem's files for its own.
pub(crate) fn embedded_library_file(name: &str) -> String {
    // A file MRI itself compiles in names itself the way MRI does.
    if name.starts_with("<internal:") {
        return name.to_string();
    }
    let feature = name.strip_suffix(".rb").unwrap_or(name);
    match gem_directory(feature) {
        Some(gem) => format!("{EMBEDDED_LIBRARY_PREFIX}gems/{gem}/lib/{feature}.rb"),
        None => format!("{EMBEDDED_LIBRARY_PREFIX}{feature}.rb"),
    }
}

/// The directory of the bundled gem the library metorex carries under
/// `feature` belongs to, `power_assert-3.0.1` for `power_assert`, or None
/// when the library is not a bundled gem's.
fn gem_directory(feature: &str) -> Option<&'static str> {
    if EMBEDDED_LIBRARIES
        .iter()
        .any(|(names, _)| names.contains(&feature))
        || PRISM_LIBRARIES
            .iter()
            .chain(RSPEC_LIBRARIES)
            .chain(REX_LIBRARIES)
            .any(|(named, _)| *named == feature)
    {
        return None;
    }
    GEM_LIBRARIES
        .iter()
        .find(|(named, _, _)| *named == feature)
        .map(|(_, gem, _)| *gem)
}

const EMBEDDED_LIBRARY_PREFIX: &str = "<metorex>/";

/// The libraries metorex carries that Ruby writes in C, as `ext/` holds them.
const C_EXTENSIONS: &[&str] = &[
    "bigdecimal",
    "coverage",
    "date",
    "etc",
    "fcntl",
    "fiddle",
    "io/console",
    "io/nonblock",
    "monitor",
    "objspace",
    "rbconfig/sizeof",
    "stringio",
    "strscan",
    "syslog",
    "zlib",
];

/// Whether code written in `file` stands for code Ruby writes in C: the
/// core library metorex loads at startup, or a library Ruby ships as a C
/// extension.
pub(crate) fn written_for_c(file: &str) -> bool {
    if file.starts_with("<internal:") {
        return true;
    }
    carried_feature(file).is_some_and(|named| C_EXTENSIONS.contains(&named))
}

/// The libraries metorex carries: the names `require` reaches each by, and
/// its source.
const EMBEDDED_LIBRARIES: &[(&[&str], &str)] = &[
    (&["base64"], include_str!("base64.rb")),
    (
        &["bigdecimal", "bigdecimal/util"],
        include_str!("bigdecimal.rb"),
    ),
    (&["zlib"], include_str!("zlib.rb")),
    (&["open3"], include_str!("open3.rb")),
    (&["syslog"], include_str!("syslog.rb")),
    (&["openssl"], include_str!("openssl.rb")),
    (&["openssl/buffering"], include_str!("openssl/buffering.rb")),
    (&["cgi", "cgi/escape", "cgi/util"], include_str!("cgi.rb")),
    (&["net/http"], include_str!("net/http.rb")),
    (&["net/https"], include_str!("net/https.rb")),
    (&["net/protocol"], include_str!("net/protocol.rb")),
    (
        &["net/http/exceptions"],
        include_str!("net/http/exceptions.rb"),
    ),
    (&["net/http/header"], include_str!("net/http/header.rb")),
    (
        &["net/http/generic_request"],
        include_str!("net/http/generic_request.rb"),
    ),
    (&["net/http/request"], include_str!("net/http/request.rb")),
    (&["net/http/requests"], include_str!("net/http/requests.rb")),
    (&["net/http/response"], include_str!("net/http/response.rb")),
    (
        &["net/http/responses"],
        include_str!("net/http/responses.rb"),
    ),
    (&["net/http/status"], include_str!("net/http/status.rb")),
    (
        &["net/http/proxy_delta"],
        include_str!("net/http/proxy_delta.rb"),
    ),
    (&["net/ftp"], include_str!("net_ftp.rb")),
    (&["English", "english"], include_str!("english.rb")),
    (&["io/nonblock"], include_str!("io_nonblock.rb")),
    (&["rbconfig"], include_str!("rbconfig.rb")),
    (&["rbconfig/sizeof"], include_str!("rbconfig_sizeof.rb")),
    (&["yaml"], include_str!("yaml.rb")),
    (&["yaml/store"], include_str!("yaml/store.rb")),
    (&["yaml/dbm"], include_str!("yaml/dbm.rb")),
    (&["psych"], include_str!("psych.rb")),
    (&["psych.so"], include_str!("psych_native.rb")),
    (
        &["psych/class_loader"],
        include_str!("psych/class_loader.rb"),
    ),
    (&["psych/coder"], include_str!("psych/coder.rb")),
    (&["psych/core_ext"], include_str!("psych/core_ext.rb")),
    (&["psych/exception"], include_str!("psych/exception.rb")),
    (&["psych/handler"], include_str!("psych/handler.rb")),
    (
        &["psych/handlers/document_stream"],
        include_str!("psych/handlers/document_stream.rb"),
    ),
    (
        &["psych/handlers/recorder"],
        include_str!("psych/handlers/recorder.rb"),
    ),
    (
        &["psych/json/ruby_events"],
        include_str!("psych/json/ruby_events.rb"),
    ),
    (&["psych/json/stream"], include_str!("psych/json/stream.rb")),
    (
        &["psych/json/tree_builder"],
        include_str!("psych/json/tree_builder.rb"),
    ),
    (
        &["psych/json/yaml_events"],
        include_str!("psych/json/yaml_events.rb"),
    ),
    (&["psych/nodes"], include_str!("psych/nodes.rb")),
    (&["psych/nodes/alias"], include_str!("psych/nodes/alias.rb")),
    (
        &["psych/nodes/document"],
        include_str!("psych/nodes/document.rb"),
    ),
    (
        &["psych/nodes/mapping"],
        include_str!("psych/nodes/mapping.rb"),
    ),
    (&["psych/nodes/node"], include_str!("psych/nodes/node.rb")),
    (
        &["psych/nodes/scalar"],
        include_str!("psych/nodes/scalar.rb"),
    ),
    (
        &["psych/nodes/sequence"],
        include_str!("psych/nodes/sequence.rb"),
    ),
    (
        &["psych/nodes/stream"],
        include_str!("psych/nodes/stream.rb"),
    ),
    (&["psych/omap"], include_str!("psych/omap.rb")),
    (&["psych/parser"], include_str!("psych/parser.rb")),
    (
        &["psych/scalar_scanner"],
        include_str!("psych/scalar_scanner.rb"),
    ),
    (&["psych/set"], include_str!("psych/set.rb")),
    (&["psych/stream"], include_str!("psych/stream.rb")),
    (&["psych/streaming"], include_str!("psych/streaming.rb")),
    (
        &["psych/syntax_error"],
        include_str!("psych/syntax_error.rb"),
    ),
    (
        &["psych/tree_builder"],
        include_str!("psych/tree_builder.rb"),
    ),
    (&["psych/versions"], include_str!("psych/versions.rb")),
    (&["psych/visitors"], include_str!("psych/visitors.rb")),
    (
        &["psych/visitors/depth_first"],
        include_str!("psych/visitors/depth_first.rb"),
    ),
    (
        &["psych/visitors/emitter"],
        include_str!("psych/visitors/emitter.rb"),
    ),
    (
        &["psych/visitors/json_tree"],
        include_str!("psych/visitors/json_tree.rb"),
    ),
    (
        &["psych/visitors/to_ruby"],
        include_str!("psych/visitors/to_ruby.rb"),
    ),
    (
        &["psych/visitors/visitor"],
        include_str!("psych/visitors/visitor.rb"),
    ),
    (
        &["psych/visitors/yaml_tree"],
        include_str!("psych/visitors/yaml_tree.rb"),
    ),
    (&["psych/y"], include_str!("psych/y.rb")),
    (&["resolv"], include_str!("resolv.rb")),
    (
        &["rubygems/defaults/operating_system"],
        include_str!("rubygems/defaults/operating_system.rb"),
    ),
    (&["optparse", "optionparser"], include_str!("optparse.rb")),
    (&["optparse/ac"], include_str!("optparse/ac.rb")),
    (&["optparse/date"], include_str!("optparse/date.rb")),
    (&["optparse/kwargs"], include_str!("optparse/kwargs.rb")),
    (
        &["optparse/shellwords"],
        include_str!("optparse/shellwords.rb"),
    ),
    (&["optparse/time"], include_str!("optparse/time.rb")),
    (&["optparse/uri"], include_str!("optparse/uri.rb")),
    (&["optparse/version"], include_str!("optparse/version.rb")),
    (&["random/formatter"], include_str!("random_formatter.rb")),
    (&["socket"], include_str!("socket.rb")),
    (&["erb"], include_str!("erb.rb")),
    (&["erb/version"], include_str!("erb/version.rb")),
    (&["erb/compiler"], include_str!("erb/compiler.rb")),
    (&["erb/def_method"], include_str!("erb/def_method.rb")),
    (&["erb/util"], include_str!("erb/util.rb")),
    (&["abbrev"], include_str!("abbrev.rb")),
    (&["etc"], include_str!("etc.rb")),
    (&["mkmf"], include_str!("mkmf.rb")),
    (&["pp"], include_str!("pp.rb")),
    (&["prettyprint"], include_str!("prettyprint.rb")),
    (&["fcntl"], include_str!("fcntl.rb")),
    (&["expect"], include_str!("expect.rb")),
    (&["fiber"], include_str!("fiber.rb")),
    (&["open-uri"], include_str!("open_uri.rb")),
    (&["find"], include_str!("find.rb")),
    (&["getoptlong"], include_str!("getoptlong.rb")),
    (&["io/console"], include_str!("io_console.rb")),
    (&["io/console/size"], include_str!("io_console_size.rb")),
    (&["ipaddr"], include_str!("ipaddr.rb")),
    (&["coverage"], include_str!("coverage.rb")),
    (&["csv"], include_str!("csv.rb")),
    (&["date"], include_str!("date.rb")),
    (&["delegate"], include_str!("delegate.rb")),
    (
        &[
            "error_highlight",
            "error_highlight/base",
            "error_highlight/core_ext",
            "error_highlight/formatter",
            "error_highlight/version",
        ],
        include_str!("error_highlight.rb"),
    ),
    (
        &[
            "did_you_mean",
            "did_you_mean/core_ext/name_error",
            "did_you_mean/formatter",
            "did_you_mean/jaro_winkler",
            "did_you_mean/levenshtein",
            "did_you_mean/spell_checker",
            "did_you_mean/spell_checkers/key_error_checker",
            "did_you_mean/spell_checkers/method_name_checker",
            "did_you_mean/spell_checkers/name_error_checkers",
            "did_you_mean/spell_checkers/null_checker",
            "did_you_mean/spell_checkers/pattern_key_name_checker",
            "did_you_mean/spell_checkers/require_path_checker",
            "did_you_mean/tree_spell_checker",
            "did_you_mean/version",
        ],
        include_str!("did_you_mean.rb"),
    ),
    (&["syntax_suggest"], include_str!("syntax_suggest.rb")),
    (
        &["syntax_suggest/core_ext"],
        include_str!("syntax_suggest_core_ext.rb"),
    ),
    (
        &[
            "syntax_suggest/api",
            "syntax_suggest/around_block_scan",
            "syntax_suggest/block_expand",
            "syntax_suggest/capture/before_after_keyword_ends",
            "syntax_suggest/capture/falling_indent_lines",
            "syntax_suggest/capture_code_context",
            "syntax_suggest/clean_document",
            "syntax_suggest/cli",
            "syntax_suggest/code_block",
            "syntax_suggest/code_frontier",
            "syntax_suggest/code_line",
            "syntax_suggest/code_search",
            "syntax_suggest/display_code_with_line_numbers",
            "syntax_suggest/display_invalid_blocks",
            "syntax_suggest/explain_syntax",
            "syntax_suggest/left_right_lex_count",
            "syntax_suggest/lex_all",
            "syntax_suggest/lex_value",
            "syntax_suggest/mini_stringio",
            "syntax_suggest/parse_blocks_from_indent_line",
            "syntax_suggest/pathname_from_message",
            "syntax_suggest/priority_engulf_queue",
            "syntax_suggest/priority_queue",
            "syntax_suggest/ripper_errors",
            "syntax_suggest/scan_history",
            "syntax_suggest/unvisited_lines",
            "syntax_suggest/version",
        ],
        include_str!("syntax_suggest_api.rb"),
    ),
    // What prism's C extension defines, over the parser build.rs compiles in.
    (&["prism/prism"], include_str!("prism_backend.rb")),
    (&["weakref"], include_str!("weakref.rb")),
    (&["logger"], include_str!("logger.rb")),
    (&["monitor"], include_str!("monitor.rb")),
    (
        &["forwardable", "forwardable/impl"],
        include_str!("forwardable.rb"),
    ),
    (&["tsort"], include_str!("tsort.rb")),
    (&["io/wait"], include_str!("io_wait.rb")),
    (&["pty"], include_str!("pty.rb")),
    (&["continuation"], include_str!("continuation.rb")),
    (&["pstore"], include_str!("pstore.rb")),
    (&["benchmark"], include_str!("benchmark.rb")),
    (&["un"], include_str!("un.rb")),
    (&["fileutils"], include_str!("fileutils.rb")),
    (&["tmpdir"], include_str!("tmpdir.rb")),
    (&["tempfile"], include_str!("tempfile.rb")),
    // Every name the digest library is reached by loads the one
    // file, which carries all of the algorithms metorex has.
    (
        &[
            "digest",
            "digest/md5",
            "digest/sha1",
            "digest/sha2",
            "digest/rmd160",
            "digest/bubblebabble",
        ],
        include_str!("digest.rb"),
    ),
    (&["matrix"], include_str!("matrix.rb")),
    (&["drb", "drb/drb"], include_str!("drb.rb")),
    (&["drb/acl"], include_str!("drb_acl.rb")),
    (&["fiddle"], include_str!("fiddle.rb")),
    (&["json"], include_str!("json.rb")),
    (&["objspace"], include_str!("objspace.rb")),
    (&["objspace/trace"], include_str!("objspace_trace.rb")),
    (&["observer"], include_str!("observer.rb")),
    (&["ostruct"], include_str!("ostruct.rb")),
    (&["pathname"], include_str!("pathname.rb")),
    (
        &["<internal:pathname_builtin>"],
        include_str!("pathname_builtin.rb"),
    ),
    (&["prime"], include_str!("prime.rb")),
    (&["ripper"], include_str!("ripper.rb")),
    // The scanner and the grammar, which Ruby writes in C, load with
    // the core class that dispatches their events.
    (
        &["ripper/core"],
        concat!(
            include_str!("ripper_core.rb"),
            include_str!("ripper_scanner.rb"),
            include_str!("ripper_grammar.rb")
        ),
    ),
    // MRI's parser tables, read when a syntax error is worded.
    (
        &["<internal:parser_tables>"],
        include_str!("parser_tables.rb"),
    ),
    (&["ripper/filter"], include_str!("ripper_filter.rb")),
    (&["ripper/lexer"], include_str!("ripper_lexer.rb")),
    (&["ripper/sexp"], include_str!("ripper_sexp.rb")),
    (&["securerandom"], include_str!("securerandom.rb")),
    (&["shellwords"], include_str!("shellwords.rb")),
    (&["singleton"], include_str!("singleton.rb")),
    (&["stringio"], include_str!("stringio.rb")),
    (&["strscan"], include_str!("strscan.rb")),
    (&["time"], include_str!("time.rb")),
    (&["timeout"], include_str!("timeout.rb")),
    (&["uri"], include_str!("uri_vendored.rb")),
    (&["uri/common"], include_str!("uri/common.rb")),
    (&["uri/file"], include_str!("uri/file.rb")),
    (&["uri/ftp"], include_str!("uri/ftp.rb")),
    (&["uri/generic"], include_str!("uri/generic.rb")),
    (&["uri/http"], include_str!("uri/http.rb")),
    (&["uri/https"], include_str!("uri/https.rb")),
    (&["uri/ldap"], include_str!("uri/ldap.rb")),
    (&["uri/ldaps"], include_str!("uri/ldaps.rb")),
    (&["uri/mailto"], include_str!("uri/mailto.rb")),
    (
        &["uri/rfc2396_parser"],
        include_str!("uri/rfc2396_parser.rb"),
    ),
    (
        &["uri/rfc3986_parser"],
        include_str!("uri/rfc3986_parser.rb"),
    ),
    (&["uri/version"], include_str!("uri/version.rb")),
    (&["uri/ws"], include_str!("uri/ws.rb")),
    (&["uri/wss"], include_str!("uri/wss.rb")),
];

/// The source of a library metorex carries, or None for a name it does not
/// have. A file on the load path wins over one of these.
pub(crate) fn embedded_library(name: &str) -> Option<&'static str> {
    let trimmed = name.strip_suffix(".rb").unwrap_or(name);
    EMBEDDED_LIBRARIES
        .iter()
        .find(|(names, _)| names.contains(&trimmed))
        .map(|(_, source)| *source)
        .or_else(|| {
            PRISM_LIBRARIES
                .iter()
                .chain(RSPEC_LIBRARIES)
                .chain(REX_LIBRARIES)
                .find(|(named, _)| *named == trimmed)
                .map(|(_, source)| *source)
        })
        .or_else(|| {
            GEM_LIBRARIES
                .iter()
                .find(|(named, _, _)| *named == trimmed)
                .map(|(_, _, source)| *source)
        })
        .or_else(|| {
            RUBYGEMS_LIBRARIES
                .iter()
                .chain(BUNDLER_LIBRARIES)
                .find(|(named, _)| *named == trimmed)
                .map(|(_, source)| *source)
        })
}

/// The files other than its library a bundled gem metorex carries reads.
pub(crate) fn gem_data() -> &'static [(&'static str, &'static str)] {
    gem_libraries::GEM_DATA
}

/// The feature name of a library metorex carries, for the path its files
/// read under, `<metorex>/` and then the feature, as `__dir__` builds them,
/// with a bundled gem's directory between.
pub(crate) fn carried_feature(name: &str) -> Option<&str> {
    let feature = name.strip_prefix(EMBEDDED_LIBRARY_PREFIX)?;
    let feature = match feature.strip_prefix("gems/") {
        Some(in_gem) => in_gem.split_once("/lib/")?.1,
        None => feature,
    };
    let feature = feature.strip_suffix(".rb").unwrap_or(feature);
    embedded_library(feature).is_some().then_some(feature)
}

/// Every name `require` finds a library metorex carries under.
pub(crate) fn embedded_library_names() -> impl Iterator<Item = &'static str> {
    EMBEDDED_LIBRARIES
        .iter()
        .flat_map(|(names, _)| names.iter().copied())
        .chain(
            PRISM_LIBRARIES
                .iter()
                .chain(RSPEC_LIBRARIES)
                .chain(REX_LIBRARIES)
                .chain(RUBYGEMS_LIBRARIES)
                .chain(BUNDLER_LIBRARIES)
                .map(|(named, _)| *named),
        )
        .chain(GEM_LIBRARIES.iter().map(|(named, _, _)| *named))
}
