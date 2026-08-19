#!/bin/zsh
# ══════════════════════════════════════════════════════════════════════════════════════
# PHIÊN ĐO NFR2 THẬT — Task 3 của Story 2.4, 2026-08-19
# ══════════════════════════════════════════════════════════════════════════════════════
# Task 3 đòi: *"Phiên gõ liên tục ≥ 30 phút trên thang nhân tạo … n = 3 phiên (AC19)"*.
# Tệp này chạy đúng ba phiên đó, TUẦN TỰ, cộng một lượt đối chứng AC21 CÙNG THỜI LƯỢNG.
#
#   cd _bmad-output/implementation-artifacts/2-4-ban-do && ./run-nfr2-real.sh
#
# ── 🔴 VÌ SAO TUẦN TỰ, KHÔNG SONG SONG ────────────────────────────────────────────────
# Hai phiên chạy chồng là hai phiên đo lẫn nhau. Và bàn đo ĐỘC CHIẾM bàn phím + chuột —
# hai bộ lái gõ cùng lúc sẽ bơm phím vào chung một cửa sổ.
#
# ── 🔴 VÌ SAO n=3 LÀ BA TỆP RIÊNG, KHÔNG MỘT TỆP GỘP ──────────────────────────────────
# AC2 và AC19 đều CẤM gộp mẫu: NFR2 là mệnh đề về ĐUÔI phân bố, và gộp mẫu là đúng cơ chế
# nuốt mất một phiên xấu. Bốn phiên smoke ngày 2026-08-19 cho hai cụm tách bạch (~9 % và
# 39 % frame vượt trần, cùng tham số) — gộp chúng lại sẽ cho một con số ~24 % không mô tả
# một phiên nào có thật.
#
# ── ⚠️ LƯỢT ĐỐI CHỨNG AC21 CHẠY CUỐI, CÓ CHỦ Ý ────────────────────────────────────────
# Ba phiên bắt buộc xong TRƯỚC. Nếu lượt chạy bị cắt giữa chừng thì thứ mất là lượt đối
# chứng, không phải một phiên nghiệm thu.
# ⚠️ Lượt đối chứng ngày 2026-08-18 chỉ dài 44 giây; AC21 đòi CÙNG thời lượng. Đây là lần
# đầu điều kiện đó được thoả.
set -u

SCRATCH="${0:A:h}"
LAD="${LAD:-l}"          # thang nhân tạo; `l` = 48.639 ký tự / 353 segment
DUR="${DUR:-1800}"       # 30 phút — sàn của AC1
LOG="$SCRATCH/real-run-$(date '+%Y%m%d-%H%M').log"

say() { print -r -- "$@" | tee -a "$LOG" }

say "══════════════════════════════════════════════════════════════"
say "PHIÊN ĐO NFR2 THẬT — thang=$LAD · ${DUR}s mỗi phiên · n=3 + 1 đối chứng"
say "bắt đầu $(date '+%Y-%m-%d %H:%M:%S')"
say "🔴 MÁY BỊ ĐỘC CHIẾM (bàn phím + chuột) tới khi xong. Đừng chạm."
say "══════════════════════════════════════════════════════════════"

# ── CỔNG MÀN HÌNH KHOÁ, hỏi TRƯỚC MỌI THỨ ─────────────────────────────────────────────
# 🔴 Đây là hàng rào đắt nhất của lượt sửa 2026-08-19: lượt chạy đầu tiên của tệp này chết
# vì màn hình khoá, và cả bốn phiên hỏng trong 12–19 giây với một thông báo trỏ sai chỗ.
# Hỏi ở đây thì lượt chạy từ chối khởi hành trong MỘT giây, thay vì đốt hai giờ chờ.
say ""
if ! require_unlocked 2>&1 | tee -a "$LOG"; then
  say "🔴 KHÔNG khởi hành. Mở khoá máy rồi chạy lại."
  exit 1
fi
say "🟢 màn hình đang mở"

# ⚠️ Và giữ nó mở: lượt chạy này dài ~2 giờ 20 phút, dài hơn mọi thiết đặt ngủ mặc định.
# `caffeinate` chạy nền và chết cùng tệp này — không để lại một thiết đặt hệ thống nào.
caffeinate -disu -w $$ &
say "🟢 caffeinate đang giữ máy thức (theo pid $$)"

# ── CỔNG TIỀN KIỂM: máy phải RẢNH trước phiên đầu ─────────────────────────────────────
# 🔴 Không phải thủ tục. Bốn phiên smoke cùng tham số cho hai cụm ~9 % và 39 % frame vượt
# trần, và biến chưa kiểm soát khả dĩ nhất là tải nền. Khởi hành trên một máy còn bận là
# đốt phiên đầu — mà một phiên là 30 phút.
# ⚠️ Ngưỡng đặt ở đây, KHÔNG đọc từ thứ đang đo (luật của kho: không phán quyết nào được
# đọc tham số từ chính thứ nó đang kiểm).
LOAD_MAX="${LOAD_MAX:-3.0}"
WAIT_MAX="${WAIT_MAX:-1800}"
say ""
say "── cổng tiền kiểm: chờ loadavg-1phút < $LOAD_MAX (tối đa $((WAIT_MAX/60)) phút) ──"
DEADLINE=$(( $(date +%s) + WAIT_MAX ))
while :; do
  L1=$(sysctl -n vm.loadavg | awk '{print $2}')
  if [ "$(perl -e "print(($L1 < $LOAD_MAX) ? 1 : 0)")" -eq 1 ]; then
    say "🟢 máy rảnh — loadavg = $(sysctl -n vm.loadavg)"
    break
  fi
  if [ "$(date +%s)" -ge "$DEADLINE" ]; then
    say "🔴 HẾT TRẦN CHỜ — loadavg-1phút vẫn $L1. KHÔNG chạy: số đo sẽ mang một biến chưa kiểm soát."
    exit 1
  fi
  sleep 30
done

# 🔴 Không instance app nào được chạy — một cái còn sống có thể đang mở DỮ LIỆU THẬT, và nó
# cũng ăn CPU của phép đo. Cùng hàng rào mà `run-grid.sh` đã có; bản đo NFR2 thì chưa.
if pgrep -x auratranslate >/dev/null; then
  say "🔴 còn một instance auratranslate đang chạy. Đóng hết rồi chạy lại."
  exit 1
fi
say "🟢 không instance app nào đang chạy"

FAILED=0
run_one() {   # $1 = nhãn, $2 = chế độ (typing|control)
  local label="$1" mode="${2:-typing}"
  say ""
  say "───── $label ($mode) · bắt đầu $(date '+%H:%M:%S') ─────"
  if "$SCRATCH/nfr2-session.sh" "$LAD" "$DUR" "$label" "$mode" >> "$LOG" 2>&1; then
    local n=$(wc -c < "$SCRATCH/nfr2-$label.json" 2>/dev/null | tr -d ' ')
    say "🟢 $label XONG $(date '+%H:%M:%S') — nfr2-$label.json ($n byte)"
  else
    # 🔴 Một phiên hỏng KHÔNG dừng cả lượt chạy: ba phiên độc lập nhau, và một phiên mất
    # vẫn để lại hai phiên dùng được. Nhưng nó phải được ĐẾM và in ra ở cuối — một lượt
    # chạy "xong" với hai phiên là một lượt chạy CHƯA thoả n=3.
    say "🔴 $label HỎNG $(date '+%H:%M:%S') — xem $LOG"
    FAILED=$((FAILED+1))
  fi
}

run_one real1 typing
run_one real2 typing
run_one real3 typing
run_one ac21ctl control     # đối chứng AC21 — cùng bàn đo, cùng thời lượng, KHÔNG gõ

say ""
say "══════════════════════════════════════════════════════════════"
say "XONG $(date '+%Y-%m-%d %H:%M:%S') · số lượt hỏng: $FAILED"
if [ "$FAILED" -gt 0 ]; then
  say "🔴 CHƯA thoả n=3 — $FAILED lượt hỏng. Đừng đọc số như một bộ đủ."
fi
say "tệp kết quả:"
for f in real1 real2 real3 ac21ctl; do
  [ -f "$SCRATCH/nfr2-$f.json" ] \
    && say "   nfr2-$f.json  ($(wc -c < "$SCRATCH/nfr2-$f.json" | tr -d ' ') byte)" \
    || say "   nfr2-$f.json  ⟨KHÔNG CÓ⟩"
done
say "trạng thái máy từng phiên: env-real1.txt · env-real2.txt · env-real3.txt · env-ac21ctl.txt"
say "══════════════════════════════════════════════════════════════"
