def option(name, type:, default:) = [name, type, default]

p option :level,
         type:
           [:fatal, :warn],
         default: :info
p(option(:mode,
         type:
           :plain,
         default: 1))
type = :held
default = 2
p option(:short, type:, default:)

def collect(**keywords) = keywords

level = 1
detail = 2
gathered = collect level:
detail
p gathered
p(collect(level:
))
