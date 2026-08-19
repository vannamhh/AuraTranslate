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

# 🔵 HIỆU CHUẨN LẠI 2026-08-18 — bề mặt đã đổi, và bộ ứng viên cũ trỏ RA NGOÀI lưới.
#
# Bộ cũ (`640,165` …) hiệu chuẩn ngày 2026-08-13 trên `EditorPanel.vue` — một dòng văn liên
# tục chiếm gần trọn cửa sổ. Lượt correct-course 2026-08-14 thay nó bằng `GridPanel.vue`:
# một LƯỚI HAI CỘT chiếm NỬA TRÁI cửa sổ, nửa phải là panel Tra cứu.
# ⇒ `x = +640` nay rơi vào panel **Tra cứu**. Bàn đo cũ sẽ trượt cả 16 ứng viên và lượt
# chẩn đoán tiếp theo sẽ đi tìm một khuyết tật con trỏ KHÔNG TỒN TẠI.
#
# Số mới đo bằng HAI đường độc lập, và chúng khớp nhau — đó là lý do tin được:
#   ① Ảnh chụp `calib-3-workspace.png` (cửa sổ chuẩn hoá {200,25,1200,900}, Retina ×2):
#      cột `[data-col="tgt"]` trải x = +270 … +474 so gốc cửa sổ, tâm ≈ +372.
#   ② Suy từ CSS `grid-template-columns: 3px 30px 1fr 1fr 96px` (`GridPanel.vue:1645`) trên
#      panel Đối chiếu rộng ~589 điểm ⇒ tâm cột đích ≈ +365.
#
# ⚠️ Trục y thì GIỮ NGUYÊN được: hàng đầu vẫn nằm quanh +170, và số cũ +165 vốn đã đúng.
# Vẫn quét nhiều hàng, vì chiều cao hàng đổi theo độ dài câu nguồn.
CAND=(
  "372,170" "340,170" "404,170" "300,170"
  "372,230" "340,230" "404,230" "300,230"
  "372,350" "340,350" "372,480" "340,480"
  "440,170" "440,230" "280,170" "466,170"
)

probe_count() { sqlite3 -readonly "$DB" "select count(*) from segment where target_text<>'';" 2>/dev/null || echo 0 }

BEFORE=$(probe_count)

# ── 🔵 2026-08-19 — HỎI TOẠ ĐỘ THẬT TRƯỚC, ĐOÁN SAU ─────────────────────────────────────
# Đo được 2026-08-19: bộ 16 ứng viên viết cứng chỉ vào được **3 trên 7 lượt**, cùng script,
# cùng thang, cùng hình học cửa sổ. Cách chữa cũ là thêm ứng viên — tức vẫn đoán, chỉ đoán
# nhiều hơn. Nay `bench.js` sống trong bản dựng và hỏi được cây DOM thật, nên nó trả về tâm
# một ô đang hiện theo TOẠ ĐỘ MÀN HÌNH (`bench.js` §⑦b, phím Ctrl+Alt+Shift+7).
#
# 🔴 Đây KHÔNG thay cú bấm bằng một sự kiện tổng hợp: vẫn `cliclick`, vẫn `mousedown` thật.
# Thứ bị thay là phép ĐOÁN toạ độ, không phải phép BẤM.
#
# ⚠️ Chỉ có trên bản dựng CÓ tiêm bench. `kill-campaign-v2.sh` chạy bản release trần, nên
# bộ 16 ứng viên ở lại làm ĐƯỜNG LÙI — đừng gỡ nó.
APPDB="${BENCH_APPDATA:-}"
MEASURED=""
if [ -n "$APPDB" ] && [ -f "$APPDB/global.db" ]; then
  require_front "$PID" 3 >/dev/null || { echo "🔴 mất tiêu điểm trước lượt hỏi toạ độ" >&2; exit 1; }
  osascript -e "tell application \"System Events\" to key code 26 using {control down, option down, shift down}" >/dev/null 2>&1
  for _ in 1 2 3 4 5 6 7 8; do
    sleep 0.5
    V=$(sqlite3 -readonly "$APPDB/global.db" "select value from config_value where key='__cellrect__';" 2>/dev/null)
    [ -n "$V" ] && break
  done
  if [ -n "${V:-}" ] && [ "$V" != "NONE" ]; then
    # Khuôn `vx,vy,iw,ih,số-ô` — quy về màn hình bằng hàm dùng chung ở `front.sh`, không tự
    # chép lại phép tính. Kích thước cửa sổ là {1200,900} do `setup-gui.sh` đặt-rồi-đọc-lại.
    MEASURED=$(to_screen "$V" $WIN_X $WIN_Y 1200 900) || MEASURED=""
    [ -n "$MEASURED" ] && \
      echo "   toạ độ ĐO ĐƯỢC: khung nhìn [$V] ⇒ màn hình $MEASURED  (tổng $(print -r -- "$V" | cut -d, -f5) ô)" >&2
  else
    echo "   ⚠️ bench không trả toạ độ (\"${V:-rỗng}\") — rơi về bộ 16 ứng viên" >&2
  fi
fi

# 🔴 MỘT HỆ TOẠ ĐỘ DUY NHẤT, và đây là chỗ dễ sai nhất của lượt sửa này: bộ 16 ứng viên cũ là
# toạ độ TƯƠNG ĐỐI gốc cửa sổ, còn số bench trả về là toạ độ MÀN HÌNH tuyệt đối. Trộn hai hệ
# trong một vòng lặp thì cú bấm rơi ra ngoài màn hình và lượt chẩn đoán tiếp theo sẽ đi tìm
# một khuyết tật con trỏ KHÔNG TỒN TẠI. ⇒ Quy hết về TUYỆT ĐỐI ngay tại đây, một lần.
ABS=()
[ -n "$MEASURED" ] && ABS+=("$MEASURED")          # đã tuyệt đối
for c in "${CAND[@]}"; do                          # tương đối ⇒ cộng gốc cửa sổ
  ABS+=("$((WIN_X + ${c%%,*})),$((WIN_Y + ${c##*,}))")
done

# 🔴 Vẫn giữ nguyên phép nghiệm thu "gõ ký tự dò → HỎI KHO": một toạ độ ĐO ĐƯỢC vẫn có thể
# trượt (ô bị một lớp khác che, hoặc lưới cuộn giữa lúc đo và lúc bấm), và một cửa vào không
# nghiệm thu là đúng thứ tệp này tồn tại để chống.
for c in "${ABS[@]}"; do
  DX="${c%%,*}"; DY="${c##*,}"
  require_front "$PID" 3 >/dev/null || { echo "🔴 mất tiêu điểm giữa lượt dò" >&2; exit 1; }
  cliclick c:${DX},${DY}      # ĐÃ tuyệt đối — xem khối quy đổi ngay trên
  sleep 0.4
  osascript -e 'tell application "System Events" to keystroke "x"' >/dev/null 2>&1
  # 🔵 Sửa 2026-08-18: 2,4 s → 4,5 s. Số 2,4 là con số ĐÃ ĐƯỢC ĐO LÀ HỎNG — `README.md`
  # §Hằng số ghi *"nhịp nghiệm thu con trỏ ≥ 4,5 s · 2,4 s cho ÂM TÍNH GIẢ ở cả 16 ứng viên"*,
  # nhưng lượt lưu bàn đo vào kho để sót số cũ trong mã. ⇒ Bản đã commit sẽ báo *"không điểm
  # nào đặt được con trỏ"* trên MỌI ứng viên, và lượt chẩn đoán tiếp theo sẽ đi vá toạ độ
  # trong khi chỗ hỏng nằm ở NHỊP. Đúng lớp lỗi §Hai lỗi của chính bàn đo đã ghi tên ba lần.
  sleep 4.5          # 2 s `EDITOR_IDLE_MS` + đường ghi + biên
  AFTER=$(probe_count)
  if [ "${AFTER:-0}" -gt "${BEFORE:-0}" ]; then
    # 🟢 trúng. Xoá ký tự dò để nó không lẫn vào số đo.
    osascript -e 'tell application "System Events" to key code 51' >/dev/null 2>&1
    sleep 4.5          # cùng lý do: lượt XOÁ cũng phải hạ cánh vào kho trước khi đo
    echo "$DX,$DY"
    exit 0
  fi
done

echo "🔴 KHÔNG điểm nào trong ${#CAND[@]} ứng viên đặt được con trỏ vào một câu" >&2
exit 1
