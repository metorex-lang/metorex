# frozen_string_literal: true
# A frozen literal holding an escaped byte that spells no UTF-8 is frozen.
held = "\xff"
p held.frozen?, held.encoding, held.valid_encoding?
