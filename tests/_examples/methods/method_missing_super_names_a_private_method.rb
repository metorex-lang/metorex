class Topic
  def method_missing(name, *arguments, &block)
    super
  end

  def respond_to_missing?(name, include_private = false)
    super
  end

  private

  def title
    "hidden"
  end

  protected

  def summary
    "guarded"
  end
end

topic = Topic.new
[-> { topic.title }, -> { topic.summary }, -> { topic.missing }].each do |call|
  call.call
rescue NoMethodError => error
  p error.message
end
p topic.send(:title)

generated = Module.new
generated.module_eval("def subject; :value; end")
Topic.include(generated)
p topic.subject
generated.send(:undef_method, :subject)
begin
  topic.subject
rescue NoMethodError => error
  p error.message
end
