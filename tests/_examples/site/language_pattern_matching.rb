def describe(response)
  case response
  in { status: 200, body: { users: [] } }
    "no users"
  in { status: 200, body: { users: [{ name: String => name }, *rest] } }
    "#{name} and #{rest.size} more"
  in { status: 400..499 => code }
    "client error #{code}"
  in { status: Integer => code } if code >= 500
    "server error #{code}"
  end
end

puts describe({ status: 200, body: { users: [] } })
puts describe({ status: 200, body: { users: [{ name: "Ada" }, { name: "Grace" }] } })
puts describe({ status: 404 })
puts describe({ status: 503 })

config = { db: { host: "localhost", port: 5432 } }
config => { db: { port: } }
p port
