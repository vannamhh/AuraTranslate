#!/bin/zsh
# Interleaved typing-bench sessions (2-15 bench) on app215-after / app215-before, 9850 sentences, 3 each.
HERE="${0:A:h}"; R="$HERE/../2-15-ban-do/run-session.sh"
trap 'echo SENTINEL_TYPING_DONE >> "$HERE/typing.log"' EXIT
for n in 1 2 3; do
  for side in after before; do
    echo "== $side-$n load=$(sysctl -n vm.loadavg)" >> "$HERE/typing.log"
    perl -e 'alarm shift; exec @ARGV' 900 "$R" "t14-$side-$n" "$HERE/app215-$side" 9850 >> "$HERE/typing.log" 2>&1
    echo "rc=$?" >> "$HERE/typing.log"; sleep 5
  done
done
