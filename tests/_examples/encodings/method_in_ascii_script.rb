# coding: US-ASCII
# A method the main script defines writes its literals in the script's
# encoding when it is called, after the script's top level has finished.

class Sample
  def escaped = "\377"
  def plain = "a"
end

p Sample.new.escaped.encoding, Sample.new.plain.encoding
at_exit { p Sample.new.escaped.encoding, Sample.new.plain.encoding }
