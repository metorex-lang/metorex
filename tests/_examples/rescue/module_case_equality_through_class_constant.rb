module Catching
  def self.===(exception)
    exception.is_a?(Exception)
  end
end

class Runner
  def run
    yield
  rescue Catcher => error
    [:caught, error.message]
  end

  if RUBY_VERSION.to_f < 1.9
    Catcher = Exception
  else
    Catcher = Catching
  end
end

class Strict < Exception; end
p Runner::Catcher
p Runner.new.run { raise Strict, "deep" }
