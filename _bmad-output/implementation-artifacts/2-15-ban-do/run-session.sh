#!/bin/zsh
# Usage: run-session.sh <label> <app-dir> <122|9850> [gap-seconds]   One session; JSON lands in ./ket-qua-<label>.json
set -u
HERE="${0:A:h}"
LABEL="$1"; APPDIR="$2"; N="$3"; GAP="${4:-0.095}"
source "$HERE/../2-4-ban-do/front.sh"
require_unlocked || exit 2
pgrep -x auratranslate >/dev/null && { echo "app already running"; exit 3; }
HOMEDIR="$HERE/.home-$LABEL"; DATA="$HOMEDIR/Library/Application Support/com.auratranslate.desktop"
rm -rf "$HOMEDIR"; mkdir -p "$HOMEDIR/Documents"
hk() { osascript -e "tell application \"System Events\" to key code $1 using {control down, option down, shift down}" >/dev/null 2>&1 }
q() { sqlite3 -readonly "$DATA/global.db" "select value from config_value where key='$1';" 2>/dev/null }
LOAD=$(sysctl -n vm.loadavg)
HOME="$HOMEDIR" perl -e 'alarm shift; exec @ARGV' 900 "$APPDIR/AuraTranslate.app/Contents/MacOS/auratranslate" >"$HERE/.app-$LABEL.log" 2>&1 &
PID=$!
cleanup() { kill -9 $PID 2>/dev/null; pkill -x auratranslate 2>/dev/null; }
trap cleanup EXIT
for i in $(seq 1 120); do [ -n "$(q __bench_alive__)" ] && break; sleep 0.5; done
[ -n "$(q __bench_alive__)" ] || { echo "no alive mark"; exit 4; }
sleep 2
APID=$(pgrep -x auratranslate | head -1)
require_front "$APID" 10 >/dev/null || { echo "front gate red"; exit 5; }
A0=$(q __bench_alive__)
[ "$N" = 122 ] && hk 12 || hk 13
for i in $(seq 1 240); do [ "$(q __bench_alive__)" != "$A0" ] && break; sleep 0.5; done
[ "$(q __bench_alive__)" != "$A0" ] || { echo "work not created: $(q __b215_err__)"; exit 6; }
sleep 4
require_front "$APID" 10 >/dev/null || { echo "front gate red 2"; exit 5; }
hk 14
for i in $(seq 1 360); do R=$(q __b215_ready__); [ -n "$R" ] && break; sleep 0.5; done
echo "ready=$R"
case "$R" in *,true,*) ;; *) echo "cell not focused"; exit 7;; esac
sleep 3
require_front "$APID" 5 >/dev/null || { echo "front gate red 3"; exit 5; }
T0=$(perl -MTime::HiRes=time -e 'printf "%.3f", time')
SENT=$(osascript "$HERE/type-10hz.applescript" "$GAP" 2>/dev/null)
T1=$(perl -MTime::HiRes=time -e 'printf "%.3f", time')
sleep 6; [ -n "${DIAG:-}" ] && { screencapture -x "/private/tmp/claude-501/-Users-hoangnam-LocalSites-addon-AuraTranslate/0633bc7a-ae1d-47ea-b42e-6f69cbf47b45/scratchpad/pre-$LABEL.png"; ps -o %cpu,rss,state -p $APID; }
for i in $(seq 1 1800); do J=$(q __b215__); [ -n "$J" ] && break; sleep 0.5; done
[ -n "$J" ] || { screencapture -x "/private/tmp/claude-501/-Users-hoangnam-LocalSites-addon-AuraTranslate/0633bc7a-ae1d-47ea-b42e-6f69cbf47b45/scratchpad/fail-$LABEL.png"; echo "no result; front=$(front_name) err=$(q __b215_err__)"; exit 8; }
LOAD2=$(sysctl -n vm.loadavg)
print -r -- "$J" | python3 -c "
import json,sys
d=json.load(sys.stdin); d['label']='$LABEL'; d['sentences']=$N; d['sender_wall_s']=round($T1-$T0,2); d['load_before']='$LOAD'; d['load_after']='$LOAD2'; d['gap']=$GAP; d['sent']=[float(x) for x in '$SENT'.split()]
json.dump(d,open('$HERE/ket-qua-$LABEL.json','w'))"
echo "saved ket-qua-$LABEL.json sender ${T1}-${T0}"
