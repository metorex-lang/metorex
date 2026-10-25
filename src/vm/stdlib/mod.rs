// The libraries metorex ships with, held in the binary so `require` finds
// them without a directory on the load path.

/// The source in the file at `path`, kept in a section of its own. A
/// lookup by name reads every table entry, and with the sources beside the
/// names it would read a page next to each source.
macro_rules! carried_source {
    ($path:literal) => {{
        const TEXT: &str = include_str!($path);
        #[cfg_attr(target_vendor = "apple", unsafe(link_section = "__TEXT,__rb_sources"))]
        #[cfg_attr(
            not(target_vendor = "apple"),
            unsafe(link_section = ".rodata.rb_sources")
        )]
        static SOURCE: [u8; TEXT.len()] =
            // SAFETY: TEXT holds TEXT.len() bytes.
            unsafe { *(TEXT.as_ptr() as *const [u8; TEXT.len()]) };
        // SAFETY: the bytes are TEXT's, which include_str! checked.
        unsafe { std::str::from_utf8_unchecked(&SOURCE) }
    }};
}

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
    (&["base64"], carried_source!("base64.rb")),
    (
        &["bigdecimal", "bigdecimal/util"],
        carried_source!("bigdecimal.rb"),
    ),
    (&["zlib"], carried_source!("zlib.rb")),
    (&["open3"], carried_source!("open3.rb")),
    (&["syslog"], carried_source!("syslog.rb")),
    (&["openssl"], carried_source!("openssl.rb")),
    (
        &["openssl/buffering"],
        carried_source!("openssl/buffering.rb"),
    ),
    (
        &["cgi", "cgi/escape", "cgi/util"],
        carried_source!("cgi.rb"),
    ),
    (&["net/http"], carried_source!("net/http.rb")),
    (&["net/https"], carried_source!("net/https.rb")),
    (&["net/protocol"], carried_source!("net/protocol.rb")),
    (
        &["net/http/exceptions"],
        carried_source!("net/http/exceptions.rb"),
    ),
    (&["net/http/header"], carried_source!("net/http/header.rb")),
    (
        &["net/http/generic_request"],
        carried_source!("net/http/generic_request.rb"),
    ),
    (
        &["net/http/request"],
        carried_source!("net/http/request.rb"),
    ),
    (
        &["net/http/requests"],
        carried_source!("net/http/requests.rb"),
    ),
    (
        &["net/http/response"],
        carried_source!("net/http/response.rb"),
    ),
    (
        &["net/http/responses"],
        carried_source!("net/http/responses.rb"),
    ),
    (&["net/http/status"], carried_source!("net/http/status.rb")),
    (
        &["net/http/proxy_delta"],
        carried_source!("net/http/proxy_delta.rb"),
    ),
    (&["net/ftp"], carried_source!("net_ftp.rb")),
    (&["English", "english"], carried_source!("english.rb")),
    (&["io/nonblock"], carried_source!("io_nonblock.rb")),
    (&["rbconfig"], carried_source!("rbconfig.rb")),
    (&["rbconfig/sizeof"], carried_source!("rbconfig_sizeof.rb")),
    (&["yaml"], carried_source!("yaml.rb")),
    (&["yaml/store"], carried_source!("yaml/store.rb")),
    (&["yaml/dbm"], carried_source!("yaml/dbm.rb")),
    (&["psych"], carried_source!("psych.rb")),
    (&["psych.so"], carried_source!("psych_native.rb")),
    (
        &["psych/class_loader"],
        carried_source!("psych/class_loader.rb"),
    ),
    (&["psych/coder"], carried_source!("psych/coder.rb")),
    (&["psych/core_ext"], carried_source!("psych/core_ext.rb")),
    (&["psych/exception"], carried_source!("psych/exception.rb")),
    (&["psych/handler"], carried_source!("psych/handler.rb")),
    (
        &["psych/handlers/document_stream"],
        carried_source!("psych/handlers/document_stream.rb"),
    ),
    (
        &["psych/handlers/recorder"],
        carried_source!("psych/handlers/recorder.rb"),
    ),
    (
        &["psych/json/ruby_events"],
        carried_source!("psych/json/ruby_events.rb"),
    ),
    (
        &["psych/json/stream"],
        carried_source!("psych/json/stream.rb"),
    ),
    (
        &["psych/json/tree_builder"],
        carried_source!("psych/json/tree_builder.rb"),
    ),
    (
        &["psych/json/yaml_events"],
        carried_source!("psych/json/yaml_events.rb"),
    ),
    (&["psych/nodes"], carried_source!("psych/nodes.rb")),
    (
        &["psych/nodes/alias"],
        carried_source!("psych/nodes/alias.rb"),
    ),
    (
        &["psych/nodes/document"],
        carried_source!("psych/nodes/document.rb"),
    ),
    (
        &["psych/nodes/mapping"],
        carried_source!("psych/nodes/mapping.rb"),
    ),
    (
        &["psych/nodes/node"],
        carried_source!("psych/nodes/node.rb"),
    ),
    (
        &["psych/nodes/scalar"],
        carried_source!("psych/nodes/scalar.rb"),
    ),
    (
        &["psych/nodes/sequence"],
        carried_source!("psych/nodes/sequence.rb"),
    ),
    (
        &["psych/nodes/stream"],
        carried_source!("psych/nodes/stream.rb"),
    ),
    (&["psych/omap"], carried_source!("psych/omap.rb")),
    (&["psych/parser"], carried_source!("psych/parser.rb")),
    (
        &["psych/scalar_scanner"],
        carried_source!("psych/scalar_scanner.rb"),
    ),
    (&["psych/set"], carried_source!("psych/set.rb")),
    (&["psych/stream"], carried_source!("psych/stream.rb")),
    (&["psych/streaming"], carried_source!("psych/streaming.rb")),
    (
        &["psych/syntax_error"],
        carried_source!("psych/syntax_error.rb"),
    ),
    (
        &["psych/tree_builder"],
        carried_source!("psych/tree_builder.rb"),
    ),
    (&["psych/versions"], carried_source!("psych/versions.rb")),
    (&["psych/visitors"], carried_source!("psych/visitors.rb")),
    (
        &["psych/visitors/depth_first"],
        carried_source!("psych/visitors/depth_first.rb"),
    ),
    (
        &["psych/visitors/emitter"],
        carried_source!("psych/visitors/emitter.rb"),
    ),
    (
        &["psych/visitors/json_tree"],
        carried_source!("psych/visitors/json_tree.rb"),
    ),
    (
        &["psych/visitors/to_ruby"],
        carried_source!("psych/visitors/to_ruby.rb"),
    ),
    (
        &["psych/visitors/visitor"],
        carried_source!("psych/visitors/visitor.rb"),
    ),
    (
        &["psych/visitors/yaml_tree"],
        carried_source!("psych/visitors/yaml_tree.rb"),
    ),
    (&["psych/y"], carried_source!("psych/y.rb")),
    (&["resolv"], carried_source!("resolv.rb")),
    (
        &["rubygems/defaults/operating_system"],
        carried_source!("rubygems/defaults/operating_system.rb"),
    ),
    (
        &["optparse", "optionparser"],
        carried_source!("optparse.rb"),
    ),
    (&["optparse/ac"], carried_source!("optparse/ac.rb")),
    (&["optparse/date"], carried_source!("optparse/date.rb")),
    (&["optparse/kwargs"], carried_source!("optparse/kwargs.rb")),
    (
        &["optparse/shellwords"],
        carried_source!("optparse/shellwords.rb"),
    ),
    (&["optparse/time"], carried_source!("optparse/time.rb")),
    (&["optparse/uri"], carried_source!("optparse/uri.rb")),
    (
        &["optparse/version"],
        carried_source!("optparse/version.rb"),
    ),
    (
        &["random/formatter"],
        carried_source!("random_formatter.rb"),
    ),
    (&["socket"], carried_source!("socket.rb")),
    (&["erb"], carried_source!("erb.rb")),
    (&["erb/version"], carried_source!("erb/version.rb")),
    (&["erb/compiler"], carried_source!("erb/compiler.rb")),
    (&["erb/def_method"], carried_source!("erb/def_method.rb")),
    (&["erb/util"], carried_source!("erb/util.rb")),
    (&["abbrev"], carried_source!("abbrev.rb")),
    (&["etc"], carried_source!("etc.rb")),
    (&["mkmf"], carried_source!("mkmf.rb")),
    (&["pp"], carried_source!("pp.rb")),
    (&["prettyprint"], carried_source!("prettyprint.rb")),
    (&["fcntl"], carried_source!("fcntl.rb")),
    (&["expect"], carried_source!("expect.rb")),
    (&["fiber"], carried_source!("fiber.rb")),
    (&["open-uri"], carried_source!("open_uri.rb")),
    (&["find"], carried_source!("find.rb")),
    (&["getoptlong"], carried_source!("getoptlong.rb")),
    (&["io/console"], carried_source!("io_console.rb")),
    (&["io/console/size"], carried_source!("io_console_size.rb")),
    (&["ipaddr"], carried_source!("ipaddr.rb")),
    (&["coverage"], carried_source!("coverage.rb")),
    (&["date"], carried_source!("date.rb")),
    (&["delegate"], carried_source!("delegate.rb")),
    (
        &[
            "error_highlight",
            "error_highlight/base",
            "error_highlight/core_ext",
            "error_highlight/formatter",
            "error_highlight/version",
        ],
        carried_source!("error_highlight.rb"),
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
        carried_source!("did_you_mean.rb"),
    ),
    (&["syntax_suggest"], carried_source!("syntax_suggest.rb")),
    (
        &["syntax_suggest/core_ext"],
        carried_source!("syntax_suggest_core_ext.rb"),
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
        carried_source!("syntax_suggest_api.rb"),
    ),
    // What prism's C extension defines, over the parser build.rs compiles in.
    (&["prism/prism"], carried_source!("prism_backend.rb")),
    (&["weakref"], carried_source!("weakref.rb")),
    (&["logger"], carried_source!("logger.rb")),
    (&["monitor"], carried_source!("monitor.rb")),
    (
        &["forwardable", "forwardable/impl"],
        carried_source!("forwardable.rb"),
    ),
    (&["tsort"], carried_source!("tsort.rb")),
    (&["io/wait"], carried_source!("io_wait.rb")),
    (&["pty"], carried_source!("pty.rb")),
    (&["continuation"], carried_source!("continuation.rb")),
    (&["pstore"], carried_source!("pstore.rb")),
    (&["benchmark"], carried_source!("benchmark.rb")),
    (&["un"], carried_source!("un.rb")),
    (&["fileutils"], carried_source!("fileutils.rb")),
    (&["tmpdir"], carried_source!("tmpdir.rb")),
    (&["tempfile"], carried_source!("tempfile.rb")),
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
        carried_source!("digest.rb"),
    ),
    (&["matrix"], carried_source!("matrix.rb")),
    (&["fiddle"], carried_source!("fiddle.rb")),
    (&["json"], carried_source!("json.rb")),
    (&["objspace"], carried_source!("objspace.rb")),
    (&["objspace/trace"], carried_source!("objspace_trace.rb")),
    (&["observer"], carried_source!("observer.rb")),
    (&["ostruct"], carried_source!("ostruct.rb")),
    (&["pathname"], carried_source!("pathname.rb")),
    (
        &["<internal:pathname_builtin>"],
        carried_source!("pathname_builtin.rb"),
    ),
    (&["prime"], carried_source!("prime.rb")),
    (&["ripper"], carried_source!("ripper.rb")),
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
        carried_source!("parser_tables.rb"),
    ),
    (&["ripper/filter"], carried_source!("ripper_filter.rb")),
    (&["ripper/lexer"], carried_source!("ripper_lexer.rb")),
    (&["ripper/sexp"], carried_source!("ripper_sexp.rb")),
    (&["securerandom"], carried_source!("securerandom.rb")),
    (&["shellwords"], carried_source!("shellwords.rb")),
    (&["singleton"], carried_source!("singleton.rb")),
    (&["stringio"], carried_source!("stringio.rb")),
    (&["strscan"], carried_source!("strscan.rb")),
    (&["time"], carried_source!("time.rb")),
    (&["timeout"], carried_source!("timeout.rb")),
    (&["uri"], carried_source!("uri_vendored.rb")),
    (&["uri/common"], carried_source!("uri/common.rb")),
    (&["uri/file"], carried_source!("uri/file.rb")),
    (&["uri/ftp"], carried_source!("uri/ftp.rb")),
    (&["uri/generic"], carried_source!("uri/generic.rb")),
    (&["uri/http"], carried_source!("uri/http.rb")),
    (&["uri/https"], carried_source!("uri/https.rb")),
    (&["uri/ldap"], carried_source!("uri/ldap.rb")),
    (&["uri/ldaps"], carried_source!("uri/ldaps.rb")),
    (&["uri/mailto"], carried_source!("uri/mailto.rb")),
    (
        &["uri/rfc2396_parser"],
        carried_source!("uri/rfc2396_parser.rb"),
    ),
    (
        &["uri/rfc3986_parser"],
        carried_source!("uri/rfc3986_parser.rb"),
    ),
    (&["uri/version"], carried_source!("uri/version.rb")),
    (&["uri/ws"], carried_source!("uri/ws.rb")),
    (&["uri/wss"], carried_source!("uri/wss.rb")),
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
