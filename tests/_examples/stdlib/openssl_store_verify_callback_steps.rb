require "openssl"

CA_PEM = <<~CERT
-----BEGIN CERTIFICATE-----
MIIDVjCCAj6gAwIBAgIBATANBgkqhkiG9w0BAQsFADA8MQswCQYDVQQGEwJKUDES
MBAGA1UECgwJSklOLkdSLkpQMQwwCgYDVQQLDANSUlIxCzAJBgNVBAMMAkNBMB4X
DTI1MDIxMjA4NDIzMVoXDTM1MDIxMDA4NDIzMVowPDELMAkGA1UEBhMCSlAxEjAQ
BgNVBAoMCUpJTi5HUi5KUDEMMAoGA1UECwwDUlJSMQswCQYDVQQDDAJDQTCCASIw
DQYJKoZIhvcNAQEBBQADggEPADCCAQoCggEBAJqxQwHle/gAbPM1QabhCqWiAJDp
XFroixm6HXKe+azptngzrE69sUFO7533lrvUhkfbPHn3JhgPgm1COGqbPJ5V/it7
dRsJKuOUeQz3+iC0Jrbz3WP+5RuIuD2m+qyR2RVPf4nCkbxQn9QJX1DpG13NyDMA
Qp3em5I5GFhECul/VybmC1+6BnJYwe6EJBv7770AAaf9imWsQkW5oTxuKDsL3Iun
+xYgZF1V4Kw8xDiMZOqZYoizhzgQRM180j0emgcDp6owPt/44mEPZX6Y7BhWhBIc
cS+fJulXJzUYtSl6wZqEvMQzHfYXGO4Q1GtNeTrF8wOoevwmpwo+D42sbVUCAwEA
AaNjMGEwDwYDVR0TAQH/BAUwAwEB/zAOBgNVHQ8BAf8EBAMCAQYwHQYDVR0OBBYE
FGjIj2OhmYxPWPibGOIXv3Eq7RWZMB8GA1UdIwQYMBaAFGjIj2OhmYxPWPibGOIX
v3Eq7RWZMA0GCSqGSIb3DQEBCwUAA4IBAQBITdjp9RtGUKjQqcNLa+WxlhHUUgSv
Y2CHTkgG/9LBoe1yCAI0lEMAGOYFFuYc0hsnRSMrrzs7/YTTN4szj+lfUA5EyaSQ
KWBzfKBz4t35YOpRBc5v02FfYxtaMfbo5QcN3XjQkApqK8GFQuwEbm8g7eyJiWlI
frVg0JwVypg87MWgrPCr0/foRl2zO1d0KG/rssAOJNQy3lrrrGyaaFNRcmGrDxsG
59O14d8PCuS7GgaO8DqxirMzRj2PW+kdgNp2+2JTUV7m5Aw6u9yX2rGN+s4zk26P
dfdzSpXKHc1Mj7MyxCSFk51nIhtwx+VsYQeS53RWoDhm2uSvVT/9+xCU
-----END CERTIFICATE-----
CERT

SUBCA_PEM = <<~CERT
-----BEGIN CERTIFICATE-----
MIIDODCCAiCgAwIBAgIBAjANBgkqhkiG9w0BAQsFADA8MQswCQYDVQQGEwJKUDES
MBAGA1UECgwJSklOLkdSLkpQMQwwCgYDVQQLDANSUlIxCzAJBgNVBAMMAkNBMB4X
DTI1MDIxMjA4NDIzMVoXDTM0MDIxMDA4NDIzMVowPzELMAkGA1UEBhMCSlAxEjAQ
BgNVBAoMCUpJTi5HUi5KUDEMMAoGA1UECwwDUlJSMQ4wDAYDVQQDDAVTdWJDQTCC
ASIwDQYJKoZIhvcNAQEBBQADggEPADCCAQoCggEBALmNNo+I5XkgbBTJbvKgJxq5
oiJMEInr64KiCJOsA5Z35oE25a8OIWeRqxKW+jufonF96V/swWNM11gaUcTjYT72
g66ux42vWNN8uFxhhqhYRyiMjfSVA3HUZFaqmU9WP7H3YQGINJZ83BPOYoinZXEP
TrQVa8wIKT9Xc5gOq7+SqUuqLvs4jIkgrB744Fo0soCh5WT3V3Op/5K8MA0+N3P4
VrFm/SP8k7deT3EtT/aOHgkaKLo3hQ8zkDegVFEKfmWyoA/MwTSciioX/ePm7jjb
z4ffl5/ifg3Hgmuk6m85ZaGULJHCkciEgP8scZUNHkRvDK6ZBluwrUpepv5ARUsC
AwEAAaNCMEAwHQYDVR0OBBYEFErc4GxPO8mVXXhTgje5yLxWLh0bMA4GA1UdDwEB
/wQEAwIBBjAPBgNVHRMBAf8EBTADAQH/MA0GCSqGSIb3DQEBCwUAA4IBAQCAcHOU
nvhuV6Vuh4gNZjeLJWapJBVYZdkAOd4EhiWtYpq0wWSqlgE+VzIXV9gccYaulVmf
NJoLuenzML764U8kOBk8mOWg+U5AYOXwRgsU+e56jgWG6/ijLSf7YfaVkVZNPe36
59+0KLag6AkkeDOdXbkfageukg6LhTk4Zg1piRMP3JSYbXVzTlgdm8hLKgPOJitQ
FdjMytGYCU5oGHrRrwhlar0xlIvynzVL5wlkuJ19hKUw4UYjoNFy3ZJtbtdqSaF2
ZGX/8KNDYFoHAyvGZCYTr22T1ywWeMFQ+4beiYEv2lifVcYIoI0bEA8Pe9i3Egiv
9u18A/unCSVLPua1
-----END CERTIFICATE-----
CERT

SERVER_PEM = <<~CERT
-----BEGIN CERTIFICATE-----
MIIDQzCCAiugAwIBAgIBAjANBgkqhkiG9w0BAQsFADA/MQswCQYDVQQGEwJKUDES
MBAGA1UECgwJSklOLkdSLkpQMQwwCgYDVQQLDANSUlIxDjAMBgNVBAMMBVN1YkNB
MB4XDTI1MDIxMjA4NDIzMVoXDTM0MDIxMDA4NDIzMVowQzELMAkGA1UEBhMCSlAx
EjAQBgNVBAoMCUpJTi5HUi5KUDEMMAoGA1UECwwDUlJSMRIwEAYDVQQDDAlsb2Nh
bGhvc3QwggEiMA0GCSqGSIb3DQEBAQUAA4IBDwAwggEKAoIBAQC4SI2h/EryUcj5
yaPn4pOgRSv7VRwSOJX+6bBVmIuGgeSaqZXd9MpnewBhY9g1doOE3PfIVnnZNrFB
FHyJBYD7h6p35g8xyNJQ0HDJNd+xOw+LISoyNmgclGnjbtWzdRlzqnBAOjI7BxOs
MbX0ehkKMcl9VgAJczdGRpQewtDN8JLh8FivHH10txsAch78g7TK4OSfWLSJQg4b
dYIFEGKATKGutBCXHsDpuM1c0CFXU0qiH0FwnjKLJyyUEZg0WgTSn9PftR1PhVZ3
xx07FO+Cgku2QLeClKrJH6/9YsFa3W+AZmujt6Hp77LkBSVWDyYoBHDc+HPcIm0Z
AKg/YuH3AgMBAAGjRjBEMB0GA1UdDgQWBBTZgo2CimpIxBjlzlRwEKfA/3I3GDAO
BgNVHQ8BAf8EBAMCBaAwEwYDVR0lBAwwCgYIKwYBBQUHAwEwDQYJKoZIhvcNAQEL
BQADggEBAJcQ7A7a3XuMbc+eeyAhcoqlWyMpq8ktbVlc4jG7k5dITnXfDYxtMtre
byTezZRc0EWNEVDxmUkTskDfBjl6r3NgZk7lNIONaoMh3KgcyUa4BIjb3t5ifeKj
v+PbXD3G6zOpEfV5TSjnAXL/FVtSbsuLsLfu4TQLpZxEW2yRIW0ftOAsVCa0Ewo5
B78aN2aDrnSgVaTSZ3O+yto/3aVkQFVinCj7uLzoUZVcIQ0maCXTnxYORbAEQlu6
nt0faBMid8RX++niVh41WxmCkpnx6CnLScGdHsgrCho8C4gnm2hxAiKoLBaM88L5
VWt9rVhXmSOX4xeYUgVFfH14y2QMOpc=
-----END CERTIFICATE-----
CERT

ca, subca, server = [CA_PEM, SUBCA_PEM, SERVER_PEM].map { |pem| OpenSSL::X509::Certificate.new(pem) }
def run(label, trusted, leaf, untrusted, answer = nil, time = nil)
  store = OpenSSL::X509::Store.new
  trusted.each { |held| store.add_cert(held) }
  store.time = time if time
  calls = []
  store.verify_callback = proc do |ok, context|
    calls << [ok, context.error_depth, context.error, context.current_cert.subject.to_s, context.chain.size]
    answer.nil? ? ok : answer
  end
  result = store.verify(leaf, untrusted)
  puts "#{label}: #{result} #{store.error} #{store.chain&.size}"
  calls.each { |held| puts "  #{held.inspect}" }
end
run("trusted", [ca], server, [subca])
run("no intermediate", [ca], server, [])
run("no intermediate accepted", [ca], server, [], true)
run("untrusted root", [], server, [subca, ca])
run("untrusted root accepted", [], server, [subca, ca], true)
run("self signed leaf", [], ca, [])
run("expired", [ca], server, [subca], nil, Time.utc(2040))
run("expired accepted", [ca], server, [subca], true, Time.utc(2040))
run("rejected", [ca], server, [subca], false)
run("intermediate trusted", [subca], server, [])
