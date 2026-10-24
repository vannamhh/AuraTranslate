#!/bin/zsh
# Lượt HIỆU CHUẨN 2026-08-18 — Ice ký: chạy một lượt trước khi đốt ≥120 lượt kill.
#
# 🔴 VÌ SAO TỆP NÀY TỒN TẠI. Mọi hằng GUI ở `README.md` §Hằng số được hiệu chuẩn ngày
# 2026-08-13, trên bề mặt Editor CŨ (`EditorPanel.vue` — một dòng văn liên tục). Lượt
# correct-course 2026-08-14 thay bề mặt đó bằng một LƯỚI HAI CỘT (`GridPanel.vue`).
# ⇒ Toạ độ ô gõ chắc chắn đổi; toạ độ nút và tab thì CHƯA BIẾT.
#
# Đường đi rẻ nhất là một ẢNH CHỤP, không một lượt quét mù: bài học 2026-08-13 nguyên văn —
# *"cái tách hai thứ đó ra là một ảnh chụp, không phải một lượt suy luận thêm"*.
#
# Tệp này KHÔNG gõ vào vùng gõ và KHÔNG kill lượt nào. Nó chỉ dựng tới màn Workspace rồi chụp.
set -u
SCRATCH="${0:A:h}"
APP="/Users/hoangnam/LocalSites/addon/AuraTranslate/src-tauri/target/release/bundle/macos/AuraTranslate.app/Contents/MacOS/auratranslate"
DRAFT_HOME="$SCRATCH/draft-home"
APPDATA="$DRAFT_HOME/Library/Application Support/com.auratranslate.desktop"
LIB="$DRAFT_HOME/Documents/AuraTranslate"
SRC_FILE="$SCRATCH/chapters/ladder-m.txt"
WIN_X=200; WIN_Y=25
source "$SCRATCH/front.sh"

rm -rf "$LIB" "$APPDATA"; mkdir -p "$DRAFT_HOME/Documents"

HOME="$DRAFT_HOME" "$APP" > "$SCRATCH/calib-app.log" 2>&1 &
PID=$!
echo "pid=$PID"

for i in $(seq 1 60); do [ -f "$APPDATA/global.db" ] && break; sleep 0.5; done
[ -f "$APPDATA/global.db" ] || { echo "🔴 global.db khong hien ra trong HOME nhap"; kill -9 $PID; exit 1; }
sleep 2

# ── ① chụp màn Library TRƯỚC khi bấm gì — để đọc lại toạ độ nút nếu setup-gui đỏ ──
require_front "$PID" 10 >/dev/null || { kill -9 $PID; exit 1; }
osascript -e "tell application \"System Events\" to tell (first application process whose unix id is $PID) to tell window 1 to set size to {1200, 900}" >/dev/null 2>&1
sleep 0.4
osascript -e "tell application \"System Events\" to tell (first application process whose unix id is $PID) to tell window 1 to set position to {$WIN_X, $WIN_Y}" >/dev/null 2>&1
sleep 0.8
screencapture -x "$SCRATCH/calib-1-library.png"
echo "① anh man Library"

# ── ② dựng Tác phẩm qua ĐƯỜNG GIAO DIỆN THẬT ──────────────────────────────────────
if ! "$SCRATCH/setup-gui.sh" $PID "$SRC_FILE"; then
  echo "🔴 setup-gui DO — cac hang Library da het dung"
  screencapture -x "$SCRATCH/calib-2-setup-fail.png"
  kill -9 $PID; exit 1
fi
for i in $(seq 1 80); do [ -n "$(find "$LIB" -name project.db 2>/dev/null | head -1)" ] && break; sleep 0.5; done
DB=$(find "$LIB" -name project.db 2>/dev/null | head -1)
if [ -z "$DB" ]; then
  echo "🔴 KHONG dung duoc Tac pham — Tab x5 hoac toa do nut da het dung"
  screencapture -x "$SCRATCH/calib-2-nowork.png"
  kill -9 $PID; exit 1
fi
echo "② dung duoc Tac pham: $DB"
sqlite3 -readonly "$DB" "select count(*) from segment;" | sed 's/^/   segment=/'
sleep 4

# ── ③ mở Workspace rồi CHỤP — đây là ảnh quyết định toạ độ ô gõ ───────────────────
require_front "$PID" 6 >/dev/null || { kill -9 $PID; exit 1; }
cliclick c:$((WIN_X + 101)),$((WIN_Y + 46))
sleep 5
screencapture -x "$SCRATCH/calib-3-workspace.png"
echo "③ anh man Workspace"

kill -9 $PID 2>/dev/null
echo "xong — app da tat"
