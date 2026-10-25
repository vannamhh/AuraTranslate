#!/bin/zsh
# Một phiên đo chi phí mỗi phím (plan giảm-chi-phí-mỗi-phím-gõ-lưới): dựng Tác phẩm, gõ DUR giây,
# bấm Ctrl+Alt+Shift+8 để bench.js đổ bảng `__bench_keys__`, đọc ra JSON rồi giết app.
# Dùng: keycost-session.sh <nhãn> <phiên> <giây>   (RATE=10hz cho 10 phím/giây)
set -u
LABEL="$1"; N="$2"; DUR="${3:-30}"
SCRATCH="${0:A:h}"
APP="${APP_BIN:-/Users/hoangnam/LocalSites/addon/AuraTranslate/src-tauri/target/release/bundle/macos/AuraTranslate.app/Contents/MacOS/auratranslate}"
DRAFT_HOME="$SCRATCH/draft-home"
APPDATA="$DRAFT_HOME/Library/Application Support/com.auratranslate.desktop"
LIB="$DRAFT_HOME/Documents/AuraTranslate"
SRC_FILE="${BENCH_SRC:-$SCRATCH/chapters/s122.txt}"
WIN_X=200; WIN_Y=25
source "$SCRATCH/front.sh"
now() { perl -MTime::HiRes=time -e 'printf "%.3f\n", time' }
OUT="$SCRATCH/keys-$LABEL-$N"
trap 'echo "SENTINEL_SESSION_DONE $LABEL $N"' EXIT

rm -rf "$LIB" "$APPDATA"; mkdir -p "$DRAFT_HOME/Documents"
HOME="$DRAFT_HOME" "$APP" > "$OUT.app.log" 2>&1 &
PID=$!
for i in $(seq 1 60); do [ -f "$APPDATA/global.db" ] && break; sleep 0.5; done
sleep 2
"$SCRATCH/setup-gui.sh" $PID "$SRC_FILE" || { echo "RIG_FAIL setup-gui"; kill -9 $PID; exit 2; }
for i in $(seq 1 80); do [ -n "$(find "$LIB" -name project.db 2>/dev/null | head -1)" ] && break; sleep 0.5; done
DB=$(find "$LIB" -name project.db 2>/dev/null | head -1)
[ -z "$DB" ] && { echo "RIG_FAIL no work"; kill -9 $PID; exit 2; }
sleep 4
require_front $PID 6 >/dev/null || { echo "RIG_FAIL front1"; kill -9 $PID; exit 2; }
cliclick m:$((WIN_X + 101)),$((WIN_Y + 46)) w:300 c:$((WIN_X + 101)),$((WIN_Y + 46))
sleep 4
require_front $PID 6 >/dev/null || { echo "RIG_FAIL front2"; kill -9 $PID; exit 2; }
HIT=$("$SCRATCH/focus-segment.sh" $PID "$DB" 2>/dev/null) || { echo "RIG_FAIL focus"; kill -9 $PID; exit 2; }
echo "focus +$HIT"
V="${VARIANT:-0}"
for v in $( [ "$V" -gt 0 ] && seq 1 $V ); do
  osascript -e 'tell application "System Events" to key code 25 using {control down, option down, shift down}' >/dev/null 2>&1; sleep 0.7
done
if [ "${CTRL:-}" = "attr" ]; then
  osascript -e 'tell application "System Events" to key code 29 using {control down, option down, shift down}' >/dev/null 2>&1; sleep 0.7
elif [ "${CTRL:-}" = "style" ]; then
  osascript -e 'tell application "System Events" to key code 27 using {control down, option down, shift down}' >/dev/null 2>&1; sleep 0.7
elif [ "${CTRL:-}" = "noop" ]; then
  osascript -e 'tell application "System Events" to key code 22 using {control down, option down, shift down}' >/dev/null 2>&1; sleep 0.7
  osascript -e 'tell application "System Events" to key code 22 using {control down, option down, shift down}' >/dev/null 2>&1; sleep 0.7
elif [ "${CTRL:-}" = "sleep" ]; then sleep 1.4; fi
sleep 1
echo "loadavg_before $(sysctl -n vm.loadavg)"
"$SCRATCH/type-driver.sh" "$DUR" "$OUT.typing.log" >/dev/null 2>&1
echo "loadavg_after $(sysctl -n vm.loadavg)"
require_front $PID 6 >/dev/null || { echo "RIG_FAIL front3 (dump)"; kill -9 $PID; exit 2; }
osascript -e 'tell application "System Events" to key code 28 using {control down, option down, shift down}' >/dev/null 2>&1
for i in $(seq 1 30); do
  V=$(sqlite3 "$APPDATA/global.db" "select value from config_value where kind='app_config' and key='__bench_keys__';" 2>/dev/null)
  [ -n "$V" ] && break; sleep 0.5
done
[ -z "$V" ] && { echo "RIG_FAIL no dump"; kill -9 $PID; exit 2; }
print -r -- "$V" > "$OUT.json"
kill -9 $PID 2>/dev/null; wait $PID 2>/dev/null
echo "OK $OUT.json"
