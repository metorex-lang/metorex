# Points activerecord's tests at SQLite, as `rake test:sqlite3` does by
# setting ARCONN before it runs them.
ENV["ARCONN"] ||= "sqlite3"
