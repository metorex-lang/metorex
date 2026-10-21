# A library metorex carries names its files under `<metorex>`, and
# File.expand_path keeps that root, whether it stands alone or before a
# directory.
p File.expand_path("bundler/cli", "<metorex>")
p File.expand_path("imap/sasl", "<metorex>/net")
p File.expand_path("<metorex>/net/imap/../pop")
