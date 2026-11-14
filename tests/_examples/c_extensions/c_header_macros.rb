# The macros ruby.h gives a C extension: the HAVE_RUBY_*_H flags, so a
# string the extension makes with rb_enc_str_new is UTF-8, RUBY_METHOD_FUNC,
# and RETURN_ENUMERATOR, which answers an Enumerator when no block is given.
require "tmpdir"
require_relative "build_helper"

Dir.mktmpdir do |directory|
  require build_extension("c_header_macros.c", "c_header_macros", directory)

  text = HeaderMacros.encoded_text
  p [text, text.encoding]

  pairs = HeaderMacros.each_pair_of(:left, :right)
  p pairs.class, pairs.to_a
  HeaderMacros.each_pair_of(1, 2) { |held| p held }
end
