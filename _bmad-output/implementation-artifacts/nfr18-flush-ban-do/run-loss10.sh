#!/bin/zsh
S="${0:A:h}"; trap 'echo SENTINEL_LOSS10B_DONE' EXIT
cd "$S"
RATE=10hz BENCH_SRC=$S/chapters/s122.txt perl -e 'alarm shift; exec @ARGV' 4800 ./kill-campaign-v2.sh 20 l10b 20 55 28
