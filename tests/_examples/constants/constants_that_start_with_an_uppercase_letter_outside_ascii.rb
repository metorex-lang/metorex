# A name that starts with an uppercase letter outside ASCII is a constant.
Ω = :omega
Ärger = :anger
module Λ
  Σ = :sigma
  def self.read = Σ
end
p Ω, Ärger, Λ::Σ, Λ.read
p Object.const_defined?(:Ω), defined?(Ω)
ω = :lower
p ω, defined?(ω)
holder = Module.new
holder.autoload(:Ψ, File.join(__dir__, "nothing_here.rb"))
p File.basename(holder.autoload?(:Ψ))
