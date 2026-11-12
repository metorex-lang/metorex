# A `{` that starts the statement after a semicolon opens a Hash, whatever
# the statement before it ended with.
count = 1
count; { "retries" => 3 }.each { |key, value| p [key, value] }

def ready = true
ready; { "ready" => ready }.each_pair { |key, value| p [key, value] }

values = []
index = 0
while index < 2; index; { "index" => index }.each_value { |held| values << held }; index += 1; end
p values
