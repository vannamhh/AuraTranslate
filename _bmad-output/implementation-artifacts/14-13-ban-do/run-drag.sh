#!/bin/zsh
# Usage: run-drag.sh <label> <app-dir>   One session: create 9850-row Work, open Workspace, run drag sweeps. JSON lands in ./ket-qua-<label>.json
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
LOAD=$(sysctl -n vm.loadavg)
HOME="$HOMEDIR" perl -e 'alarm shift; exec @ARGV' 900 "$APPDIR/AuraTranslate.app/Contents/MacOS/auratranslate" >"$HERE/.app-$LABEL.log" 2>&1 &
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
hk 13   # Ctrl+Alt+Shift+W
for i in $(seq 1 240); do [ "$(q __bench_alive__)" != "$A0" ] && break; sleep 0.5; done
[ "$(q __bench_alive__)" != "$A0" ] || { echo "work not created: $(q __b1413_err__)"; exit 6; }
sleep 4
require_front "$APID" 10 >/dev/null || { echo "front gate red 2"; exit 5; }
hk 14   # E
for i in $(seq 1 480); do R=$(q __b1413_ready__); [ -n "$R" ] && break; sleep 0.5; done
echo "ready=$R"
case "$R" in 9850) ;; *) echo "workspace not ready"; exit 7;; esac
sleep 3
require_front "$APID" 5 >/dev/null || { echo "front gate red 3"; exit 5; }
hk 2    # D
for i in $(seq 1 240); do D=$(q __b1413_done__); [ -n "$D" ] && break; sleep 0.5; done
[ -n "$D" ] || { echo "sweeps not done; err=$(q __b1413_err__)"; exit 9; }
sleep 1
hk 32   # U
for i in $(seq 1 60); do J=$(q __b1413__); [ -n "$J" ] && break; sleep 0.5; done
[ -n "$J" ] || { echo "no result; err=$(q __b1413_err__)"; exit 8; }
LOAD2=$(sysctl -n vm.loadavg)
print -r -- "$J" | python3 -c "
import json,sys
d=json.load(sys.stdin); d['label']='$LABEL'; d['load_before']='$LOAD'; d['load_after']='$LOAD2'
json.dump(d,open('$HERE/ket-qua-$LABEL.json','w'))"
echo "saved ket-qua-$LABEL.json"
