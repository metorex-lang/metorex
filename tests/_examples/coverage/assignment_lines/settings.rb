Settings = Struct.new(:headers)
config = Settings.new
config.headers = {
  "Cache-Control" => "max-age=#{60 * 60}"
}
