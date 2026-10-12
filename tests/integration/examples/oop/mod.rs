// The example scripts this topic covers, each run and compared against
// the output it is expected to write.

mod attributes;
mod classes;
mod copying;
mod method_tables;
mod modules;
mod operators;
mod singletons;
mod super_calls;
mod visibility;

/// The expected output of both `oop/refinement_names/target` variants.
pub(super) const REFINEMENT_NAMES_OUTPUT: &str = concat!(
    "Held\n",
    "Held\n",
    "Module\n",
    "\"counted\"\n",
    "nil\n",
    "#<Encoding:US-ASCII>\n",
    "#<Encoding:UTF-8>\n",
    "#<Encoding:US-ASCII>\n",
    "#<Encoding:BINARY (ASCII-8BIT)>\n",
    "\"あ\"\n"
);

/// The expected output of both `oop/singleton_on_a_collection` variants, which
/// differ only in whether the calls are written with parentheses.
pub(super) const SINGLETON_ON_A_COLLECTION_OUTPUT: &str =
    "[2, 4]\ntrue\nfalse\ntrue\nfalse\n77\n1\n";

/// The expected output of both `oop/basic_object/answers_nothing` variants, which show
/// what an object rooted at BasicObject answers, and what it takes on from Kernel and differ only in whether the calls are
/// written with parentheses.
pub(super) const BASIC_OBJECT_ANSWERS_NOTHING_OUTPUT: &str = "undefined method 'to_s' for an instance of BasicObject\nfalse\nBasicObject\nfalse\n[true, false]\n";

/// The expected output of both `oop/class_variable_home` variants.
pub(super) const CLASS_VARIABLE_HOME_OUTPUT: &str = concat!(
    "module\n",
    "written\n",
    "from_child\n",
    "true\n",
    "class variable access from toplevel\n"
);

/// The expected output of both `oop/class_body_clauses` variants.
pub(super) const CLASS_BODY_CLAUSES_OUTPUT: &str = "body\nstopped\nmodule ensured\nsingleton stopped\nTrueClass\nNilClass\ncan't define singleton\n";

/// The expected output of both `oop/definition_values` variants.
pub(super) const DEFINITION_VALUES_OUTPUT: &str = "20\nheld\nnil\n[:@tally]\n[:@@shared]\nsuperclass must be an instance of Class (given an instance of String)\ninner\n";

/// The expected output of both `oop/module_reopening` variants, which differ
/// only in whether the calls are written with parentheses.
pub(super) const MODULE_REOPENING_OUTPUT: &str =
    "Named is not a module\nInner is not a module\nfalse\n\"NamedRoot\"\n\"NamedRoot::Second\"\n";

pub(super) const CASE_EQUALITY_OF_CLASSES_OUTPUT: &str = concat!(
    "Module === String -> true\n",
    "Class === String -> true\n",
    "String === String -> false\n",
    "Module === M -> true\n",
    "Class === M -> false\n",
    "C === D -> false\n",
    "C === C -> false\n",
    "Object === C -> true\n",
    "Comparable === Integer -> false\n",
    "Module === Comparable -> true\n",
    "true\ntrue\n",
    "written here: [1]\n",
    "written here: [1, 2]\n",
);

pub(super) const REFINEMENT_INDIRECT_CALLS_OUTPUT: &str = concat!(
    "[\"refined\", \"refined\", \"refined\", \"refined\", \"refined\", \"refined\"]\n",
    "plain\n",
    "true\n",
    "refined\n",
    "true\n",
    "from the module refinement\n",
    "from the subclass\n",
);
