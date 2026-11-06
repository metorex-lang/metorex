require 'openssl'
["!ALL", "", [], "ALL", "DEFAULT", "ALL:!ALL", "-ALL", "FOO", "FOO:BAR", "!aNULL", nil, "ALL:!aNULL:!eNULL:!SSLv2", "ECDHE-RSA-AES128-GCM-SHA256", "HIGH:!aNULL:!MD5"].each do |list|
  context = OpenSSL::SSL::SSLContext.new
  begin
    context.ciphers = list
    p [list, :accepted]
  rescue OpenSSL::SSL::SSLError => error
    p [list, error.message]
  end
end
