# Builds the gem in ledger_totals/ and writes the compact index Bundler reads
# a gem source through into repository/: `versions`, `info/<gem>`, and the
# gem under `gems/`. Run under MRI, which builds gems:
#
#   ruby tests/_examples/bundler/build_repository.rb
require "digest"
require "fileutils"
require "rubygems/package"

here = __dir__
repository = File.join(here, "repository")
spec = Dir.chdir(File.join(here, "ledger_totals")) do
  spec = Gem::Specification.load("ledger_totals.gemspec")
  file = Gem::Package.build(spec)
  FileUtils.mv(file, File.join(repository, "gems", file))
  spec
end
gem_file = File.join(repository, "gems", "#{spec.full_name}.gem")
info = "---\n#{spec.version} |checksum:#{Digest::SHA256.file(gem_file).hexdigest}\n"
File.write(File.join(repository, "info", spec.name), info)
File.write(File.join(repository, "versions"),
           "created_at: 2026-10-21T00:00:00Z\n---\n#{spec.name} #{spec.version} #{Digest::MD5.hexdigest(info)}\n")
