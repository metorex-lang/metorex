Dir.chdir("/") { p __dir__.end_with?("tests/_examples/paths") }
def where = __dir__
Dir.chdir("/") { p where.end_with?("tests/_examples/paths") }
