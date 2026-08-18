#!/bin/zsh
# Gom sáu tệp kết quả thành BẢNG CỦA AC8 — bảng này LÀ câu trả lời cho AC3.
#
# 🔴 AC2 cấm gộp mẫu: NFR18 là mệnh đề về MAX, và thống kê max là loại tệ nhất khi bị gộp —
# một điểm lưới xấu bị nuốt trọn. Nên bảng in **max của TỪNG điểm riêng**, không một con số
# chung, và in kèm cỡ mẫu ngay trong hàng.
#
# 🔴 Dung sai AC2: `max − trung vị > 2 s` HOẶC có bất kỳ mẫu nào > 5 s ⇒ gắn cờ BẤT ỔN, và
# lượt đo đó KHÔNG được khai là "nhất quán" kể cả khi mọi mẫu đều dưới ngưỡng.
set -u
SCRATCH="${0:A:h}"

printf '%-8s %5s %6s %8s %9s %9s %9s %7s %10s %7s %s\n' \
  'nguong' 'ban' 'hople' 'min_s' 'trung_vi' 'max_s' 'max-tv' 'vuot5s' 'dinh_wal_p' 'busy' 'co'
printf '%s\n' '────────────────────────────────────────────────────────────────────────────────────────────────────'

for LABEL in 512kib 1mib 2mib 4mib 8mib 16mib; do
  F="$SCRATCH/kill2-g$LABEL.tsv"
  [ -f "$F" ] || { printf '%-8s %s\n' "$LABEL" '— chưa chạy —'; continue }

  SHOTS=$(awk -F'\t' 'NR>1 && $1!=""' "$F" | wc -l | tr -d ' ')
  # chỉ VALID và VALID_IDLE vào mẫu (AC9); RIG_FAIL/MISS/AMBIG/BLUR_FAIL thì không
  WINS=$(awk -F'\t' 'NR>1 && ($9=="VALID" || $9=="VALID_IDLE") && $6!="" {print $6}' "$F" | sort -n)
  N=$(print -r -- "$WINS" | grep -c . )
  [ "${N:-0}" -eq 0 ] && { printf '%-8s %5s %6s %s\n' "$LABEL" "$SHOTS" 0 '— 0 mẫu hợp lệ —'; continue }

  MIN=$(print -r -- "$WINS" | head -1)
  MAX=$(print -r -- "$WINS" | tail -1)
  MED=$(print -r -- "$WINS" | awk -v n="$N" 'NR==int((n+1)/2){print}')
  SPREAD=$(perl -e "printf '%.3f', $MAX-$MED")
  OVER=$(print -r -- "$WINS" | awk '$1>5.0' | grep -c .)
  WALP=$(awk -F'\t' 'NR>1 && $3!="" && $3+0>m {m=$3+0} END{print m+0}' "$SCRATCH/wal2-g$LABEL.tsv" 2>/dev/null)
  BUSY=$(awk -F'\t' 'NR>1 {b+=$10+$11} END{print b+0}' "$F")

  FLAG=''
  [ "$OVER" -gt 0 ] && FLAG='🔴 BẤT ỔN: có mẫu > 5 s'
  [ "$(perl -e "print(($SPREAD>2.0)?1:0)")" -eq 1 ] && FLAG="${FLAG}${FLAG:+ · }🔴 BẤT ỔN: max−trung vị > 2 s"
  [ "$N" -lt 20 ] && FLAG="${FLAG}${FLAG:+ · }⚠️ n=$N DƯỚI sàn 20 của AC2"
  [ -z "$FLAG" ] && FLAG='🟢'

  printf '%-8s %5s %6s %8s %9s %9s %9s %7s %10s %7s %s\n' \
    "$LABEL" "$SHOTS" "$N" "$MIN" "$MED" "$MAX" "$SPREAD" "$OVER" "${WALP:-—}" "$BUSY" "$FLAG"
done

cat <<'NOTE'

── ĐỌC BẢNG NÀY THẾ NÀO (AC8 → AC3) ──────────────────────────────────────────────────
Lưới PHẲNG  ⇒ NFR18 KHÔNG treo trên `wal_threshold_bytes`. Cặp đánh đổi gốc của hàng
             Deferred `ARCHITECTURE-SPINE.md:894` (WAL ⟷ NFR18) HẸP HƠN câu hỏi đã đặt.
             Dòng đóng `:894` PHẢI khai sự thu hẹp đó bằng một câu kèm số — thiếu nó,
             người đọc SPINE sau này tin nhầm trade-off gốc đã giải triệt để (AC3).
Lưới DỐC    ⇒ cặp đánh đổi có thật; chọn giá trị TỪ CHÍNH BẢNG, đừng suy từ giả thuyết.

⚠️ `busy` là SỐ ĐẾM lượt checkpoint bị chặn, KHÔNG phải thời lượng chờ khoá. Độ trễ I-O
   thật cần đo TRONG tiến trình, tức mã sản phẩm, tức ngoài rào phạm vi story này.
   Ghi là "chưa đo được" kèm chủ — CẤM để ai đọc `busy=N` thành độ trễ I-O.

🔴 Mọi cửa sổ trong bảng là CẬN TRÊN: `type-driver.sh` ghi mốc bơm TRƯỚC lượt `osascript`,
   nên nó cộng luôn thời gian gõ ra chính câu đó (đo 2026-08-18: ≈ 0,4 s).
NOTE
