# Runs a file's tests one at a time. Minitest runs the tests of a class that
# calls `parallelize_me!` on as many threads as there are processors, and an
# isolated test forks a process for each, as actionpack's tests fork a worker
# for each MT_CPU. sprockets-rails' isolated tests all build their app in the
# same tmp/app, so run together they remove each other's files. The Rails
# frameworks' tests run in forked processes while the runner runs other files
# beside them, which together take more memory than the machine has.
ENV["MT_CPU"] = "1"
