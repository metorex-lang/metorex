# Names compare by their canonical form, and a copy of a name takes entries
# of its own.
require "openssl"

name = OpenSSL::X509::Name.parse("/C=US/O=Example/CN=Example Root")
same = OpenSSL::X509::Name.parse("/C=US/O=EXAMPLE/CN=example   root")
p(name == same)
p(name.hash == same.hash)
copy = name.dup
copy.add_entry("OU", "Unit")
p(copy.to_s)
p(name.to_s)
p(name == copy)
p(name == same)
