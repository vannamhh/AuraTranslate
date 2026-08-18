#!/bin/zsh
# ══════════════════════════════════════════════════════════════════════════════════════
# LƯỚI SÁU ĐIỂM `wal_threshold_bytes` — Task 5 + AC8 của Story 2.4
# ══════════════════════════════════════════════════════════════════════════════════════
# Ice chạy tệp này khi máy RẢNH. Nó chạy ~3,5 giờ và ĐỘC CHIẾM bàn phím + chuột suốt thời
# gian đó — xem §Điều kiện tiên quyết dưới đây, chúng được kiểm bằng máy chứ không bằng lời.
#
#   cd _bmad-output/implementation-artifacts/2-4-ban-do && ./run-grid.sh
#
# ── VÌ SAO MỖI ĐIỂM CẦN MỘT LƯỢT DỰNG RIÊNG ───────────────────────────────────────────
# `wal_threshold_bytes` sống trong `Tuning::default` (`core/store/mod.rs`) và KHÔNG có đường
# ghi đè lúc chạy. Dựng một đường như thế là thêm mã sản phẩm và phải đi qua hai lớp gác của
# AD-45 — ngoài rào phạm vi của một mũi thăm dò. ⇒ Đổi hằng, dựng lại, đo. Sáu lần.
#
# ── LƯỚI ĐÃ GHIM, KHÔNG PHẢI CHỮ "THEO LƯỚI" ──────────────────────────────────────────
# 512 KiB · 1 · 2 · 4 · 8 · 16 MiB — sáu điểm, năm bậc gấp đôi, ôm cả hai đầu.
# Điều kiện dừng: chạy HẾT sáu điểm. Nhánh một-ngưỡng-trượt của AC5 treo lên đúng chỗ này —
# chưa hết lưới thì CHƯA được báo Ice.
#
# ── HAI THỨ TỆP NÀY TỰ LÀM, ĐỪNG LÀM TAY ──────────────────────────────────────────────
# ① TRẢ LẠI hằng số: `trap` chạy `git checkout` trên `mod.rs` kể cả khi Ice bấm Ctrl-C.
#    Bỏ máy giữa chừng KHÔNG để lại một hằng đo dở trong cây nguồn.
# ② CHẠY LẠI ĐƯỢC: điểm nào đã đủ mẫu hợp lệ thì BỎ QUA. Một lượt 3,5 giờ mà chết ở điểm
#    thứ tư không được bắt làm lại từ điểm thứ nhất.
set -u

SCRATCH="${0:A:h}"
REPO="${SCRATCH:h:h:h}"
MODRS="$REPO/src-tauri/src/core/store/mod.rs"
APPBIN="$REPO/src-tauri/target/release/bundle/macos/AuraTranslate.app/Contents/MacOS/auratranslate"

NEED="${NEED:-20}"        # mẫu HỢP LỆ mỗi điểm (sàn AC2)
CAP="${CAP:-45}"          # trần lượt bắn mỗi điểm
TMIN="${TMIN:-20}"; TMAX="${TMAX:-55}"

# byte:nhãn — nhãn đi vào tên tệp kết quả
POINTS=(
  "524288:512kib"
  "1048576:1mib"
  "2097152:2mib"
  "4194304:4mib"
  "8388608:8mib"
  "16777216:16mib"
)

say() { print -r -- "$@" }
die() { print -r -- "🔴 $@" >&2; exit 1 }

restore() {
  say ""
  say "── trả lại hằng số về nguyên trạng ──"
  git -C "$REPO" checkout -- src-tauri/src/core/store/mod.rs 2>/dev/null \
    && say "🟢 mod.rs đã trả lại" || say "🔴 KHÔNG trả lại được mod.rs — kiểm bằng tay!"
}
# 🔴 `trap` CỐ Ý chưa cài ở đây — xem chỗ cài nó bên dưới, sau phép kiểm `mod.rs` sạch.

# ══ ĐIỀU KIỆN TIÊN QUYẾT — kiểm bằng máy, thiếu một cái là DỪNG ═══════════════════════
say "══ điều kiện tiên quyết ══"

[ -x "$(command -v cliclick)" ] || die "chưa cài cliclick  (brew install cliclick)"
say "🟢 cliclick"

# 🔴 `mod.rs` phải SẠCH: tệp này sẽ sửa rồi `git checkout` để trả lại, nên một thay đổi chưa
# commit ở đó sẽ bị XOÁ MẤT. Đây là chỗ duy nhất tệp này có thể huỷ việc của người khác.
# Ba ca khác nhau, và ca giữa CHỈ có ở tệp này nên nó phải được gọi tên riêng:
#   ① mod.rs sạch                              ⇒ đi tiếp
#   ② bẩn, và diff CHỈ là dòng `wal_threshold_bytes`
#      ⇒ tàn dư của một lượt chạy TRƯỚC bị giết cứng (SIGKILL / mất điện / treo máy), lúc đó
#        `trap` không kịp chạy. Đường đúng là `git checkout`, KHÔNG phải "commit hoặc stash" —
#        khuyên commit ở đây là khuyên đưa một hằng số đo dở vào lịch sử kho.
#   ③ bẩn vì chuyện khác                       ⇒ DỪNG, đó là việc của người khác
if [ -n "$(git -C "$REPO" status --porcelain -- src-tauri/src/core/store/mod.rs)" ]; then
  OTHER=$(git -C "$REPO" diff -U0 -- src-tauri/src/core/store/mod.rs \
          | grep -E '^[+-][^+-]' | grep -vE '^[+-][[:space:]]*wal_threshold_bytes:')
  [ -n "$OTHER" ] && die "mod.rs đang có thay đổi CHƯA COMMIT ngoài \`wal_threshold_bytes\`. Tệp này trả lại hằng số bằng \`git checkout\` nên nó sẽ xoá mất. Commit hoặc stash trước."
  say "⚠️  mod.rs chỉ lệch ở \`wal_threshold_bytes\` — tàn dư của một lượt chạy bị giết cứng."
  git -C "$REPO" checkout -- src-tauri/src/core/store/mod.rs || die "không trả lại được mod.rs"
  say "🟢 đã trả lại tự động"
else
  say "🟢 mod.rs sạch"
fi

# 🔴 CHỖ CÀI `trap`, VÀ THỨ TỰ NÀY LÀ CÓ LÝ DO — không dời lên trên.
# `restore` gọi `git checkout -- mod.rs`. Cài `trap` TRƯỚC phép kiểm sạch ở trên thì một
# lượt `die()` của chính phép kiểm đó sẽ kích `trap` và XOÁ đúng thay đổi chưa commit mà nó
# vừa từ chối đụng vào — hàng rào tự tay huỷ thứ nó canh.
# ⇒ Chỉ cài sau khi đã biết chắc `mod.rs` không mang việc của ai khác.
trap restore EXIT INT TERM

# 🔴 `pgrep -x` (khớp TÊN tiến trình), KHÔNG `pgrep -f` (khớp cả dòng lệnh).
# Đo được 2026-08-18: `pgrep -f 'auratranslate'` khớp một shell hoàn toàn vô can chỉ vì dòng
# lệnh của nó CHỨA đường dẫn nhị phân — hàng rào báo ĐỎ trong khi 0 tiến trình app nào chạy.
# Một hàng rào báo oan còn tệ hơn không có: nó dạy người chạy bỏ qua chính nó.
# `-x auratranslate` phủ CẢ bản release lẫn `tauri dev` — hai bản dùng chung tên nhị phân.
pgrep -x auratranslate >/dev/null \
  && die "còn một instance app đang chạy — có thể nó đang mở DỮ LIỆU THẬT. Đóng hết rồi chạy lại." \
  || say "🟢 không instance app nào đang chạy"

./classify-selftest.sh >/dev/null 2>&1 || die "bộ phân loại AC9 tự kiểm ĐỎ — đừng đốt một lượt kill nào"
say "🟢 bộ phân loại AC9 tự kiểm 7/7"

# Hàng rào chiều âm chạy TRƯỚC lượt kill đầu tiên, không sau (bài học Story 1.22).
./fence.sh snap grid-before >/dev/null || die "không chụp được ảnh gốc dữ liệu thật"
say "🟢 ảnh gốc dữ liệu thật: $(grep -c . fence-grid-before.txt) dòng"

say ""
say "══ lưới sáu điểm · $NEED mẫu hợp lệ mỗi điểm · trần $CAP lượt bắn ══"
say "⚠️  ĐỘC CHIẾM bàn phím và chuột từ đây. Đừng chạm máy tới khi xong."
say ""

for entry in "${POINTS[@]}"; do
  BYTES="${entry%%:*}"; LABEL="${entry##*:}"
  OUT="$SCRATCH/kill2-g$LABEL.tsv"

  # ── chạy lại được: điểm đã đủ mẫu thì bỏ qua ────────────────────────────────────────
  if [ -f "$OUT" ]; then
    DONE=$(awk -F'\t' 'NR>1 && ($9=="VALID" || $9=="VALID_IDLE")' "$OUT" | wc -l | tr -d ' ')
    if [ "${DONE:-0}" -ge "$NEED" ]; then
      say "⏭  $LABEL — đã có $DONE mẫu hợp lệ, BỎ QUA"
      continue
    fi
    say "↻  $LABEL — mới có ${DONE:-0}/$NEED mẫu, chạy lại điểm này"
  fi

  say ""
  say "═══════ ĐIỂM $LABEL ($BYTES byte) ═══════"

  # ── đổi hằng ở ĐÚNG MỘT CHỖ KHAI (AC14) ────────────────────────────────────────────
  sed -i '' -E "s/^([[:space:]]*)wal_threshold_bytes: .*,$/\1wal_threshold_bytes: $BYTES,/" "$MODRS"
  grep -qE "^[[:space:]]*wal_threshold_bytes: $BYTES,$" "$MODRS" \
    || die "lượt đổi hằng KHÔNG ăn — `Tuning::default` đã đổi hình dạng, đọc lại mod.rs"
  say "🟢 đặt-rồi-ĐỌC-LẠI: $(grep -nE '^[[:space:]]*wal_threshold_bytes:' "$MODRS")"

  # ── dựng lại release ───────────────────────────────────────────────────────────────
  # 🔴 `build.rs:66` khai `cargo:rerun-if-changed=windows-app-manifest.xml`. Một khi build
  # script phát BẤT KỲ `rerun-if-changed` nào, cargo chỉ theo dõi ĐÚNG danh sách đó ⇒ phải
  # chạm đúng tệp mà build.rs khai là đầu vào, nếu không tài nguyên nhúng giữ nguyên bản CŨ
  # và lượt đo sẽ đo một nhị phân không phải cái mình vừa đổi. Đo được 2026-08-13.
  touch "$REPO/src-tauri/windows-app-manifest.xml" "$REPO/src-tauri/build.rs"
  say "   dựng release (~10 phút)…"
  ( cd "$REPO" && npx tauri build --bundles app ) >"$SCRATCH/build-g$LABEL.log" 2>&1 \
    || { tail -20 "$SCRATCH/build-g$LABEL.log"; die "dựng ĐỎ ở điểm $LABEL"; }
  [ "$APPBIN" -nt "$REPO/src-tauri/build.rs" ] \
    || die "nhị phân CŨ hơn build.rs — tài nguyên nhúng chưa dựng lại, số đo sẽ là của bản trước"
  say "🟢 nhị phân: $(stat -f '%z B  %Sm' "$APPBIN")"

  # ── đo ─────────────────────────────────────────────────────────────────────────────
  ./kill-campaign-v2.sh "$NEED" "g$LABEL" "$TMIN" "$TMAX" "$CAP"
done

say ""
say "══ xong cả sáu điểm ══"
./fence.sh snap grid-after >/dev/null
./fence.sh diff grid-before grid-after
say ""
./grid-table.sh
