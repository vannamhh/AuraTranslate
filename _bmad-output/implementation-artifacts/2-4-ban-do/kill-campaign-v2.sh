#!/bin/zsh
# Bộ kill v2 — Story 2.4 sau lượt code review 2026-08-13 (commit 18829a2).
#
# Khác v1 ở BỐN chỗ, mỗi chỗ ứng một AC đã vá:
#
#  ① AC9 — PHÂN BA LOẠI, không phải hai. v1 vứt mọi lượt `.db-wal = 0` ra khỏi mẫu. Nhưng
#     `.db-wal = 0` có HAI nguyên nhân khác hẳn nhau:
#       · kill trúng lúc app RẢNH (checkpoint vừa TRUNCATE xong, không còn gì chờ ghi)
#         ⇒ mất 0 giây công việc ⇒ đây là kết quả THÀNH CÔNG, GIỮ trong mẫu
#       · kill TRƯỢT (tiến trình không chết, hoặc đi qua đường thoát bình thường)
#         ⇒ phép đo hỏng ⇒ bỏ và ghi ra
#     Phân biệt bằng chỉ số đơn điệu tăng: max_n trong kho == n cuối cùng đã bơm ⇒ rảnh thật.
#     Vứt cả hai như v1 là vứt đúng nhóm kết quả TỐT NHẤT ⇒ đẩy phân bố lệch về phía xấu hơn
#     thực tế. Đúng lớp lỗi Story 1.22 đã ghi tên hai lần.
#
#  ② AC2 — SÀN ĐẾM ĐÚNG CÁI CẦN ĐẾM: chạy tới khi đủ N mẫu HỢP LỆ, không phải bắn đúng N lượt.
#     Ghi ra CẢ HAI số (đã bắn / hợp lệ).
#
#  ③ AC10 vế ② — TRANH CHẤP giữa hai luồng checkpoint, đo từ NGOÀI, 0 dòng mã sản phẩm:
#     `checkpoint.rs::note()` in thẳng stderr, mỗi dòng mang tiền tố `store[global]` /
#     `store[project]`, và dòng `wal_checkpoint(PASSIVE) blocked: busy=N` là bằng chứng tranh
#     chấp TRỰC TIẾP. Cộng thời gian CPU TỪNG LUỒNG qua `ps -M` ngay trước lượt kill.
#     ⚠️ `busy=N` là SỐ ĐẾM, không phải thời lượng. Độ trễ I-O thật KHÔNG đo được từ đây —
#     ghi là "chưa đo được" kèm chủ, đừng để ai đọc `busy=N` thành nó.
#
#  ④ AC21 — MẤT TIÊU ĐIỂM cửa sổ làm phím không hạ cánh, và v1 không phát hiện được.
#     Lấy mẫu tiến trình frontmost trong lúc gõ; mất > 5% thời lượng ⇒ BỎ LƯỢT, chạy lại.
#
# Dùng: kill-campaign-v2.sh <mẫu-hợp-lệ-cần> <nhãn> [giây-min] [giây-max] [trần-lượt-bắn]
set -u
NEED="${1:-20}"
LABEL="${2:-run}"
TMIN="${3:-20}"
TMAX="${4:-55}"
CAP="${5:-$(( NEED * 2 ))}"

SCRATCH="${0:A:h}"
APP="/Users/hoangnam/LocalSites/addon/AuraTranslate/src-tauri/target/release/bundle/macos/AuraTranslate.app/Contents/MacOS/auratranslate"
DRAFT_HOME="$SCRATCH/draft-home"
APPDATA="$DRAFT_HOME/Library/Application Support/com.auratranslate.desktop"
LIB="$DRAFT_HOME/Documents/AuraTranslate"
OUT="$SCRATCH/kill2-$LABEL.tsv"
WALLOG="$SCRATCH/wal2-$LABEL.tsv"
CPULOG="$SCRATCH/cpu2-$LABEL.tsv"
SRC_FILE="${BENCH_SRC:-$SCRATCH/chapters/ladder-m.txt}"
WIN_X=200; WIN_Y=25
source "$SCRATCH/front.sh"

now() { perl -MTime::HiRes=time -e 'printf "%.3f\n", time' }
key() { osascript -e "tell application \"System Events\" to key code $1 using {control down, option down, shift down}" >/dev/null 2>&1 }
frontmost() { osascript -e 'tell application "System Events" to name of first application process whose frontmost is true' 2>/dev/null }

printf 'round\tkill_ts\tmax_n\tlast_n\tinject_ts\twindow_s\twal_project\twal_global\tclass\tbusy_project\tbusy_global\tpassive_project\tpassive_global\tthr_project\tthr_global\tblur_pct\tnote\n' > "$OUT"
printf 'round\tt\twal_project\twal_global\n' > "$WALLOG"
printf 'round\ttid\tcpu_time\n' > "$CPULOG"

shots=0; valid=0
while [ "$valid" -lt "$NEED" ] && [ "$shots" -lt "$CAP" ]; do
  shots=$((shots+1)); r=$shots
  echo "───── lượt bắn $shots · hợp lệ $valid/$NEED ─────"
  rm -rf "$LIB" "$APPDATA"; mkdir -p "$DRAFT_HOME/Documents"

  LOGF="$SCRATCH/app2-$LABEL-$r.log"
  HOME="$DRAFT_HOME" "$APP" > "$LOGF" 2>&1 &
  PID=$!

  for i in $(seq 1 60); do [ -f "$APPDATA/global.db" ] && break; sleep 0.5; done
  sleep 2

  # 🔴 v3: dựng Tác phẩm qua GIAO DIỆN THẬT, không qua `key 18` của bench.js.
  # `key 18` là phím do `bench.js:242-245` đăng ký — KHÔNG phải phím sản phẩm. Bộ kill v1 vì thế
  # phụ thuộc thẳng vào bàn đo tiêm, đúng cái đang bị chặn. Xem §Debug Log của story.
  "$SCRATCH/setup-gui.sh" $PID "$SRC_FILE" || { echo "  🔴 setup-gui đỏ (cổng tiêu điểm hoặc hình học)"; kill -9 $PID 2>/dev/null; printf '%s\t\t\t\t\t\t\t\tRIG_FAIL\t\t\t\t\t\t\t\tsetup-gui do\n' "$r" >> "$OUT"; continue; }
  for i in $(seq 1 80); do [ -n "$(find "$LIB" -name project.db 2>/dev/null | head -1)" ] && break; sleep 0.5; done
  DB=$(find "$LIB" -name project.db 2>/dev/null | head -1)
  if [ -z "$DB" ]; then
    echo "  🔴 không dựng được Tác phẩm — lượt bắn hỏng, KHÔNG tính vào mẫu"
    kill -9 $PID 2>/dev/null; wait $PID 2>/dev/null
    printf '%s\t\t\t\t\t\t\t\tRIG_FAIL\t\t\t\t\t\t\t\tkhông dựng được Tác phẩm\n' "$r" >> "$OUT"
    continue
  fi
  sleep 4

  # mở Workspace rồi đặt con trỏ vào MỘT CÂU — cliclick phát mousedown THẬT.
  # `System Events ... click at` KHÔNG đặt được con trỏ vào contenteditable (đo được: kho trả 0).
  # Toạ độ tương đối gốc cửa sổ, hiệu chuẩn bằng lưới 20 điểm: ô đầu dòng ở (+640, +165).
  require_front $PID 6 >/dev/null || { echo "  🔴 mất tiêu điểm trước khi mở Workspace"; kill -9 $PID 2>/dev/null; continue; }
  cliclick c:$((WIN_X + 101)),$((WIN_Y + 46))      # tab Workspace
  sleep 4
  require_front $PID 6 >/dev/null || { echo "  🔴 mất tiêu điểm trước khi đặt con trỏ"; kill -9 $PID 2>/dev/null; continue; }
  # 🔴 KHONG tin mot toa do: vung trung nho va khong on dinh giua cac luot (do duoc).
  # `focus-segment.sh` bam -> go chuoi do -> HOI KHO -> truot thi thu diem ke.
  if ! HIT=$("$SCRATCH/focus-segment.sh" $PID "$DB" 2>/dev/null); then
    echo "  🔴 khong dat duoc con tro vao mot cau — bo luot"
    kill -9 $PID 2>/dev/null
    printf '%s\t\t\t\t\t\t\t\tRIG_FAIL\t\t\t\t\t\t\t\tkhong dat duoc con tro\n' "$r" >> "$OUT"
    continue
  fi
  echo "  con tro vao duoc tai (+$HIT)"

  DUR=$(( TMIN + RANDOM % (TMAX - TMIN + 1) ))
  echo "  gõ ${DUR}s rồi SIGKILL"
  TYPELOG="$SCRATCH/typing2-$LABEL-$r.log"
  "$SCRATCH/type-driver.sh" "$DUR" "$TYPELOG" >/dev/null 2>&1 &
  TYPER=$!

  # lấy mẫu .db-wal CẢ HAI kho + tiêu điểm cửa sổ (AC10 vế ① · AC21)
  BLURF="$SCRATCH/.blur-$LABEL-$r"; : > "$BLURF"
  ( while kill -0 $TYPER 2>/dev/null; do
      printf '%s\t%s\t%s\t%s\n' "$r" "$(now)" \
        "$(stat -f '%z' "$DB-wal" 2>/dev/null || echo 0)" \
        "$(stat -f '%z' "$APPDATA/global.db-wal" 2>/dev/null || echo 0)" >> "$WALLOG"
      echo "$(frontmost)" >> "$BLURF"
      sleep 1
    done ) &
  SAMPLER=$!

  wait $TYPER 2>/dev/null
  kill $SAMPLER 2>/dev/null

  # AC10 vế ② — thời gian CPU TỪNG LUỒNG, ngay trước lượt kill
  ps -M "$PID" 2>/dev/null | awk -v r="$r" 'NR>1 && NF>=6 {print r"\t"NR-1"\t"$(NF-2)}' >> "$CPULOG"

  WALP=$(stat -f '%z' "$DB-wal" 2>/dev/null || echo 0)
  WALG=$(stat -f '%z' "$APPDATA/global.db-wal" 2>/dev/null || echo 0)
  KILL_TS=$(now)
  kill -9 $PID 2>/dev/null
  wait $PID 2>/dev/null
  sleep 1

  # ── AC21: tiêu điểm ──────────────────────────────────────────────────────────
  TOT=$(wc -l < "$BLURF" | tr -d ' '); [ "$TOT" -eq 0 ] && TOT=1
  # 🔴 KHONG phan biet hoa thuong: `frontmost` tra ve `auratranslate` (chu thuong, ten nhi
  # phan), khong phai `AuraTranslate` (ten bundle). Ban truoc dung `grep -cv 'AuraTranslate'`
  # ⇒ MOI mau bi dem la mat tieu diem ⇒ blur 100% ⇒ bo sach moi luot. Hang rao bao oan chinh no.
  OFF=$(grep -civ 'auratranslate' "$BLURF" 2>/dev/null || echo 0)
  BLURPCT=$(perl -e "printf '%.1f', 100*$OFF/$TOT")
  rm -f "$BLURF"

  # ── chẩn đoán checkpoint, tách theo KHO (AC10) ───────────────────────────────
  BUSYP=$(grep -c 'store\[project\].*blocked' "$LOGF" 2>/dev/null || echo 0)
  BUSYG=$(grep -c 'store\[global\].*blocked'  "$LOGF" 2>/dev/null || echo 0)
  PASP=$(grep -c 'store\[project\].*PASSIVE'  "$LOGF" 2>/dev/null || echo 0)
  PASG=$(grep -c 'store\[global\].*PASSIVE'   "$LOGF" 2>/dev/null || echo 0)
  THRP=$(grep -c 'store\[project\].*threshold' "$LOGF" 2>/dev/null || echo 0)
  THRG=$(grep -c 'store\[global\].*threshold'  "$LOGF" 2>/dev/null || echo 0)

  # ── đọc kho bằng sqlite3 CHỈ ĐỌC, không mở app lại (Quyết định #4) ───────────
  # 🔴 Rut chi so LON NHAT trong TOAN BO van ban, khong phai chi so DAU TIEN cua moi segment.
  # `instr(t,']')` chi thay dau `]` dau tien ⇒ mot segment chua `[7] ... [42] ...` tra ve 7.
  # Do duoc 2026-08-13: ky vong 42, truy van cu tra 7. Rut bang grep roi lay max.
  MAXN=$(sqlite3 -readonly "$DB" "select target_text from segment where target_text like '%[%]%';" 2>/dev/null \
         | grep -oE '\[[0-9]+\]' | tr -d '[]' | sort -n | tail -1)
  [ -z "$MAXN" ] && MAXN=0
  LASTN=$(awk '!/^#/ {n=$1} END {print n+0}' "$TYPELOG" 2>/dev/null); [ -z "$LASTN" ] && LASTN=0

  # ── AC9: PHÂN BA LOẠI ────────────────────────────────────────────────────────
  CLASS=""; NOTE=""; WIN=""
  if [ "$WALP" -gt 0 ]; then
    CLASS="VALID"
    INJ=$(awk -v n="$MAXN" '!/^#/ && $1==n {print $2; exit}' "$TYPELOG" 2>/dev/null)
    if [ -n "$INJ" ] && [ "$MAXN" -gt 0 ]; then
      WIN=$(perl -e "printf '%.3f', $KILL_TS - $INJ")
    else
      CLASS="AMBIG"; NOTE="wal>0 nhưng không truy ngược được chỉ số (max_n=$MAXN)"
    fi
  elif [ "$MAXN" -gt 0 ] && [ "$MAXN" -eq "$LASTN" ]; then
    # Kho đã nuốt trọn dòng gõ VÀ wal rỗng ⇒ checkpoint vừa TRUNCATE xong ⇒ mất 0 giây.
    CLASS="VALID_IDLE"; WIN="0.000"
    NOTE="kill trúng lúc rảnh — mất 0 s, GIỮ trong mẫu (AC9)"
  else
    CLASS="MISS"
    NOTE="wal=0 và max_n=$MAXN < last_n=$LASTN ⇒ không phân biệt được, bỏ khỏi mẫu"
  fi

  # AC21: mất tiêu điểm > 5% ⇒ phiên hỏng, không tính dù phân loại là gì
  if [ "$(perl -e "print(($BLURPCT>5.0)?1:0)")" -eq 1 ]; then
    CLASS="BLUR_FAIL"; NOTE="mất tiêu điểm ${BLURPCT}% > 5% ⇒ bỏ lượt (AC21)"
  fi

  case "$CLASS" in VALID|VALID_IDLE) valid=$((valid+1));; esac

  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$r" "$KILL_TS" "$MAXN" "$LASTN" "${INJ:-}" "${WIN:-}" "$WALP" "$WALG" \
    "$CLASS" "$BUSYP" "$BUSYG" "$PASP" "$PASG" "$THRP" "$THRG" "$BLURPCT" "$NOTE" >> "$OUT"
  echo "  $CLASS  cửa sổ=${WIN:-?}s  wal(p/g)=$WALP/$WALG  busy(p/g)=$BUSYP/$BUSYG  blur=${BLURPCT}%  $NOTE"
done

echo "═══ xong · đã bắn $shots · hợp lệ $valid/$NEED · $OUT ═══"
[ "$valid" -lt "$NEED" ] && echo "🔴 CHƯA ĐỦ MẪU — chạm trần $CAP lượt bắn. Đây là một kết quả phải ghi ra, không phải lỗi."
