def report(*values, **options, &block)
  [values, options, block&.call]
end

def forward_all(*, **, &)
  report(
    *,
    **,
    &
  )
end

def forward_block(label, &)
  report(
    label, &
  )
end

p forward_all(1, 2, unit: "kg") { :weighed }
p forward_block("crate") { :counted }
