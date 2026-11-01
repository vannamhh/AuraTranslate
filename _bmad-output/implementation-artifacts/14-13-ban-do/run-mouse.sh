#!/bin/zsh
# Usage: run-mouse.sh <label> <app-dir>   Real-mouse pass: drag across three sentence numbers, then Shift+click a far one. Result lands in ./ket-qua-mouse-<label>.json
set -u
HERE="${0:A:h}"
LABEL="$1"; APPDIR="$2"
source "$HERE/../2-4-ban-do/front.sh"
require_unlocked || exit 2
pgrep -x auratranslate >/dev/null && { echo "app already running"; exit 3; }
HOMEDIR="$HERE/.home-$LABEL"; DATA="$HOMEDIR/Library/Application Support/com.auratranslate.desktop"
rm -rf "$HOMEDIR"; mkdir -p "$HOMEDIR/Documents"
hk() { [ -n "${APID:-}" ] && { require_front "$APID" 6 >/dev/null || { echo "front lost before hk $1: $(front_name)"; exit 5; }; }; osascript -e "tell application \"System Events\" to key code $1 using {control down, option down, shift down}" >/dev/null 2>&1 }
q() { sqlite3 -readonly "$DATA/global.db" "select value from config_value where key='$1';" 2>/dev/null }
HOME="$HOMEDIR" perl -e 'alarm shift; exec @ARGV' 600 "$APPDIR/AuraTranslate.app/Contents/MacOS/auratranslate" >"$HERE/.app-$LABEL.log" 2>&1 &
PID=$!
cleanup() { kill -9 $PID 2>/dev/null; pkill -x auratranslate 2>/dev/null; }
trap cleanup EXIT
for i in $(seq 1 120); do [ -n "$(q __bench_alive__)" ] && break; sleep 0.5; done
[ -n "$(q __bench_alive__)" ] || { echo "no alive mark"; exit 4; }
sleep 2
APID=$(pgrep -x auratranslate | head -1)
osascript -e "tell application \"System Events\" to tell (first application process whose unix id is $APID) to tell window 1 to set size to {1200, 900}" >/dev/null 2>&1
osascript -e "tell application \"System Events\" to tell (first application process whose unix id is $APID) to tell window 1 to set position to {200, 25}" >/dev/null 2>&1
require_front "$APID" 10 >/dev/null || { echo "front gate red"; exit 5; }
A0=$(q __bench_alive__)
hk 13
for i in $(seq 1 240); do [ "$(q __bench_alive__)" != "$A0" ] && break; sleep 0.5; done
[ "$(q __bench_alive__)" != "$A0" ] || { echo "work not created"; exit 6; }
sleep 4
require_front "$APID" 10 >/dev/null || { echo "front gate red 2"; exit 5; }
hk 14
for i in $(seq 1 480); do R=$(q __b1413m_ready__); [ -n "$R" ] && break; sleep 0.5; done
echo "ready=$R"; [ "$R" = 9850 ] || exit 7
sleep 3
cmdtab() { osascript -e 'tell application "System Events" to key code 48 using command down'; sleep 0.7 }
cmdtab; cmdtab; sleep 1
require_front "$APID" 10 >/dev/null || { echo "front gate red 3"; exit 5; }
rep() { hk 46; sleep 1.2; q __b1413m__ }   # Ctrl+Alt+Shift+M
snap() { sqlite3 "$DATA/global.db" "delete from config_value where key='__b1413m__';" >/dev/null 2>&1; rep }
J0=$(snap)
IH=$(print -r -- "$J0" | python3 -c "import json,sys;print(json.load(sys.stdin)['win']['ih'])")
pt() { print -r -- "$J0" | python3 -c "import json,sys;d=json.load(sys.stdin);v=[r for r in d['visible'] if r['i']==$1][0];print('%d,%d'%(round(200+v['x']),round(25+(900-$IH)+v['y'])))"; }
cc() { require_front "$APID" 5 >/dev/null || { echo "front lost: $(front_name)"; exit 5; }; cliclick -e 40 "$@" }
# warm-up: single clicks are swallowed until the app is active; click a neutral spot (page title area) until a mousedown lands
D0=0
for t in 1 2 3 4 5 6 7 8 9 10; do
  cc m:700,60; sleep 0.2; cc c:700,60; sleep 0.6
  J=$(snap); D=$(print -r -- "$J" | python3 -c "import json,sys;print(json.load(sys.stdin)['downs'])")
  [ "$D" -gt 0 ] && break
  cmdtab; cmdtab
done
echo "warmup downs=$D"; [ "$D" -gt 0 ] || { echo "clicks swallowed"; exit 8; }
J0=$J
P2=$(pt 2); P3=$(pt 3); P4=$(pt 4); PF=$(pt 12)
echo "points $P2 $P3 $P4 far=$PF"
cc m:$P2; sleep 0.25; cc dd:$P2 m:$P3 m:$P4 du:$P4; sleep 0.5
JD=$(snap)
cc m:$PF; sleep 0.25; cc kd:shift c:$PF ku:shift; sleep 0.5
JS=$(snap)
python3 - "$HERE/ket-qua-mouse-$LABEL.json" "$JD" "$JS" <<'PY'
import json,sys
d=json.loads(sys.argv[2]); s=json.loads(sys.argv[3])
json.dump({'afterDrag':{k:d[k] for k in ('selectedIdx','selectedCells','downs','last')},'afterShift':{k:s[k] for k in ('selectedIdx','selectedCells','downs','last')}},open(sys.argv[1],'w'))
print('drag',d['selectedIdx'],d['selectedCells'],d['last']); print('shift',s['selectedIdx'],s['selectedCells'],s['last'])
PY
