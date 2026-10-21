Kernel.require "bundler/vendor/pub_grub/lib/pub_grub/package"
Kernel.require "bundler/vendor/pub_grub/lib/pub_grub/static_package_source"
Kernel.require "bundler/vendor/pub_grub/lib/pub_grub/term"
Kernel.require "bundler/vendor/pub_grub/lib/pub_grub/version_range"
Kernel.require "bundler/vendor/pub_grub/lib/pub_grub/version_constraint"
Kernel.require "bundler/vendor/pub_grub/lib/pub_grub/version_union"
Kernel.require "bundler/vendor/pub_grub/lib/pub_grub/version_solver"
Kernel.require "bundler/vendor/pub_grub/lib/pub_grub/incompatibility"
Kernel.require "bundler/vendor/pub_grub/lib/pub_grub/solve_failure"
Kernel.require "bundler/vendor/pub_grub/lib/pub_grub/failure_writer"
Kernel.require "bundler/vendor/pub_grub/lib/pub_grub/version"

module Bundler::PubGrub
  class << self
    attr_writer :logger

    def logger
      @logger || default_logger
    end

    private

    def default_logger
      require "logger"

      logger = ::Logger.new(STDERR)
      logger.level = $DEBUG ? ::Logger::DEBUG : ::Logger::WARN
      @logger = logger
    end
  end
end
