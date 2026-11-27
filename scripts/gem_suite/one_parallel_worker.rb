# Runs actionview's tests in one process. Its test helper calls Rails'
# `parallelize`, which forks a worker for each processor, and those workers
# beside the runner's other files take more memory than the machine has.
# railties leaves it unset, since its tests check what parallel testing does
# in the applications they generate.
ENV["PARALLEL_WORKERS"] = "1"
