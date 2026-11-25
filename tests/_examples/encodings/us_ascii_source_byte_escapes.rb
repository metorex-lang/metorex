# encoding: US-ASCII
# frozen_string_literal: true
p "a\xC3\xA9".encoding
p "abc".encoding
p "\x41".encoding
p "x#{1}\xC3\xA9".encoding
p "\u00e9".encoding
p %Q(\xC3\xA9).encoding
p "caf\xC3\xA9".bytes
p "a\xC3\xA9".valid_encoding?
