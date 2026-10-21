require "yaml"

text = <<~'YAML'
  first: "RUBY0001 OK [Error=\"Exchange:
    Cross Server access\"
    Service=Imap4] done.\r\n"
  second: '[Error="Exchange: Cross
    access" more

    after a blank line'
  third: "ends \
    joined"
  fourth: plain
YAML
p YAML.load(text)
