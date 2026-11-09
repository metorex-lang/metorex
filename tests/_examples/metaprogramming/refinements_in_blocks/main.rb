module Shout
  refine String do
    def shout = upcase + "!"
  end
end

class Group
  def self.describe(&block) = Class.new(self).module_exec(&block)
  def self.it(&block) = (@examples ||= []) << block
  def self.examples = @examples
end

class Module
  def describe(&block) = Group.describe(&block)
end

def run_block = yield

$groups = []
load File.join(__dir__, "examples.rb")

group = $groups.first
p(group.examples.map(&:call))
p(group.examples.map { |example| group.new.instance_exec(&example) })
p(group.examples.map { |example| group.class_exec(&example) })

run_block do
  class Speaker
    using Shout
    def self.say = "hello".shout
  end

  module Announcer
    using Shout
    def self.say = "welcome".shout
  end
end

p Speaker.say
p Announcer.say
