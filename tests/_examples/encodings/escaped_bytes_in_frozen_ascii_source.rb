# coding: US-ASCII
# frozen_string_literal: true
# A frozen literal escaping a byte past ASCII in a US-ASCII source is BINARY.

p "\377".encoding
p "\x41".encoding
