#!/usr/bin/env bash
# Prints a Markdown table of the size and runtime of each solution.

accum=()

# Borrowed from bkt's benchmark script
avg_floats() {
  python <(cat <<EOF
import sys
total = sum((float(arg) for arg in sys.argv[1:]))
print("{:.3f}".format(total/(len(sys.argv)-1)))
EOF
    ) "$@"
}

# Borrowed from bkt's benchmark script
sum_floats() {
  python <(cat <<EOF
import sys
total = sum((float(arg) for arg in sys.argv[1:]))
print("{:.3f}".format(total))
EOF
    ) "$@"
}

# Borrowed from bash-cache's bc::_time
runtime() {
  (
    TIMEFORMAT=%R
    time "$@" &> /dev/null
  ) 2>&1
}

row() {
  local binary="target/release/${1}" binary_size runtimes=() avg_runtime
  binary_size=$(stat -c %s "$binary" 2>/dev/null) || return

  for i in {0..3}; do
    runtimes+=("$(runtime "$binary")")
  done

  # don't count the first run (:1 syntax) as it can be slower on some systems
  avg_runtime=$(avg_floats "${runtimes[@]:1}")
  accum+=("$avg_runtime")

  printf '| %12s ' \
    "${1#0}" \
    "$(( binary_size / 1024 ))KB" \
    "${avg_runtime}s" \
    "$(sum_floats "${accum[@]}")s"
  echo '|'
}

cargo clean --release
cargo build --release

printf '| %12s ' "Day" "Binary Size" "Runtime" "Accumulated"; echo '|'
echo '|--------------|--------------|--------------|--------------|'
for i in {01..25}; do
  row "$i"
done