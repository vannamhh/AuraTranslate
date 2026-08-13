#!/bin/zsh
# Tự kiểm bộ phân loại AC9 của `kill-campaign-v2.sh` — chạy TRƯỚC lượt kill thật.
#
# Lý do tồn tại: v1 chỉ có hai nhánh và nhánh sai của nó vứt đúng nhóm kết quả TỐT NHẤT.
# Một bộ phân loại chưa ai thấy nó phân sai là một bộ phân loại chưa ai biết nó phân đúng.
# Năm ca dưới đây gồm CẢ ca phải rơi vào MISS — nếu mọi ca đều VALID thì bộ này vô dụng.
set -u

classify() {  # walp maxn lastn blurpct -> "CLASS WIN"
  local WALP=$1 MAXN=$2 LASTN=$3 BLURPCT=$4
  local CLASS="" WIN=""
  if [ "$WALP" -gt 0 ]; then
    if [ "$MAXN" -gt 0 ]; then CLASS="VALID"; WIN="12.345"; else CLASS="AMBIG"; fi
  elif [ "$MAXN" -gt 0 ] && [ "$MAXN" -eq "$LASTN" ]; then
    CLASS="VALID_IDLE"; WIN="0.000"
  else
    CLASS="MISS"
  fi
  if [ "$(perl -e "print(($BLURPCT>5.0)?1:0)")" -eq 1 ]; then CLASS="BLUR_FAIL"; WIN=""; fi
  echo "$CLASS ${WIN:-—}"
}

fail=0
check() {  # nhãn kỳ-vọng thực-tế
  if [ "$2" = "$3" ]; then printf '  🟢 %-46s %s\n' "$1" "$3"
  else printf '  🔴 %-46s kỳ vọng[%s] nhận[%s]\n' "$1" "$2" "$3"; fail=$((fail+1)); fi
}

echo "── Tự kiểm bộ phân loại AC9 ──"
check "wal>0, truy ngược được chỉ số"            "VALID 12.345"      "$(classify 8192 41 44 0.0)"
check "wal>0, KHÔNG truy ngược được"             "AMBIG —"           "$(classify 8192  0 44 0.0)"
check "wal=0, kho nuốt trọn ⇒ kill lúc RẢNH"     "VALID_IDLE 0.000"  "$(classify    0 44 44 0.0)"
check "wal=0, kho tụt sau ⇒ TRƯỢT"               "MISS —"            "$(classify    0 30 44 0.0)"
check "wal=0, kho rỗng ⇒ TRƯỢT"                  "MISS —"            "$(classify    0  0 44 0.0)"
check "mất tiêu điểm 9,2% ⇒ phủ quyết mọi lớp"   "BLUR_FAIL —"       "$(classify 8192 41 44 9.2)"
check "mất tiêu điểm 4,9% ⇒ CHƯA phủ quyết"      "VALID 12.345"      "$(classify 8192 41 44 4.9)"

echo
if [ "$fail" -eq 0 ]; then
  echo "✅ 7/7 xanh — và ba ca trong đó BẮT BUỘC phải đỏ ở một bộ phân loại hỏng"
  echo "   (VALID_IDLE, MISS, BLUR_FAIL). v1 sẽ trượt ca VALID_IDLE: nó gọi ca đó là 'trượt'."
else
  echo "🔴 $fail ca ĐỎ"
  exit 1
fi
