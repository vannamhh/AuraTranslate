#!/bin/zsh
# keycost-variants.sh <vòng> : xen kẽ biến thể 0..5 ở 40 phím/giây, 30 s mỗi phiên
set -u
S="${0:A:h}"; trap 'echo SENTINEL_VARIANTS_DONE' EXIT
for r in $(seq 1 $1); do
  for v in ${=VLIST:-0 1 2 3 4 5}; do
    VARIANT=${v%%[a-z]*} CTRL=${v##[0-9]} perl -e 'alarm shift; exec @ARGV' 300 "$S/keycost-session.sh" "w${v}" $r 30 2>&1 | grep -E "RIG_FAIL|OK|loadavg_before"
  done
done
