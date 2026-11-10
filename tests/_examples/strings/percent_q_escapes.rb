# frozen_string_literal: true
# A %Q literal reads escapes as a double-quoted string does: a backslash
# before a character that opens no escape stands for that character, one at
# the end of a line joins the lines, \c, \C-, and \M- name control and meta
# bytes, and the literal is frozen under the frozen_string_literal comment.
p %Q{<<\"EOS\n\nEOS\n}
p %Q{a\
b}, %Q{\d\w}, %Q{a\}b}
p %Q{\cA\C-b\M-a}.bytes, %Q{\xff\xfe}.bytes, %Q{a\xffb}.valid_encoding?
p %Q{x}.frozen?, %Q{x#{1}}.frozen?
