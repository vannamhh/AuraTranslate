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
