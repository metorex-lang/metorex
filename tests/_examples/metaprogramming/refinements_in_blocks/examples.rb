module Reports
  describe do
    using Shout
    it { "ready".shout }
    $groups << self
  end
end
