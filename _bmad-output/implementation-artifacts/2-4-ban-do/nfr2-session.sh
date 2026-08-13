#!/bin/zsh
# Phiên đo NFR2 của Story 2.4 — AC1 · AC11 · AC12 · AC13 · AC21 · AC22.
#
# Dùng: nfr2-session.sh <thang: xs|s|m|l> <giây-gõ> <nhãn> [control]
#   control ⇒ lượt ĐỐI CHỨNG của AC21: cùng bàn đo, cùng thời lượng, KHÔNG gõ.
set -u
LAD="${1:-m}"; DUR="${2:-1800}"; LABEL="${3:-run}"; MODE="${4:-typing}"
SCRATCH="${0:A:h}"
APP="/Users/hoangnam/LocalSites/addon/AuraTranslate/src-tauri/target/release/bundle/macos/AuraTranslate.app/Contents/MacOS/auratranslate"
DRAFT="$SCRATCH/draft-home"
APPDATA="$DRAFT/Library/Application Support/com.auratranslate.desktop"
CLICK_X="${BENCH_CLICK_X:-980}"; CLICK_Y="${BENCH_CLICK_Y:-430}"

key() { osascript -e "tell application \"System Events\" to key code $1 using {control down, option down, shift down}" >/dev/null 2>&1 }
# mã phím: 1→18  2→19  3→20  4→21  5→23  6→22
now() { perl -MTime::HiRes=time -e 'printf "%.3f\n", time' }

cp "$SCRATCH/chapters/ladder-$LAD.txt" /tmp/at-bench-chapter.txt
rm -rf "$DRAFT"; mkdir -p "$DRAFT/Documents"

echo "== khởi động (thang=$LAD · $(wc -m < /tmp/at-bench-chapter.txt) ký tự · chế độ=$MODE) =="
HOME="$DRAFT" "$APP" > "$SCRATCH/app-$LABEL.log" 2>&1 &
PID=$!
for i in $(seq 1 60); do [ -f "$APPDATA/global.db" ] && break; sleep 0.5; done
sleep 3

echo "== dựng Tác phẩm + tách segment =="
key 18
for i in $(seq 1 120); do [ -n "$(find "$DRAFT/Documents/AuraTranslate" -name project.db 2>/dev/null | head -1)" ] && break; sleep 0.5; done
DB=$(find "$DRAFT/Documents/AuraTranslate" -name project.db 2>/dev/null | head -1)
[ -z "$DB" ] && { echo "🔴 không dựng được Tác phẩm"; kill -9 $PID; exit 1; }
sleep 6   # chờ nạp lại + dựng segment
echo "   segment: $(sqlite3 "$DB" 'select count(*) from segment;' 2>/dev/null)"

echo "== AC13: đo lượt DỰNG lại cả Chương =="
key 23; sleep 2
echo "== AC12: đo ba ĐƯỜNG NÓNG =="
key 21; sleep 6

echo "== bắt đầu lấy mẫu frame =="
key 19; sleep 1

if [ "$MODE" = "typing" ]; then
  osascript -e "tell application \"System Events\" to click at {$CLICK_X, $CLICK_Y}" >/dev/null 2>&1
  sleep 1
  echo "== gõ ${DUR}s =="
  "$SCRATCH/type-driver.sh" "$DUR" "$SCRATCH/typing-$LABEL.log" >/dev/null 2>&1
else
  echo "== ĐỐI CHỨNG: bàn đo chạy ${DUR}s, KHÔNG gõ (AC21) =="
  perl -e "select(undef,undef,undef,$DUR)"
fi

echo "== ngừng + đổ số =="
key 20; sleep 3

sqlite3 "$APPDATA/global.db" "select value from config_value where key='__bench__';" > "$SCRATCH/nfr2-$LABEL.json" 2>/dev/null
echo "   → $SCRATCH/nfr2-$LABEL.json  ($(wc -c < "$SCRATCH/nfr2-$LABEL.json") byte)"
printf 'wal_project=%s wal_global=%s\n' \
  "$(stat -f '%z' "$DB-wal" 2>/dev/null || echo 0)" \
  "$(stat -f '%z' "$APPDATA/global.db-wal" 2>/dev/null || echo 0)"

kill -9 $PID 2>/dev/null
echo "═══ xong ═══"
