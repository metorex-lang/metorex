// The names Ruby's core classes and standard libraries answer to under a
// second name.

/// Each core or standard library method Ruby defines a second time under
/// another name, keyed as `Class#name` for an instance method and
/// `Class.name` for a singleton one, with the name it repeats. A Method or
/// UnboundMethod for either name is equal to one for the other.
const CORE_ALIASES: &[(&str, &str)] = &[
    ("ARGF.class#each", "each_line"),
    ("ARGF.class#eof", "eof?"),
    ("ARGF.class#inspect", "to_s"),
    ("ARGF.class#path", "filename"),
    ("ARGF.class#tell", "pos"),
    ("ARGF.class#to_a", "readlines"),
    ("ARGF.class#to_i", "fileno"),
    ("Addrinfo#to_s", "to_sockaddr"),
    ("Array#append", "push"),
    ("Array#collect!", "map!"),
    ("Array#collect", "map"),
    ("Array#detect", "find"),
    ("Array#filter!", "select!"),
    ("Array#filter", "select"),
    ("Array#index", "find_index"),
    ("Array#length", "size"),
    ("Array#prepend", "unshift"),
    ("Array#slice", "[]"),
    ("Array#to_s", "inspect"),
    ("BasicObject#equal?", "=="),
    ("BigDecimal#clone", "dup"),
    ("Complex#conj", "conjugate"),
    ("Complex#imag", "imaginary"),
    ("Complex#magnitude", "abs"),
    ("Complex#quo", "/"),
    ("Complex.rect", "rectangular"),
    ("Data#inspect", "to_s"),
    ("Date#ctime", "asctime"),
    ("Date#mday", "day"),
    ("Date#mon", "month"),
    ("Date#succ", "next"),
    ("Date.valid_civil?", "valid_date?"),
    ("DateTime#iso8601", "xmlschema"),
    ("DateTime#min", "minute"),
    ("DateTime#sec", "second"),
    ("DateTime#second_fraction", "sec_fraction"),
    ("Digest::Instance#size", "length"),
    ("Digest::Instance#update", "<<"),
    ("Digest::MD5#size", "length"),
    ("Digest::MD5#update", "<<"),
    ("Digest::SHA256#size", "length"),
    ("Digest::SHA256#update", "<<"),
    ("Digest::SHA384#size", "length"),
    ("Digest::SHA384#update", "<<"),
    ("Digest::SHA512#size", "length"),
    ("Digest::SHA512#update", "<<"),
    ("Dir#path", "to_path"),
    ("Dir#tell", "pos"),
    ("Dir.getwd", "pwd"),
    ("Dir.rmdir", "delete"),
    ("Dir.unlink", "delete"),
    ("Encoding#to_s", "name"),
    ("Enumerable#collect", "map"),
    ("Enumerable#collect_concat", "flat_map"),
    ("Enumerable#detect", "find"),
    ("Enumerable#entries", "to_a"),
    ("Enumerable#filter", "select"),
    ("Enumerable#find_all", "select"),
    ("Enumerable#member?", "include?"),
    ("Enumerator#with_object", "each_with_object"),
    ("Enumerator::Lazy#collect", "map"),
    ("Enumerator::Lazy#collect_concat", "flat_map"),
    ("Enumerator::Lazy#enum_for", "to_enum"),
    ("Enumerator::Lazy#filter", "select"),
    ("Enumerator::Lazy#find_all", "select"),
    ("FalseClass#^", "|"),
    ("FalseClass#inspect", "to_s"),
    ("File#path", "to_path"),
    ("File.fnmatch?", "fnmatch"),
    ("File.unlink", "delete"),
    ("File.zero?", "empty?"),
    ("FileTest.zero?", "empty?"),
    ("Float#===", "=="),
    ("Float#inspect", "to_s"),
    ("Float#magnitude", "abs"),
    ("Float#modulo", "%"),
    ("Float#quo", "fdiv"),
    ("Float#to_int", "to_i"),
    ("Hash#each", "each_pair"),
    ("Hash#filter!", "select!"),
    ("Hash#filter", "select"),
    ("Hash#has_key?", "include?"),
    ("Hash#has_value?", "value?"),
    ("Hash#key?", "include?"),
    ("Hash#length", "size"),
    ("Hash#member?", "include?"),
    ("Hash#store", "[]="),
    ("Hash#to_s", "inspect"),
    ("Hash#update", "merge!"),
    ("Integer#===", "=="),
    ("Integer#inspect", "to_s"),
    ("Integer#magnitude", "abs"),
    ("Integer#modulo", "%"),
    ("Integer#next", "succ"),
    ("Kernel#enum_for", "to_enum"),
    ("Kernel#fail", "raise"),
    ("Kernel#format", "sprintf"),
    ("Kernel#kind_of?", "is_a?"),
    ("Kernel#then", "yield_self"),
    ("Kernel.fail", "raise"),
    ("Kernel.format", "sprintf"),
    ("Marshal.restore", "load"),
    ("MatchData#deconstruct", "captures"),
    ("MatchData#eql?", "=="),
    ("MatchData#length", "size"),
    ("Method#===", "call"),
    ("Method#[]", "call"),
    ("Method#eql?", "=="),
    ("Method#inspect", "to_s"),
    ("Module#class_eval", "module_eval"),
    ("Module#class_exec", "module_exec"),
    ("Module#inspect", "to_s"),
    ("NilClass#^", "|"),
    ("Numeric#conj", "conjugate"),
    ("Numeric#imag", "imaginary"),
    ("Numeric#magnitude", "abs"),
    ("Numeric#modulo", "%"),
    ("ObjectSpace::WeakMap#each_pair", "each"),
    ("ObjectSpace::WeakMap#key?", "include?"),
    ("ObjectSpace::WeakMap#length", "size"),
    ("ObjectSpace::WeakMap#member?", "include?"),
    ("Proc#===", "call"),
    ("Proc#[]", "call"),
    ("Proc#eql?", "=="),
    ("Proc#inspect", "to_s"),
    ("Proc#yield", "call"),
    ("Process.waitpid", "wait"),
    ("Process.waitpid2", "wait2"),
    ("Range#entries", "to_a"),
    ("Range#member?", "include?"),
    ("Rational#magnitude", "abs"),
    ("Rational#quo", "/"),
    ("Regexp#eql?", "=="),
    ("Regexp.escape", "quote"),
    ("Set#+", "|"),
    ("Set#<", "proper_subset?"),
    ("Set#<<", "add"),
    ("Set#<=", "subset?"),
    ("Set#===", "include?"),
    ("Set#>", "proper_superset?"),
    ("Set#>=", "superset?"),
    ("Set#collect!", "map!"),
    ("Set#difference", "-"),
    ("Set#eql?", "=="),
    ("Set#filter!", "select!"),
    ("Set#inspect", "to_s"),
    ("Set#intersection", "&"),
    ("Set#length", "size"),
    ("Set#member?", "include?"),
    ("Set#union", "|"),
    ("String#===", "=="),
    ("String#dedup", "-@"),
    ("String#intern", "to_sym"),
    ("String#next!", "succ!"),
    ("String#next", "succ"),
    ("String#size", "length"),
    ("String#slice", "[]"),
    ("String#to_str", "to_s"),
    ("StringIO#each", "each_line"),
    ("StringIO#eof", "eof?"),
    ("StringIO#isatty", "tty?"),
    ("StringIO#size", "length"),
    ("StringIO#tell", "pos"),
    ("StringScanner#bol?", "beginning_of_line?"),
    ("StringScanner#concat", "<<"),
    ("StringScanner#pointer", "pos"),
    ("StringScanner#pointer=", "pos="),
    ("Struct#deconstruct", "to_a"),
    ("Struct#filter", "select"),
    ("Struct#inspect", "to_s"),
    ("Struct#length", "size"),
    ("Struct#values", "to_a"),
    ("Symbol#===", "=="),
    ("Symbol#id2name", "to_s"),
    ("Symbol#intern", "to_sym"),
    ("Symbol#next", "succ"),
    ("Symbol#size", "length"),
    ("Symbol#slice", "[]"),
    ("Thread#exit", "kill"),
    ("Thread#inspect", "to_s"),
    ("Thread#terminate", "kill"),
    ("Thread.fork", "start"),
    ("Thread::Queue#deq", "pop"),
    ("Thread::Queue#enq", "<<"),
    ("Thread::Queue#length", "size"),
    ("Thread::Queue#push", "<<"),
    ("Thread::Queue#shift", "pop"),
    ("Thread::SizedQueue#deq", "pop"),
    ("Thread::SizedQueue#enq", "<<"),
    ("Thread::SizedQueue#length", "size"),
    ("Thread::SizedQueue#push", "<<"),
    ("Thread::SizedQueue#shift", "pop"),
    ("Time#ctime", "asctime"),
    ("Time#getgm", "getutc"),
    ("Time#gmt?", "utc?"),
    ("Time#gmt_offset", "utc_offset"),
    ("Time#gmtime", "utc"),
    ("Time#gmtoff", "utc_offset"),
    ("Time#isdst", "dst?"),
    ("Time#mday", "day"),
    ("Time#mon", "month"),
    ("Time#tv_nsec", "nsec"),
    ("Time#tv_sec", "to_i"),
    ("Time#tv_usec", "usec"),
    ("Time#xmlschema", "iso8601"),
    ("Time.gm", "utc"),
    ("Time.iso8601", "xmlschema"),
    ("Time.mktime", "local"),
    ("Time.rfc822", "rfc2822"),
    ("TrueClass#inspect", "to_s"),
    ("UnboundMethod#eql?", "=="),
    ("UnboundMethod#inspect", "to_s"),
];

/// The name the core method `key` repeats, where `key` is `Class#name` or
/// `Class.name`.
pub(crate) fn core_alias_target(key: &str) -> Option<&'static str> {
    CORE_ALIASES
        .iter()
        .find(|(alias, _)| *alias == key)
        .map(|(_, target)| *target)
}

/// Whether two Method objects name the same core or standard library
/// method, under either of its names: the name each repeats is the same,
/// and the two were looked up from a class in common.
pub(crate) fn same_core_method(left: &super::Method, right: &super::Method) -> bool {
    match (core_definition(left), core_definition(right)) {
        (Some((left_name, left_places)), Some((right_name, right_places))) => {
            left_name == right_name && left_places.iter().any(|place| right_places.contains(place))
        }
        _ => false,
    }
}

/// The name a core method repeats, with the places it was looked up from,
/// as `Class#` or `Class.`. None for a method written outside the core and
/// the standard libraries.
fn core_definition(method: &super::Method) -> Option<(String, Vec<String>)> {
    if method.c_function.is_some() {
        return None;
    }
    let written_in_core = method.source_location.as_ref().is_none_or(|location| {
        location
            .filename
            .as_deref()
            .is_some_and(|file| file.starts_with("<internal:") || file.starts_with("<metorex>/"))
    });
    if !written_in_core {
        return None;
    }
    let name = method
        .original_name
        .clone()
        .unwrap_or_else(|| method.name.clone());
    // An anonymous class is told apart from another by where it lives.
    let place = |class: &std::rc::Rc<crate::class::Class>| match class.name() {
        "" => format!("{:p}", std::rc::Rc::as_ptr(class)),
        named => named.to_string(),
    };
    let mut places = Vec::new();
    if let Some(receiver) = &method.receiver
        && let super::View::Class(class) | super::View::Module(class) = receiver.view()
    {
        places.push(format!("{}.", place(class)));
    }
    for class in [&method.origin_class, &method.owner_class]
        .into_iter()
        .flatten()
    {
        places.push(format!("{}#", place(class)));
    }
    if let Some(owner) = &method.owner {
        places.push(format!("{owner}#"));
    }
    // A method looked up on an anonymous subclass is the one its nearest
    // named ancestor hands down.
    let mut cursor = method.origin_class.clone();
    while let Some(class) = cursor {
        if !class.name().is_empty() {
            places.push(format!("{}#", class.name()));
            break;
        }
        cursor = class.superclass();
    }
    let repeated = places
        .iter()
        .find_map(|place| core_alias_target(&format!("{place}{name}")))
        .map_or(name, str::to_string);
    Some((repeated, places))
}
