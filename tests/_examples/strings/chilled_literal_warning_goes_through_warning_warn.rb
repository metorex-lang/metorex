# Changing a literal string that a later release will freeze warns through
# `Warning.warn`, in the deprecated category.
Warning[:deprecated] = true
module Warning
  def self.warn(message, category: nil)
    STDOUT.puts "#{category.inspect}: #{message.sub(/\A.*?: warning:/, "warning:")}"
  end
end

label = "crate"
label << "s"
p label
