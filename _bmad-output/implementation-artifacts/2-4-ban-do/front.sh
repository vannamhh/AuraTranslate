#!/bin/zsh
# Cửa tiêu điểm — nguồn dùng chung cho mọi script có gửi phím.
#
# 🔴 VÌ SAO TỆP NÀY TỒN TẠI. Sự cố 2026-08-13 15:35: chạy nhị phân thẳng từ shell trên macOS
# KHÔNG đưa cửa sổ lên trước. App chạy ở nền, còn `osascript ... keystroke` thì gửi vào cửa sổ
# đang ở TRƯỚC — hôm đó là Brave. Năm lượt phím đi nhầm chỗ trước khi một ảnh chụp phát hiện ra.
#
# Bài học, và nó là bài học của kho chứ không của riêng lượt này: bàn đo v2 CÓ kiểm tiêu điểm,
# nhưng chỉ TRONG LÚC GÕ. Bước dựng Tác phẩm chạy TRƯỚC đó ⇒ lọt qua đúng khe. Một hàng rào
# đặt sau chỗ cần chặn là một hàng rào không chặn gì.
#
# ⇒ Luật: KHÔNG script nào được gửi phím đầu tiên trước khi `require_front` xanh.
#    Và `activate_app` phải "đặt rồi ĐỌC LẠI", không tin lượt đặt.
set -u

# ── CỔNG MÀN HÌNH KHOÁ ────────────────────────────────────────────────────────────────
# 🔴 Sự cố 2026-08-19 20:25: lượt đo thật 2 giờ 20 phút chết ở phiên đầu sau 12 giây, và cả
# bốn phiên hỏng y hệt. Triệu chứng đọc lên là *"CỔNG TIÊU ĐIỂM ĐỎ — cần [auratranslate] ở
# trước, đang là [ghostty]"*. Nguyên nhân thật: **màn hình đang KHOÁ**. Khi khoá, app khởi
# động bình thường *(dấu sống `__bench_alive__` vẫn ghi được)* nhưng **không tạo được cửa
# sổ** — đo được `count windows = 0` — nên không tiến trình nào lên trước được.
#
# ⚠️ Vì sao nó nguy hiểm hơn một lỗi thường: nửa Rust chạy ĐÚNG, nên mọi phép kiểm dựa vào
# kho đều xanh. Chỉ nửa GUI chết, và nó chết ở một chỗ báo ra một nguyên nhân KHÁC HẲN —
# lượt chẩn đoán tiếp theo sẽ đi vá cổng tiêu điểm, đúng chỗ không hỏng.
#
# ⇒ Hỏi thẳng, và hỏi TRƯỚC mọi thứ khác.
# 🔴 Khuôn chuỗi không có dấu cách: `CGSSessionScreenIsLocked"=Yes`. Bản dò đầu tiên của tôi
# viết `"CGSSessionScreenIsLocked" = Yes` và nó KHÔNG khớp — tức nó báo "màn hình đang mở"
# trên một máy đang khoá. Một hàng rào báo sai còn tệ hơn không có.
screen_is_locked() {
  ioreg -n Root -d1 -r 2>/dev/null | grep -q 'CGSSessionScreenIsLocked"=Yes'
}

# Cổng cứng: khoá màn hình ⇒ KHÔNG chạy. Trả 1 và nói rõ phải làm gì.
require_unlocked() {
  if screen_is_locked; then
    echo "🔴 MÀN HÌNH ĐANG KHOÁ — bàn đo cần một phiên GUI đang mở." >&2
    echo "   App vẫn khởi động được và vẫn ghi được vào kho, nhưng nó KHÔNG tạo cửa sổ," >&2
    echo "   nên mọi cú bấm và mọi phím đều đi vào hư không." >&2
    echo "   ⇒ Mở khoá máy, rồi chạy lại. Cân nhắc \`caffeinate -disu\` cho lượt đo dài." >&2
    return 1
  fi
  return 0
}

# ── QUY TOẠ ĐỘ KHUNG NHÌN → TOẠ ĐỘ MÀN HÌNH ───────────────────────────────────────────
# 🔴 Sống ở ĐÂY, một chỗ duy nhất, vì hai script cùng cần nó và hai bản chép sẽ lệch nhau
# trong im lặng — đúng cái đã xảy ra với bộ 16 ứng viên toạ độ.
#
# `bench.js` chỉ báo thứ nó biết chắc: tâm phần tử trong KHUNG NHÌN + cỡ khung nhìn
# (`vx,vy,iw,ih`). Nó KHÔNG báo toạ độ màn hình, vì `window.screenX/screenY` của WKWebView
# trả về một hệ khác hệ của `cliclick` khi máy có hai màn hình — đo được 2026-08-19: tab
# Workspace ra `(102,980)` trong khi cửa sổ nằm ở `(200,25)` cỡ `1200×900`.
#
# Phép quy đổi chỉ cần gốc và cỡ cửa sổ, hai thứ `setup-gui.sh` đã ĐẶT RỒI ĐỌC LẠI:
#   x = WIN_X + (WIN_W − iw) + vx        (viền trái, thường 0)
#   y = WIN_Y + (WIN_H − ih) + vy        (thanh tiêu đề)
# Dùng: to_screen "<vx,vy,iw,ih[,...]>" <win_x> <win_y> <win_w> <win_h>  →  in ra "x,y"
to_screen() {
  local v="$1" wx="$2" wy="$3" ww="$4" wh="$5"
  local vx vy iw ih
  vx=$(print -r -- "$v" | cut -d, -f1); vy=$(print -r -- "$v" | cut -d, -f2)
  iw=$(print -r -- "$v" | cut -d, -f3); ih=$(print -r -- "$v" | cut -d, -f4)
  [ -z "$vx" ] || [ -z "$ih" ] && { echo "🔴 to_screen: khuôn sai [$v]" >&2; return 1; }
  print -r -- "$(( wx + (ww - iw) + vx )),$(( wy + (wh - ih) + vy ))"
}

front_name() {
  osascript -e 'tell application "System Events" to name of first application process whose frontmost is true' 2>/dev/null
}

# Đưa tiến trình có đúng PID này lên trước. Dùng `unix id` chứ không dùng tên: tên tiến trình
# của một nhị phân chạy thẳng khác tên bundle, và một lượt khớp tên có thể tóm nhầm instance.
activate_app() {  # $1 = pid
  osascript -e "tell application \"System Events\" to set frontmost of (first application process whose unix id is $1) to true" >/dev/null 2>&1
}

# Cổng CỨNG: không lên trước được thì KHÔNG gửi phím nào, thoát 1.
# Trả tên tiến trình đang ở trước ra stderr để lượt hỏng nói được nó hỏng vì cái gì.
require_front() {  # $1 = pid, $2 = số lượt thử (mặc định 10)
  local pid=$1 tries=${2:-10} want name
  want=$(osascript -e "tell application \"System Events\" to name of (first application process whose unix id is $pid)" 2>/dev/null)
  [ -z "$want" ] && { echo "🔴 không có tiến trình nào mang pid $pid" >&2; return 1; }
  for i in $(seq 1 "$tries"); do
    activate_app "$pid"
    sleep 0.4
    name=$(front_name)
    [ "$name" = "$want" ] && { echo "$want"; return 0; }
  done
  echo "🔴 CỔNG TIÊU ĐIỂM ĐỎ — cần [$want] ở trước, đang là [$name]. KHÔNG gửi phím nào." >&2
  return 1
}
