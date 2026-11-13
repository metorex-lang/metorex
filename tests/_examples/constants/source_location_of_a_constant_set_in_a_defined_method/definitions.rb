class Definitions
  extend Macro

  define_method(:assign_through_define_method) do
    ::SET_IN_BLOCK = 1
  end

  macro(:assign_through_macro) do
    ::SET_IN_MACRO = 3
  end
end
