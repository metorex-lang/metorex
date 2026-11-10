# attr_reader, attr_writer, and attr_accessor named as local variables, as
# RuboCop's Style/BisectedAttrAccessor does, and passed to a visibility
# method without parentheses.
attr_accessor = "attr_accessor :id, :name\n"
attr_reader, attr_writer = 1, 2
p attr_accessor, attr_reader + attr_writer

class Entry
  private attr_accessor :draft
  protected attr_reader :total, :count
  attr_writer :note
  def summary; end
end
p Entry.private_instance_methods(false).sort
p Entry.protected_instance_methods(false).sort
p Entry.public_instance_methods(false).sort

def reader_line(names)
  attr_reader = "attr_reader #{names.join(', ')}\n"
  attr_reader
end
p reader_line([:id, :name])
