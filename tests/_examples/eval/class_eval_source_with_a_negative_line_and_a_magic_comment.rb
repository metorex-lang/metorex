# class_eval of a String counts its lines from the line it is given, which
# may be below one so that lines the caller added come before line one, and
# a magic comment written with dashes freezes the String literals in it, as
# Tilt's compiled templates rely on.
source = <<~RUBY
  # frozen-string-literal: true
  ::Object.class_eval do
  def compiled_template(locals)
  value = locals[:value]
  [value, "literal".frozen?, __LINE__, caller_locations(0, 1).first.lineno]
  end
  def raising_template
  nil
  raise "from the template"
  end
  end
RUBY
Object.class_eval(source, "template.erb", -2)
p compiled_template(value: 1)
begin
  raising_template
rescue => error
  p error.backtrace.first[/template\.erb:-?\d+/]
end
unfrozen = "def unfrozen_literal = 'literal'.frozen?\n"
Object.class_eval(unfrozen, "plain.rb", 1)
p unfrozen_literal
underscored = "# frozen_string_literal: true\ndef underscored_literal = 'literal'.frozen?\n"
Object.class_eval(underscored, "plain.rb", 1)
p underscored_literal
