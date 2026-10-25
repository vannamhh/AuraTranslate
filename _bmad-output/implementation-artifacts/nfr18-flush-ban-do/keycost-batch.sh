#!/bin/zsh
# keycost-batch.sh <nhãn-gốc> <số-phiên-40hz> <số-phiên-10hz> [giây]
set -u
S="${0:A:h}"; L="$1"; N40="$2"; N10="$3"; DUR="${4:-30}"
trap 'echo SENTINEL_BATCH_DONE' EXIT
for i in $(seq 1 $N40); do
  perl -e 'alarm shift; exec @ARGV' 300 "$S/keycost-session.sh" "${L}40" $i $DUR 2>&1 | tail -4; done
for i in $(seq 1 $N10); do
  RATE=10hz perl -e 'alarm shift; exec @ARGV' 300 "$S/keycost-session.sh" "${L}10" $i $DUR 2>&1 | tail -4; done
