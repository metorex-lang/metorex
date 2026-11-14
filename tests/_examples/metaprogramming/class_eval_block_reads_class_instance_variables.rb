# frozen_string_literal: true

module Registry
  class Listing
    def self.default
      @default ||= "loaded"
    end

    def self.default=(value)
      @default = value
    end
  end
end

p Registry::Listing.class_eval { @default }
Registry::Listing.default
p Registry::Listing.class_eval { @default }
p Registry::Listing.instance_eval { @default }
Registry::Listing.default = nil
p Registry::Listing.class_eval { @default }
Registry::Listing.default
p(class Registry::Listing; @default; end)
p Registry::Listing.module_exec { @default }
