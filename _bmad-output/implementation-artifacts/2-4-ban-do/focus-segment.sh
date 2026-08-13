#!/bin/zsh
# Đặt con trỏ vào MỘT câu trong panel Bản dịch — và NGHIỆM THU rằng nó đã vào.
#
# 🔴 VÌ SAO KHÔNG DÙNG MỘT TOẠ ĐỘ CỐ ĐỊNH. Lượt quét lưới 2026-08-13 tìm được (840,190) ăn.
# Lượt sau, cùng toạ độ, cùng hình học cửa sổ, cùng tệp nguồn ⇒ KHÔNG ăn. Vùng trúng không ổn
# định giữa các lượt. Ice mô tả đúng hiện tượng đó khi gõ tay: *"click không chính xác thì không
# hiển thị input và không gõ được"*.
#
# ⇒ Không tin một toạ độ. Bấm → gõ MỘT ký tự dò → HỎI KHO. Kho trả lời có thì mới đi tiếp.
# Đây là "đặt rồi đọc lại" áp cho con trỏ, đúng luật của kho.
#
# ⚠️ Ký tự dò được XOÁ sau khi nghiệm thu, để nó không lẫn vào số đo.
#
# Dùng: focus-segment.sh <pid> <duong-dan-project.db>
# Trả 0 kèm in ra toạ độ đã ăn; trả 1 nếu không điểm nào ăn.
set -u
PID="${1:?thiếu pid}"
DB="${2:?thiếu project.db}"

SCRATCH="${0:A:h}"
source "$SCRATCH/front.sh"

WIN_X=200; WIN_Y=25

# Ứng viên: quét ngang qua đầu dòng, nhiều dòng. Thứ tự theo xác suất trúng đã đo
# (hàng y=190 và y=220 từng ăn ở lượt quét lưới).
CAND=(
  "640,165" "660,165" "620,165" "680,165"
  "640,195" "660,195" "620,195" "680,195"
  "640,150" "660,150" "640,210" "660,210"
  "700,165" "700,195" "600,165" "600,195"
)

probe_count() { sqlite3 -readonly "$DB" "select count(*) from segment where target_text<>'';" 2>/dev/null || echo 0 }

BEFORE=$(probe_count)

for c in "${CAND[@]}"; do
  DX="${c%%,*}"; DY="${c##*,}"
  require_front "$PID" 3 >/dev/null || { echo "🔴 mất tiêu điểm giữa lượt dò" >&2; exit 1; }
  cliclick c:$((WIN_X + DX)),$((WIN_Y + DY))
  sleep 0.4
  osascript -e 'tell application "System Events" to keystroke "x"' >/dev/null 2>&1
  sleep 2.4          # nhịp flush 2 s của AD-35 + biên
  AFTER=$(probe_count)
  if [ "${AFTER:-0}" -gt "${BEFORE:-0}" ]; then
    # 🟢 trúng. Xoá ký tự dò để nó không lẫn vào số đo.
    osascript -e 'tell application "System Events" to key code 51' >/dev/null 2>&1
    sleep 2.4
    echo "$DX,$DY"
    exit 0
  fi
done

echo "🔴 KHÔNG điểm nào trong ${#CAND[@]} ứng viên đặt được con trỏ vào một câu" >&2
exit 1
