#!/bin/zsh
set -u
S="${0:A:h}"; trap 'echo SENTINEL_CTRL_DONE' EXIT
for r in 1 2 3; do
  for c in none noop sleep; do
    if [ $c = none ]; then V=0; C=; else V=0; C=$c; fi
    VARIANT=$V CTRL=$C perl -e 'alarm shift; exec @ARGV' 300 "$S/keycost-session.sh" "c${c}" $r 30 2>&1 | grep -E "RIG_FAIL|OK"
  done
done
