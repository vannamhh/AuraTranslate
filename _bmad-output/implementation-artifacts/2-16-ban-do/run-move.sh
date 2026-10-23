#!/bin/zsh
# Usage: run-move.sh <label> <app-dir> <122|9850>   One session: layout time, 3 real mouse clicks (cliclick), 3 segment.next chords. JSON lands in ./ket-qua-<label>.json
set -u
HERE="${0:A:h}"
LABEL="$1"; APPDIR="$2"; N="$3"
source "$HERE/../2-4-ban-do/front.sh"
require_unlocked || exit 2
pgrep -x auratranslate >/dev/null && { echo "app already running"; exit 3; }
HOMEDIR="$HERE/.home-$LABEL"; DATA="$HOMEDIR/Library/Application Support/com.auratranslate.desktop"
rm -rf "$HOMEDIR"; mkdir -p "$HOMEDIR/Documents"
hk() { [ -n "${APID:-}" ] && { require_front "$APID" 6 >/dev/null || { echo "front lost before hk $1: $(front_name)"; exit 5; }; }; osascript -e "tell application \"System Events\" to key code $1 using {control down, option down, shift down}" >/dev/null 2>&1 }
q() { sqlite3 -readonly "$DATA/global.db" "select value from config_value where key='$1';" 2>/dev/null }
stamp() { perl -MTime::HiRes=time -e 'printf "%.4f", time' }
LOAD=$(sysctl -n vm.loadavg)
HOME="$HOMEDIR" perl -e 'alarm shift; exec @ARGV' 1500 "$APPDIR/AuraTranslate.app/Contents/MacOS/auratranslate" >"$HERE/.app-$LABEL.log" 2>&1 &
PID=$!
cleanup() { kill -9 $PID 2>/dev/null; pkill -x auratranslate 2>/dev/null; }
trap cleanup EXIT
for i in $(seq 1 120); do [ -n "$(q __bench_alive__)" ] && break; sleep 0.5; done
[ -n "$(q __bench_alive__)" ] || { echo "no alive mark"; exit 4; }
sleep 2
APID=$(pgrep -x auratranslate | head -1)
WX=200; WY=25; WW=1200; WH=900
osascript -e "tell application \"System Events\" to tell (first application process whose unix id is $APID) to tell window 1 to set size to {$WW, $WH}" >/dev/null 2>&1
osascript -e "tell application \"System Events\" to tell (first application process whose unix id is $APID) to tell window 1 to set position to {$WX, $WY}" >/dev/null 2>&1
require_front "$APID" 10 >/dev/null || { echo "front gate red"; exit 5; }
A0=$(q __bench_alive__)
[ "$N" = 122 ] && hk 12 || hk 13
for i in $(seq 1 240); do [ "$(q __bench_alive__)" != "$A0" ] && break; sleep 0.5; done
[ "$(q __bench_alive__)" != "$A0" ] || { echo "work not created: $(q __b216_err__)"; exit 6; }
sleep 4
require_front "$APID" 10 >/dev/null || { echo "front gate red 2"; exit 5; }
hk 14
for i in $(seq 1 480); do R=$(q __b216_ready__); [ -n "$R" ] && break; sleep 0.5; done
echo "ready=$R"
case "$R" in *,true,*) ;; *) echo "cell not focused"; exit 7;; esac
sleep 3
require_front "$APID" 5 >/dev/null || { echo "front gate red 3"; exit 5; }
hk 15
for i in $(seq 1 40); do TG=$(q __b216_target__); [ -n "$TG" ] && break; sleep 0.5; done
echo "targets=$TG err=$(q __b216_err__)"
LIST=${TG%%v*}; VIS=${TG##*v}
[ -n "$LIST" ] || { hk 32; sleep 3; echo "no targets; dump=$(q __b216__ | head -c 200) front=$(front_name)"; exit 9; }
hk 17   # arm listeners (T)
sleep 1
MS=(); 
for t in ${(s:_:)LIST}; do
  XY=$(to_screen "$t" $WX $WY $WW $WH) || exit 10
  require_front "$APID" 5 >/dev/null || { echo "front gate red 4"; exit 5; }
  S=$(stamp); cliclick c:$XY; MS+=($S)
  sleep 4
done
require_front "$APID" 5 >/dev/null || { echo "front gate red 5"; exit 5; }
KS=$(osascript "$HERE/keys-3.applescript" 4 2>/dev/null)
sleep 3
hk 32
for i in $(seq 1 120); do J=$(q __b216__); [ -n "$J" ] && break; sleep 0.5; done
[ -n "$J" ] || { echo "no result; front=$(front_name) err=$(q __b216_err__)"; exit 8; }
LOAD2=$(sysctl -n vm.loadavg)
print -r -- "$J" | python3 -c "
import json,sys
d=json.load(sys.stdin); d['label']='$LABEL'; d['sentences']=$N; d['load_before']='$LOAD'; d['load_after']='$LOAD2'
d['mouse_sent']=[float(x) for x in '${MS[*]}'.split()]; d['key_sent']=[float(x) for x in '$KS'.split()]; d['visible_cells']=$VIS
json.dump(d,open('$HERE/ket-qua-$LABEL.json','w'))"
echo "saved ket-qua-$LABEL.json"
