# The gems the specs of the aws-sdk-ruby repository's gems load, which each
# of those gems' Gemfiles names. The XML and JSON engines with C extensions,
# ox, oga, nokogiri, libxml-ruby, and oj, are left out, so their specs skip
# under either engine.
eval_gemfile File.expand_path("ruby_shipped.rb", __dir__)

AWS_SDK_RUBY = File.expand_path("../../../gems/aws-sdk-ruby", __dir__)

%w[aws-sdk-core aws-sigv4 aws-partitions aws-eventstream aws-sdk-s3 aws-sdk-kms].each do |name|
  gem name, path: "#{AWS_SDK_RUBY}/gems/#{name}"
end
gem "aws-sdk-code-generator", path: "#{AWS_SDK_RUBY}/build_tools/aws-sdk-code-generator"
gem "jmespath", path: File.expand_path("../../../gems/jmespath", __dir__)
gem "rspec"
gem "webmock"
gem "opentelemetry-sdk"
gem "http-2"
gem "addressable"
gem "multipart-post"

shipped "base64", "bigdecimal", "logger", "rexml", "json", "ostruct"
