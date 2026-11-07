$greeting = "hello"
@name = "ledger"
"report 42" =~ /(\d+)/
p "#$1 items", "#$greeting world", "#@name entries", "#$~", "#$& found"
p "#@ sign", "##{1}", "#$1#$1", "#$ alone"
class Counter
  @@total = 7
  def self.show = "#@@total total"
end
p Counter.show
p "%(%*)Z".gsub(/%(%*)Z/) { "#$1|" }
"order 77" =~ /(\d+)/
text = <<~TEXT
  heredoc #$1 and #@name
TEXT
p text, %Q(percent #$1), `echo command #$1`.chomp
p(/#$1/.source)
