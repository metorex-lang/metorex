values = { :"\#$SAFE" => "A", a: "a" }
p "#{values[:"\#$SAFE"]}"
p "\#{} #{values[:a]}"
p "x#{"\#$y"}z"
source = <<-'RUBY'
  def joined(v = {})
    "\#{} #{((t = v[:a]) && t.respond_to?(:call) ? t.call : t)||(v.has_key?(:a) && '')}"
  end
RUBY
holder = Object.new
holder.instance_eval(source)
p holder.joined(a: "a")
keyed = <<-'RUBY'
  def keyed(v = {})
    "#{((t = v[:"\#$SAFE"]) && t.respond_to?(:call) ? t.call : t)}"
  end
RUBY
holder.instance_eval(keyed)
p holder.keyed(:"\#$SAFE" => "A")
opened = { :"\#{" => "B" }
p "#{opened[:"\#{"]}"
p "\"\#{ab#{values[:a]}c}\""
p "#{"\#{"}"

"before-middle-after" =~ /middle/
p "#{$`.upcase}|#{$'.upcase}"
