def first_after(sexp)
  case sexp
  in [:fcall | :method_add_block, s, *] then s
  in [:named, *rest] then rest
  else :none
  end
end

p first_after([:method_add_block, 1, 2])
p first_after([:fcall, 3])
p first_after([:named, 4, 5])
p first_after([:other])
