# An anonymous *, ** or & parameter may end its line, with the list going
# on or closing on the next, as in Active Record's TableDefinition.
class TableDefinition
  def initialize(
    name,
    temporary: false,
    **
  )
    @name = name
    @temporary = temporary
  end

  def columns(
    first,
    * # the rest
  )
    first
  end

  def each_column(
    &
  )
    [:id, :name].each(&)
  end

  attr_reader :name, :temporary
end
table = TableDefinition.new("users", temporary: true, if_not_exists: true)
p [table.name, table.temporary]
p table.columns(:id, :name, :email)
table.each_column { |column| p column }
p TableDefinition.instance_method(:initialize).parameters
