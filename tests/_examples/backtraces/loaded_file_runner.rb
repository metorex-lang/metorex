module LoadedFileRunner
  def self.load_and_report(path)
    load path
  rescue NameError => error
    error.backtrace.first(2).map { |line| line.sub(/\A.*\//, "") }
  end
end
