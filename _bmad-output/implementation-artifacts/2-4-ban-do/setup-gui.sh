#!/bin/zsh
# Dựng Tác phẩm bằng ĐƯỜNG GIAO DIỆN THẬT — 0 phụ thuộc bench.js.
#
# ═══════════════════════════════════════════════════════════════════════════════════
# VÌ SAO TỆP NÀY TỒN TẠI, VÀ VÌ SAO NÓ TRỘN HAI CƠ CHẾ
# ═══════════════════════════════════════════════════════════════════════════════════
# ① `kill-campaign.sh` v1 dựng Tác phẩm bằng `key 18` (Ctrl+Alt+Shift+1). Phím đó KHÔNG phải
#    của sản phẩm — `bench.js:242-245` đăng ký nó. Tức bộ kill phụ thuộc thẳng vào bàn đo tiêm,
#    đúng cái đang bị chặn. v1 chưa từng chạy nên chưa ai thấy chỗ phụ thuộc này.
#
# ② Máy đo có `AppleKeyboardUIMode = 0` — mặc định của macOS. Trong chế độ đó, Tab trong nội
#    dung web đi qua Ô VĂN BẢN nhưng BỎ QUA NÚT BẤM. Đo được, không suy:
#      · Tab×5 ⇒ ô "Hoặc nhập đường dẫn tệp" nhận tiêu điểm (vòng xanh + chữ hạ cánh) — `p3.png`
#      · `click at` ⇒ mở được `<select>` "Ngôn ngữ nguồn" — `probe.png`
#      · `click at` vào một `<input>` ⇒ KHÔNG đặt được tiêu điểm — `p2.png`, ô vẫn rỗng
#    ⇒ Dùng Tab cho ô, dùng click cho nút. Mỗi cơ chế đúng chỗ nó chạy được.
#
#    ⚠️ Đây là một phát hiện đáng vào báo cáo NFR17: `commands/index.ts:526-530` khai hai nút
#    import "tới được bằng bàn phím qua Tab + Enter/Space". Đúng theo chuẩn HTML, nhưng trên một
#    macOS cấu hình MẶC ĐỊNH thì KHÔNG — người dùng chỉ-bàn-phím không Tab tới được hai nút đó.
#
# ③ Toạ độ tính theo GỐC CỬA SỔ, và gốc đó phải ĐỌC LẠI: một lượt `set position to {0,25}` đã
#    trả về `{206,25}` — macOS ràng buộc lại. Tin lượt đặt là lệch 206 điểm.
#    Cửa sổ đặt ở x=200 để Dock (chiếm ~54 điểm bên trái) không đè lên nút.
#
# Thứ tự Tab của form (`LibraryMode.vue:77-134`), form rỗng:
#   ① Tên ② Ngôn ngữ ③ Thể loại ④ Dán văn bản ⑤[nút disabled — BỎ QUA] ⑥ Đường dẫn tệp
#   ⇒ đúng 5 chặng Tab tới ô đường dẫn.
#
# Dùng: setup-gui.sh <pid> <đường-dẫn-tệp-nguồn>
set -u
PID="${1:?thiếu pid}"
SRC="${2:?thiếu đường dẫn tệp nguồn}"

SCRATCH="${0:A:h}"
source "$SCRATCH/front.sh"

WIN_X=200; WIN_Y=25; WIN_W=1200; WIN_H=900
BTN_DX=85; BTN_DY=685      # nút "Tạo Tác phẩm từ tệp", tương đối gốc cửa sổ
TABS=5                     # số chặng Tab tới ô đường dẫn

require_front "$PID" 10 >/dev/null || exit 1

# ── Chờ cửa sổ VẼ XONG — mốc dừng thật, không sleep mù ────────────────────────────
for i in $(seq 1 40); do
  N=$(osascript -e "tell application \"System Events\" to tell (first application process whose unix id is $PID) to count windows" 2>/dev/null)
  [ "${N:-0}" -ge 1 ] 2>/dev/null && break
  sleep 0.25
done
[ "${N:-0}" -ge 1 ] 2>/dev/null || { echo "🔴 cửa sổ không xuất hiện sau 10 s" >&2; exit 1; }

# ── Chuẩn hoá hình học: size TRƯỚC, position SAU (đổi cỡ có thể dời cửa sổ) ────────
osascript -e "tell application \"System Events\" to tell (first application process whose unix id is $PID) to tell window 1 to set size to {$WIN_W, $WIN_H}" >/dev/null 2>&1
sleep 0.4
osascript -e "tell application \"System Events\" to tell (first application process whose unix id is $PID) to tell window 1 to set position to {$WIN_X, $WIN_Y}" >/dev/null 2>&1
sleep 0.7

GEOM=$(osascript -e "tell application \"System Events\" to tell (first application process whose unix id is $PID) to get {position, size} of window 1" 2>/dev/null)
WX=$(echo "$GEOM" | cut -d, -f1 | tr -d ' ')
WY=$(echo "$GEOM" | cut -d, -f2 | tr -d ' ')
[ -z "$WX" ] && { echo "🔴 không đọc được hình học cửa sổ" >&2; exit 1; }

require_front "$PID" 6 >/dev/null || exit 1

# ── Tab tới ô đường dẫn, gõ, rồi CLICK nút ────────────────────────────────────────
for i in $(seq 1 $TABS); do
  osascript -e 'tell application "System Events" to key code 48' >/dev/null 2>&1
  sleep 0.2
done
osascript -e "tell application \"System Events\" to keystroke \"$SRC\"" >/dev/null 2>&1
sleep 0.6

require_front "$PID" 3 >/dev/null || exit 1   # cổng ngay trước cú bấm quyết định
osascript -e "tell application \"System Events\" to click at {$((WX + BTN_DX)), $((WY + BTN_DY))}" >/dev/null 2>&1
