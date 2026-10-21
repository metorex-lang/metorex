# frozen_string_literal: true

require "rake/clean"
CLOBBER.include "pkg"

Kernel.require "bundler/gem_helper"
Bundler::GemHelper.install_tasks
