#!/bin/zsh
# Đối chứng AC21 — vế "cùng bàn đo, cùng thời lượng, KHÔNG gõ".
#
# 🔴 VÌ SAO NÓ CẦN. AC21 nêu BỐN nguồn chi phí chạy trên CÙNG cái máy với thứ đang đo. Với nửa
# NFR18 hôm nay, hai nguồn vắng mặt (vòng rAF không chạy vì bench chưa tiêm; vòng đo tranh chấp
# của AC10 vế ② chưa dựng) và hai nguồn có mặt: vòng `stat()` và BỘ BƠM PHÍM.
#
# Vòng `stat()` đã được bác bằng lượt đổi nhịp (`WAL_EVERY=4` cho cùng phân bố). Tệp này đo
# nguồn còn lại: `osascript` — mỗi câu là một tiến trình `osascript` MỚI, và đó là thứ đắt.
#
# Ba chế độ, mỗi chế độ cùng thời lượng, đo `loadavg` 1 phút:
#   ① máy trần            — nền của chính máy đo
#   ② + vòng lấy mẫu      — `stat()` + `frontmost` mỗi 1 s
#   ③ + bộ bơm phím       — thêm `osascript keystroke` theo đúng nhịp type-driver
set -u
SCRATCH="${0:A:h}"
DUR="${1:-25}"
load() { sysctl -n vm.loadavg | awk '{print $2}' }

echo "che-do\tloadavg_truoc\tloadavg_sau\tdelta"

# ① máy trần
A=$(load); sleep $DUR; B=$(load)
printf '① may tran\t%s\t%s\t%s\n' "$A" "$B" "$(perl -e "printf '%+.2f', $B-$A")"

# ② vòng lấy mẫu
A=$(load)
( END=$((SECONDS+DUR)); while [ $SECONDS -lt $END ]; do
    stat -f '%z' "$SCRATCH/draft-home/Library/Application Support/com.auratranslate.desktop/global.db-wal" >/dev/null 2>&1
    osascript -e 'tell application "System Events" to name of first application process whose frontmost is true' >/dev/null 2>&1
    sleep 1
  done ) 
B=$(load)
printf '② + vong lay mau\t%s\t%s\t%s\n' "$A" "$B" "$(perl -e "printf '%+.2f', $B-$A")"

# ③ vòng lấy mẫu + bộ bơm phím (gõ vào HƯ KHÔNG — không app nào ở trước để nhận)
A=$(load)
( END=$((SECONDS+DUR)); while [ $SECONDS -lt $END ]; do
    stat -f '%z' "$SCRATCH/draft-home/Library/Application Support/com.auratranslate.desktop/global.db-wal" >/dev/null 2>&1
    osascript -e 'tell application "System Events" to name of first application process whose frontmost is true' >/dev/null 2>&1
    sleep 1
  done ) &
S=$!
( END=$((SECONDS+DUR)); n=0; while [ $SECONDS -lt $END ]; do
    n=$((n+1))
    osascript -e 'tell application "System Events" to get name of current application' >/dev/null 2>&1
    perl -e 'select(undef,undef,undef, 0.6 + rand(0.8))'
  done ) 
kill $S 2>/dev/null
B=$(load)
printf '③ + bo bom phim\t%s\t%s\t%s\n' "$A" "$B" "$(perl -e "printf '%+.2f', $B-$A")"
