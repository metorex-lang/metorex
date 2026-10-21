require "ostruct"
blank = OpenStruct.allocate
p blank.respond_to?(:name)
