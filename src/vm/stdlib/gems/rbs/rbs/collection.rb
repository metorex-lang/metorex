# frozen_string_literal: true

require 'yaml'
require 'fileutils'

require "rbs/cli/colored_io"
require "rbs/collection/sources"
require "rbs/collection/config"
require "rbs/collection/config/lockfile"
require "rbs/collection/config/lockfile_generator"
require "rbs/collection/installer"
require "rbs/collection/cleaner"

module RBS
  module Collection
  end
end
