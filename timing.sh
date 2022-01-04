#!/usr/bin/env bash
# Prints a Markdown table of the size and runtime of each solution.

# Borrowed from bash-cache's bc::_time
runtime() {
  (
    TIMEFORMAT=%R
    time "$@" &> /dev/null
  ) 2>&1
}

row() {
  local binary="target/release/${1}" binary_size
  binary_size=$(stat -c %s "$binary" 2>/dev/null) || return

  printf '| %12s ' \
    "${1#0}" \
    "$(( binary_size / 1024 ))KB" \
    "$(runtime "$binary")s" \
    "$(runtime "$binary")s"
  echo '|'
}

cargo clean --release
cargo build --release

printf '| %12s ' "Day" "Binary Size" "Runtime" "Repeated"; echo '|'
echo '|--------------|--------------|--------------|--------------|'
for i in {01..25}; do
  row "$i"
done