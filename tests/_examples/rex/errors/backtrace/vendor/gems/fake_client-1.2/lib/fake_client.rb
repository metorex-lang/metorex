module FakeClient
  def self.fetch(path)
    raise IOError, "connection refused for #{path}"
  end
end
