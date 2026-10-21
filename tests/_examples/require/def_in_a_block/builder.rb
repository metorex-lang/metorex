BUILD_RECEIVER = proc do
  receiver = Object.new

  def receiver.record(entry)
    entry
  end

  receiver
end
