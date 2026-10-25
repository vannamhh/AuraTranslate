#!/bin/zsh
# keycost-probe.sh <rounds>: interleave real cell (pb0), probe outside the grid (pb8), probe next to the real cell (pb9), unfocused probe control (pb10); 40 keys/s, 30 s.
S="${0:A:h}"; trap 'echo SENTINEL_PROBE_DONE' EXIT
for r in $(seq 1 $1); do
  for spec in "pb0:" "pb8:24" "pb9:30" "pb10:33"; do
    PROBE=${spec#*:} perl -e 'alarm shift; exec @ARGV' 300 "$S/keycost-session.sh" "${spec%%:*}" $r 30 2>&1 | grep -E "RIG_FAIL|OK|loadavg_before"
  done
done
