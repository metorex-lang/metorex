# A modifier `if` whose condition joins `defined?` with `and` and carries on
# past a line break, as rexml's ParseException#line is written.
class Source
  def current_line = [12, 0, 1]
end

class Report
  def initialize = @source = Source.new

  def line
    @source.current_line[2] if @source and defined? @source.current_line and
    @source.current_line
  end
end

p Report.new.line
