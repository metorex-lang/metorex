# A class and a module defined by a plain require of the file their
# autoloads name are no autoloads any more, also once the directory the
# require found the file in leaves the load path.
library = File.join(__dir__, "library")
$LOAD_PATH << library
Object.autoload :RemovedDirectoryTemplate, "removed_directory_template"
Object.autoload :RemovedDirectoryHelpers, "removed_directory_template"
p require("removed_directory_template")
$LOAD_PATH.delete(library)
p Object.autoload?(:RemovedDirectoryTemplate)
p Object.autoload?(:RemovedDirectoryHelpers)
p [RemovedDirectoryTemplate, RemovedDirectoryHelpers]
