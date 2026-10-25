#!/bin/zsh
set -u
S="${0:A:h}"; trap 'echo SENTINEL_CTRL2_DONE' EXIT
for r in 1 2 3; do
  for c in none attr style v3; do
    V=0; C=$c; L=d$c
    [ $c = none ] && C=
    [ $c = v3 ] && { V=3; C=; }
    VARIANT=$V CTRL=$C perl -e 'alarm shift; exec @ARGV' 300 "$S/keycost-session.sh" "$L" $r 30 2>&1 | grep -E "RIG_FAIL|OK"
  done
done
