#!/bin/zsh
trap 'echo AB_DONE' EXIT
cd "${0:A:h}/.."
for i in 1 2 3 4 5 6; do
  for v in A B; do
    echo "== $v $i load: $(sysctl -n vm.loadavg)"
    APP_BIN="$PWD/ab/$v.app/Contents/MacOS/auratranslate" BENCH_SRC=$PWD/chapters/s122.txt perl -e 'alarm shift; exec @ARGV' 600 ./kill-campaign-v2.sh 1 ab$v$i 20 55 3 >ab/run-$v$i.out 2>&1
    echo "exit $?"
  done
done
