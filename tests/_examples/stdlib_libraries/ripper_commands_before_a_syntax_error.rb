require "ripper"

class RenderCalls < Ripper
  attr_reader :seen

  def initialize(source)
    super
    @seen = []
  end

  def on_ident(name) = name
  def on_ivar(name) = name
  def on_var_ref(name) = name
  def on_tstring_content(text) = text
  def on_string_add(_content, text) = text
  def on_string_literal(text) = text
  def on_args_new = []
  def on_args_add(list, item) = list + [item]
  def on_args_add_block(list, _block) = list

  def on_command(name, arguments)
    @seen << [name, arguments]
  end

  def on_parse_error(message)
    @seen << [:error, message]
  end
end

[
  "buffer.append=( render'messages/message' )",
  "buffer.append=  render 'messages/layout' do ;\nbuffer",
  "render 'messages/form'",
  "render@section"
].each do |source|
  parser = RenderCalls.new(source)
  parser.parse
  p parser.seen
end
