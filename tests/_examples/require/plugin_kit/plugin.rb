module Kit
  module Plugin
    def plugin_name = name.split("::").last.downcase
  end
end
