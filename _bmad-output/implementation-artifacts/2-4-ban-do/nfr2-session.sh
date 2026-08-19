#!/bin/zsh
# Phiên đo NFR2 của Story 2.4 — AC1 · AC11 · AC12 · AC13 · AC21 · AC22.
#
# Dùng: nfr2-session.sh <thang: xs|s|m|l> <giây-gõ> <nhãn> [control]
#   control ⇒ lượt ĐỐI CHỨNG của AC21: cùng bàn đo, cùng thời lượng, KHÔNG gõ.
set -u
LAD="${1:-m}"; DUR="${2:-1800}"; LABEL="${3:-run}"; MODE="${4:-typing}"
SCRATCH="${0:A:h}"
APP="/Users/hoangnam/LocalSites/addon/AuraTranslate/src-tauri/target/release/bundle/macos/AuraTranslate.app/Contents/MacOS/auratranslate"
DRAFT="$SCRATCH/draft-home"
APPDATA="$DRAFT/Library/Application Support/com.auratranslate.desktop"
# 🔵 2026-08-19 — HAI DÒNG TOẠ ĐỘ CHẾT ĐÃ GỠ, và đây là lý do chúng nguy hiểm hơn một lỗi thường.
#
# Bản trước ghim `CLICK_X=980 CLICK_Y=430` rồi bấm thẳng bằng `osascript`, KHÔNG nghiệm thu.
# Hai chuyện đã xảy ra sau khi hai số đó được viết ra:
#   ① Hiệu chuẩn lại 2026-08-18 dời ô gõ về `(+372, +170)` TƯƠNG ĐỐI gốc cửa sổ — `x = +640`
#      của bộ cũ nay rơi vào panel Tra cứu (xem `focus-segment.sh` §HIỆU CHUẨN LẠI). Tệp này
#      là tệp DUY NHẤT không được cập nhật theo; `kill-campaign-v2.sh` đã đi qua đường mới.
#   ② `focus-segment.sh` ra đời chính vì một toạ độ cố định KHÔNG tin được: cùng toạ độ, cùng
#      hình học cửa sổ, lượt này ăn lượt sau không.
#
# 🔴 Vì sao nó tệ hơn một cú bấm trượt bình thường: trượt ô ⇒ `type-driver.sh` gõ vào hư không
# ⇒ bàn đo vẫn lấy mẫu frame đủ 30 phút và vẫn đổ ra một tệp JSON trông bình thường, nhưng đó
# là số của một phiên KHÔNG AI GÕ. Tức NFR2 sẽ XANH trên một phép đo hỏng — đúng lớp lỗi
# "hàng rào báo sai" mà story này đã vá ba ca ngày 2026-08-18.
#
# ⇒ Đường duy nhất là `focus-segment.sh`: bấm → gõ ký tự dò → HỎI KHO → xoá. Nó trả mã khác 0
# khi cả 16 ứng viên đều trượt, và ở đây một lượt trượt phải là DỪNG, không phải một cảnh báo.

source "$SCRATCH/front.sh"
WIN_X=200; WIN_Y=25          # hình học chuẩn hoá, do `setup-gui.sh` đặt rồi ĐỌC LẠI
TAB_WS_DX=101; TAB_WS_DY=46  # tab Workspace, tương đối gốc cửa sổ

key() { osascript -e "tell application \"System Events\" to key code $1 using {control down, option down, shift down}" >/dev/null 2>&1 }
# mã phím: 1→18  2→19  3→20  4→21  5→23  6→22
now() { perl -MTime::HiRes=time -e 'printf "%.3f\n", time' }

# ── CỔNG "APP ĐANG NHẬN PHÍM", chạy TRƯỚC hai phím quyết định ─────────────────────────
# 🔴 Đo được 2026-08-19, hai phiên liên tiếp cùng tham số: phím đổ số phải bắn **2** lượt rồi
# **5** lượt mới tới nơi (trần là 5 — sát mép). Nguyên nhân chưa định vị, nhưng nó chỉ xuất
# hiện SAU một phiên gõ dài, và vòng rAF của bench vẫn chạy suốt ⇒ không phải app treo.
#
# ⇒ Đừng bắn mù vào một app có thể đang không nhận. Hỏi trước: gửi phím 7 và chờ bộ đếm phím
# của bench TĂNG. Bộ đếm là cột thứ sáu của `__cellrect__` (`bench.js` §⑦b). Đếm tăng nghĩa là
# app đang nhận phím NGAY LÚC NÀY — một mốc dừng đo được, thay cho một `sleep` đoán.
settle_keys() {   # $1 = số giây tối đa chờ
  local deadline=$(( $(date +%s) + ${1:-60} )) before after
  before=$(sqlite3 -readonly "$APPDATA/global.db" "select value from config_value where key='__cellrect__';" 2>/dev/null | cut -d, -f6)
  while [ "$(date +%s)" -lt "$deadline" ]; do
    require_front "$PID" 4 >/dev/null || return 1
    key 26; sleep 1.5
    after=$(sqlite3 -readonly "$APPDATA/global.db" "select value from config_value where key='__cellrect__';" 2>/dev/null | cut -d, -f6)
    [ -n "$after" ] && [ "${after:-0}" -gt "${before:-0}" ] 2>/dev/null && { print -r -- "${after:-0}"; return 0; }
  done
  return 1
}

# ── AC22: TRẠNG THÁI MÁY, ghi theo TỪNG PHIÊN ─────────────────────────────────────────
# 🔴 Đây là điều kiện, không phải thủ tục — và số đo mới làm nó thành điều kiện. Bốn phiên
# smoke ngày 2026-08-19, CÙNG bàn đo · CÙNG tham số · CÙNG thang, cho hai cụm tách bạch:
# hai phiên ~9 % frame vượt trần 50 ms *(max 101–113 ms)* và hai phiên **39 %** *(max
# 313–321 ms)*. Một phân bố hai cụm như thế không giải thích được bằng nhiễu đo; nó đòi một
# biến chưa kiểm soát, và nghi can đầu là tải nền.
# ⇒ Ghi `loadavg` + tiết lưu + nguồn điện ở CẢ HAI đầu phiên. 🔴 Và CẤM gộp mẫu giữa các
# phiên — AC2 đã cấm, số này nói vì sao lệnh cấm đó không phải hình thức.
ENVF="$SCRATCH/env-$LABEL.txt"
snap_env() {   # $1 = nhãn thời điểm
  {
    printf '── %s · %s\n' "$1" "$(date '+%Y-%m-%d %H:%M:%S')"
    printf '   loadavg      : %s\n' "$(sysctl -n vm.loadavg)"
    printf '   ncpu         : %s\n' "$(sysctl -n hw.ncpu)"
    # 🔵 Sửa 2026-08-19, cùng ngày với lượt viết ra. Bản đầu tách bằng `-F': *'` và trường
    # này RỖNG ở cả bốn tệp sinh ra — `pmset -g therm` in `CPU_Speed_Limit \t= 100`, tức
    # phân tách bằng TAB + `=`, không phải `: `. Một trường AC22 luôn rỗng là đúng lớp "rỗng
    # im lặng" mà kho cấm: nó nằm im trong hồ sơ và người đọc sau tưởng đã có ai xét.
    # 🔴 `|| echo KHONG DOC DUOC` — thiếu số thì phải NÓI, không để một dòng trắng.
    printf '   CPU_Speed_Limit: %s\n' "$(pmset -g therm 2>/dev/null | awk -F'=' '/CPU_Speed_Limit/{gsub(/ /,"",$2); print $2}' | head -1 || true)"
    printf '   nguon dien   : %s\n' "$(pmset -g batt 2>/dev/null | head -1)"
    # 🔵 Sửa cùng lượt: bản đầu viết `%%%%` và in ra `23.1%%` nguyên văn — thoát dư một lớp.
    printf '   top-CPU      : %s\n' "$(ps -Ao pcpu,comm -r | awk 'NR>1&&NR<=4{printf "%s%% %s  ", $1, $2}')"
  } >> "$ENVF"
}
# 🔴 CỔNG MÀN HÌNH KHOÁ — hỏi TRƯỚC khi khởi động app, không sau. Xem §CỔNG MÀN HÌNH KHOÁ
# ở `front.sh`: một phiên chạy trên máy khoá vẫn ghi được dấu sống rồi chết ở cổng tiêu điểm,
# tức nó báo ra một nguyên nhân KHÁC HẲN nguyên nhân thật.
require_unlocked || exit 1

: > "$ENVF"
snap_env "TRUOC phien $LABEL (thang=$LAD, ${DUR}s)"

cp "$SCRATCH/chapters/ladder-$LAD.txt" /tmp/at-bench-chapter.txt
rm -rf "$DRAFT"; mkdir -p "$DRAFT/Documents"

echo "== khởi động (thang=$LAD · $(wc -m < /tmp/at-bench-chapter.txt) ký tự · chế độ=$MODE) =="
HOME="$DRAFT" "$APP" > "$SCRATCH/app-$LABEL.log" 2>&1 &
PID=$!
for i in $(seq 1 60); do [ -f "$APPDATA/global.db" ] && break; sleep 0.5; done
sleep 3

# 🔴 HỎI DẤU SỐNG TRƯỚC KHI ĐỐT MỘT PHÚT NÀO — thêm 2026-08-19.
# Bản trước không hỏi. Nếu `bench.js` không có trong webview *(nhị phân dựng bằng
# `npx tauri build` trần thay vì `build-bench.sh`, hoặc `beforeBuildCommand` đã dựng lại
# `dist/` và xoá lượt tiêm)*, phiên vẫn chạy trọn 30 phút, vẫn `sqlite3` ra một tệp, và tệp
# đó **RỖNG** — rồi script thoát **0**. Đó là đúng lớp lỗi trung tâm của dự án: một kết quả
# rỗng không tự nói vì sao nó rỗng, và ở đây nó đội lốt một phiên đo thành công.
# ⇒ Hỏi ngay sau khi `global.db` xuất hiện, và trượt là DỪNG.
ALIVE=$(sqlite3 -readonly "$APPDATA/global.db" "select value from config_value where kind='app_config' and key='__bench_alive__';" 2>/dev/null) || true
if [ -z "$ALIVE" ]; then
  echo "🔴 DỪNG — không có dấu sống \`__bench_alive__\`: bench.js KHÔNG chạy trong webview này."
  echo "   Nhị phân hiện tại không phải bản của \`build-bench.sh\`. Chạy \`./build-bench.sh\` trước."
  kill -9 $PID 2>/dev/null; exit 1
fi
echo "   🟢 dấu sống bench = $ALIVE"

# 🔵 VIẾT LẠI 2026-08-19 — KHÚC NÀY TRƯỢT VÌ THIẾU HAI BƯỚC MÀ `kill-campaign-v2.sh` CÓ.
# Lượt thử 5 phút đầu tiên chết ở `focus-segment.sh` với *"KHÔNG điểm nào trong 16 ứng viên"*.
# So hai tệp thì ra, và cả hai chỗ thiếu đều là chỗ ĐÃ CÓ BÀI HỌC VIẾT SẴN:
#
#   ① Bản cũ dựng Tác phẩm bằng `key 18` — một phím do CHÍNH `bench.js` đăng ký.
#      `kill-campaign-v2.sh:70-72` đã cố ý bỏ đường đó ở v3 vì nó *"phụ thuộc thẳng vào bàn đo
#      tiêm, đúng cái đang bị chặn"*. Nó còn kéo theo một `location.reload()` (`bench.js:255`),
#      và sau lượt nạp lại app nằm ở màn **Library** — nơi KHÔNG có lưới nào để bấm.
#      ⇒ Dựng qua GIAO DIỆN THẬT bằng `setup-gui.sh`, như bộ kill đã làm 121 lượt.
#
#   ② Bản cũ KHÔNG mở tab Workspace, và KHÔNG chuẩn hoá hình học cửa sổ. Mọi toạ độ của
#      `focus-segment.sh` là TƯƠNG ĐỐI gốc cửa sổ `{200,25}` mà `setup-gui.sh` đặt-rồi-đọc-lại.
#      Không có bước đó thì 16 ứng viên trỏ vào một hệ toạ độ không tồn tại.
#
# 🔴 Và một luật bị phá lặng lẽ: `front.sh` viết *"KHÔNG script nào được gửi phím đầu tiên
# trước khi `require_front` xanh"*. Bản cũ gửi `key 18` mà chưa qua cổng đó — đúng khe hở mà
# sự cố 2026-08-13 15:35 đã chui lọt (năm lượt phím đi vào Brave).
echo "== dựng Tác phẩm QUA GIAO DIỆN THẬT (setup-gui.sh) =="
"$SCRATCH/setup-gui.sh" "$PID" /tmp/at-bench-chapter.txt \
  || { echo "🔴 DỪNG — setup-gui đỏ (cổng tiêu điểm hoặc hình học cửa sổ)"; kill -9 $PID 2>/dev/null; exit 1; }
for i in $(seq 1 120); do [ -n "$(find "$DRAFT/Documents/AuraTranslate" -name project.db 2>/dev/null | head -1)" ] && break; sleep 0.5; done
DB=$(find "$DRAFT/Documents/AuraTranslate" -name project.db 2>/dev/null | head -1)
[ -z "$DB" ] && { echo "🔴 không dựng được Tác phẩm"; kill -9 $PID 2>/dev/null; exit 1; }
sleep 4
echo "   segment: $(sqlite3 -readonly "$DB" 'select count(*) from segment;' 2>/dev/null)"

echo "== mở tab Workspace (lưới phải có mặt TRƯỚC khi đo hay đặt con trỏ) =="
require_front "$PID" 6 >/dev/null \
  || { echo "🔴 DỪNG — mất tiêu điểm trước khi mở Workspace"; kill -9 $PID 2>/dev/null; exit 1; }

# 🔴 DÙNG PHÍM TẮT CỦA CHÍNH SẢN PHẨM, BỎ HẲN TOẠ ĐỘ — sửa 2026-08-19 (lượt thứ hai).
#
# Lượt sửa trước thay toạ độ ĐOÁN bằng toạ độ ĐO của tab, và nó vẫn trượt: ảnh chụp
# `fail-tab-retry.png` cho thấy app đứng nguyên ở Library sau cú bấm, dù toạ độ tính ra
# `(302,73)` khớp ảnh chụp trong ~2 điểm. ⇒ Vấn đề không nằm ở CON SỐ mà ở việc dùng chuột
# cho một việc mà sản phẩm đã có đường bàn phím.
#
# `commands/index.ts:549-566` đăng ký `mode.workspace` với `keys: ['Mod+2']` và `run` gọi
# `setMode(mode)` **vô điều kiện** — không một cửa chặn nào. Đây là đường của chính sản phẩm,
# không phải một lối tắt của bàn đo, nên nó cũng KHÔNG vi phạm Quyết định #2 *(bơm phím ở
# tầng hệ điều hành)*.
#
# ⚠️ `Mod` = ⌘ trên macOS — xem §Trap 1 của `src/commands/keys.ts`. Mã phím 19 = Digit2.
# Không đụng họ phím của bench *(Ctrl+Alt+Shift+…)*.
#
# 🔴 Và NGHIỆM THU bằng kho, không bằng cú gửi phím: hỏi `__cellrect__` tới khi lưới có mặt.
# `finishSubmit` gọi `ensureChapterLoaded`/`ensureSegmentsLoaded` bất đồng bộ, nên 122 ô có
# thể chưa dựng xong ở mili-giây đầu — chờ có mốc dừng thật, không `sleep` mù.
CELLXY=""
for att in 1 2 3 4 5 6; do
  require_front "$PID" 6 >/dev/null \
    || { echo "🔴 DỪNG — mất tiêu điểm trước khi sang Workspace"; kill -9 $PID 2>/dev/null; exit 1; }
  osascript -e 'tell application "System Events" to key code 19 using {command down}' >/dev/null 2>&1
  sleep 2
  key 26; sleep 1.5                      # bench ghi `__cellrect__`
  CELLXY=$(sqlite3 -readonly "$APPDATA/global.db" "select value from config_value where key='__cellrect__';" 2>/dev/null)
  [ -n "${CELLXY:-}" ] && [ "$CELLXY" != "NONE" ] && break
  echo "   ⚠️ lượt $att: chưa thấy lưới (\"${CELLXY:-rỗng}\") — gửi lại ⌘2"
  CELLXY=""
done
if [ -z "$CELLXY" ]; then
  echo "🔴 DỪNG — sau 6 lượt ⌘2 vẫn KHÔNG có ô .cell-tgt nào: chưa sang được Workspace."
  screencapture -o -R 200,25,1200,900 "$SCRATCH/fail-tab-$LABEL.png" 2>/dev/null \
    && echo "   ảnh màn hình lúc trượt: $SCRATCH/fail-tab-$LABEL.png"
  kill -9 $PID 2>/dev/null; exit 1
fi
echo "   🟢 đã sang Workspace — lưới có $(print -r -- "$CELLXY" | cut -d, -f5) ô"
require_front "$PID" 6 >/dev/null \
  || { echo "🔴 DỪNG — mất tiêu điểm sau khi mở Workspace"; kill -9 $PID 2>/dev/null; exit 1; }

# 🔴 AC12 chạy SAU lượt mở Workspace: nó đo trên cây DOM của LƯỚI (`hotpaths()` hỏi
# `document.querySelector('.grid')`). Gọi ở màn Library là đo RỖNG — `bench.js` §⑤ đã ghi rõ
# đó đúng lớp lỗi "xanh rỗng" mà AC21 cấm.
#
# 🔴 AC13 KHÔNG còn ở đây — nó đã dời xuống SAU phiên gõ, và đây là lý do đo được.
# `measureRebuild()` (`bench.js` §⑥) làm `grid.innerHTML = ''` rồi gán lại. Nó PHÁ cây DOM
# thật của lưới và dựng lại từ HTML thô: Vue giữ tham chiếu tới các node CŨ nay đã tháo rời,
# còn node mới là HTML CHẾT — không handler, không `contenteditable` được nối. Sau lượt đó,
# bấm vào lưới không có gì xảy ra và gõ không đi đâu cả.
# ⇒ Bản cũ gọi AC13 TRƯỚC phiên gõ, nên phép đo AC13 GIẾT chính phiên NFR2 đứng sau nó.
# Triệu chứng: `focus-segment.sh` trượt cả 16 ứng viên, hai lượt liền, trên cả hai cỡ thang.
# ⚠️ `hotpaths()` thì đã kiểm: nó KHÔNG ghi vào DOM, nên nó ở lại đây được.
echo "== AC12: đo ba ĐƯỜNG NÓNG =="
key 21; sleep 6

# 🔴 THỨ TỰ Ở ĐÂY LÀ CÓ LÝ DO — đặt con trỏ TRƯỚC, bật lấy mẫu SAU.
# `focus-segment.sh` gõ một ký tự dò rồi chờ 4,5 s, xoá rồi chờ 4,5 s, và nó thử tới 16 ứng
# viên ⇒ tối đa ~2,4 phút thao tác bàn phím thật. Bật lấy mẫu trước lượt dò thì toàn bộ giai
# đoạn đó nằm TRONG mẫu NFR2 — mà nó không phải một phiên gõ, nó là bàn đo tự dò đường.
# Trộn vào là làm bẩn đúng cái đuôi phân bố mà AC11 hỏi.
if [ "$MODE" = "typing" ]; then
  echo "== đặt con trỏ vào một câu, và NGHIỆM THU rằng nó đã vào =="
  # `BENCH_APPDATA` mở đường ĐO toạ độ của `focus-segment.sh` (§⑦b của bench.js): nó cần
  # `global.db` để đọc `__cellrect__`. Không đặt biến này thì tệp kia lặng lẽ rơi về bộ 16
  # ứng viên đoán — vẫn chạy, nhưng chạy ở chế độ chập chờn 3/7 mà lượt sửa này đang bỏ.
  HIT=$(BENCH_APPDATA="$APPDATA" "$SCRATCH/focus-segment.sh" "$PID" "$DB") || {
    echo "🔴 DỪNG — không đặt được con trỏ vào một câu nào trong 16 ứng viên."
    echo "   Chạy tiếp là sản xuất một bảng số của một phiên KHÔNG AI GÕ (NFR2 xanh giả)."
    # 🔴 CHỤP TRƯỚC KHI GIẾT — cùng lý do như hàng rào đổ số. Một lượt trượt con trỏ có hai
    # nguyên nhân KHÁC HẲN NHAU mà cùng một triệu chứng: ⑴ đang ở màn Library vì cú bấm mở
    # tab Workspace trượt ⇒ không có lưới nào để bấm; ⑵ đang ở lưới nhưng toạ độ sai.
    # Chỉ một ảnh chụp phân biệt được hai ca, và không có nó thì lượt chẩn đoán tiếp theo là
    # một lượt đoán.
    screencapture -o -R 200,25,1200,900 "$SCRATCH/fail-focus-$LABEL.png" 2>/dev/null \
      && echo "   ảnh màn hình lúc trượt: $SCRATCH/fail-focus-$LABEL.png"
    kill -9 $PID 2>/dev/null; exit 1
  }
  echo "   🟢 con trỏ đã vào, toạ độ ăn: $HIT"
fi

echo "== bắt đầu lấy mẫu frame =="
key 19; sleep 1

if [ "$MODE" = "typing" ]; then
  echo "== gõ ${DUR}s =="
  "$SCRATCH/type-driver.sh" "$DUR" "$SCRATCH/typing-$LABEL.log" >/dev/null 2>&1
else
  echo "== ĐỐI CHỨNG: bàn đo chạy ${DUR}s, KHÔNG gõ (AC21) =="
  perl -e "select(undef,undef,undef,$DUR)"
fi

# 🔴 AC13 CHẠY Ở ĐÂY — sau phiên gõ, trước lượt đổ số. Nó PHÁ cây DOM của lưới (xem chú thích
# ở khúc AC12), nên mọi thứ cần một lưới sống phải đã xong. `dump()` đọc `B.build` nên AC13
# vẫn phải đứng TRƯỚC `key 20`.
# ⚠️ GIỚI HẠN THẬT: lúc này bộ lấy mẫu VẪN CHẠY (`dump()` mới là chỗ gọi `stop()`, và không có
# phím "ngừng" riêng). Nên frame của lượt dựng lại RƠI VÀO `all`. Nó KHÔNG rơi vào `busy` —
# `summarize()` chỉ giữ mẫu có `input` hoặc `flush` trong cửa sổ, mà lượt dựng lại không sinh
# cái nào. `busy` là số nghiệm thu AC1, nên phán quyết AC1 không bị đụng. 🔴 Nhưng ai trích cột
# `all` phải biết nó chở thêm một frame của AC13 — đừng đọc `all.max` như một số của phiên gõ.
# 🔴 CỔNG TIÊU ĐIỂM TRƯỚC MỖI PHÍM CUỐI — thêm 2026-08-19 sau một lượt mất số.
# Đo được: phiên `flushtest` gõ đủ 60 s (48 dòng trong `typing-flushtest.log`) rồi đổ ra
# **0 byte**. Kho lúc đó có `__bench_alive__`, `mode`, `workspace_layout` — tức `put_config`
# vẫn chạy suốt phiên; thứ không tới nơi là `key 20`. Bản cũ bắn hai phím cuối mà KHÔNG kiểm
# lại tiêu điểm, dù `front.sh` đã viết thành luật: *"KHÔNG script nào được gửi phím đầu tiên
# trước khi `require_front` xanh"*. Một phiên 30 phút mất ở đúng bước cuối là 30 phút bỏ đi.
echo "== chờ app nhận phím trở lại sau phiên gõ =="
if KN=$(settle_keys 90); then
  echo "   🟢 app đang nhận phím (bộ đếm phím nóng = $KN)"
else
  echo "🔴 DỪNG — app KHÔNG nhận phím trong 90 s sau phiên gõ; số của phiên này KHÔNG lấy được"
  screencapture -o -R 200,25,1200,900 "$SCRATCH/fail-settle-$LABEL.png" 2>/dev/null \
    && echo "   ảnh màn hình lúc trượt: $SCRATCH/fail-settle-$LABEL.png"
  kill -9 $PID 2>/dev/null; exit 1
fi

if [ "$MODE" = "typing" ]; then
  echo "== AC13: đo lượt DỰNG lại cả Chương (PHÁ DOM — nên nó ở cuối) =="
  require_front "$PID" 6 >/dev/null \
    || { echo "🔴 DỪNG — mất tiêu điểm trước AC13; số của phiên này KHÔNG lấy được"; kill -9 $PID 2>/dev/null; exit 1; }
  key 23; sleep 3
fi


# 🔴 BẮN RỒI ĐỌC LẠI, TỚI KHI SỐ VỀ — sửa 2026-08-19.
# Đo được: nhật ký phím của bench *(ô `#__bench__`, đọc qua ảnh chụp)* ghi `⌨[7,7,4,7,2]` ở
# một phiên hỏng — tức phím 7·4·2 tới nơi còn **5 (AC13) và 3 (đổ số) thì không**, dù cổng
# tiêu điểm xanh và vòng rAF của bench vẫn chạy *(ô đó còn cập nhật)*. Nên luồng chính KHÔNG
# nghẽn; đây là chuyện GIAO PHÍM.
# ⚠️ Nguyên nhân chưa định vị được. Nhưng "bắn một phát rồi tin" là thứ chắc chắn sai: một
# phiên 30 phút mất ở bước cuối là 30 phút bỏ đi. ⇒ Bắn, ĐỌC KHO, chưa có thì bắn lại.
# 🔴 Đây là hàng rào, KHÔNG phải bản vá: nếu nó phải thử tới lượt thứ hai trở lên thì con số
# đó tự nó là một phát hiện — in ra, đừng nuốt.
echo "== ngừng + đổ số =="
DUMPED=""
for att in 1 2 3 4 5; do
  require_front "$PID" 6 >/dev/null \
    || { echo "🔴 DỪNG — mất tiêu điểm trước lượt đổ số; số của phiên này KHÔNG lấy được"; kill -9 $PID 2>/dev/null; exit 1; }
  key 20
  sleep 3
  DUMPED=$(sqlite3 -readonly "$APPDATA/global.db" "select value from config_value where key='__bench__';" 2>/dev/null)
  if [ -n "$DUMPED" ]; then
    [ "$att" -gt 1 ] && echo "   ⚠️ phím đổ số phải bắn $att lượt mới tới nơi — GHI RA, đây là một phát hiện"
    break
  fi
  echo "   ⚠️ lượt $att: phím đổ số chưa tới, bắn lại"
done

sqlite3 "$APPDATA/global.db" "select value from config_value where key='__bench__';" > "$SCRATCH/nfr2-$LABEL.json" 2>/dev/null
NBYTE=$(wc -c < "$SCRATCH/nfr2-$LABEL.json" | tr -d ' ')
echo "   → $SCRATCH/nfr2-$LABEL.json  ($NBYTE byte)"
# 🔴 RỖNG LÀ ĐỎ, KHÔNG PHẢI MỘT DÒNG THÔNG BÁO. Bản cũ in "0 byte" rồi thoát 0 — một phiên
# mất trắng đọc lên giống hệt một phiên thành công. Đúng lớp lỗi trung tâm của dự án.
if [ "${NBYTE:-0}" -lt 2 ]; then
  echo "🔴 PHIÊN HỎNG — không có số nào được đổ ra. Các khoá thật sự có trong kho:"
  sqlite3 -readonly "$APPDATA/global.db" "select kind||'/'||key||'  ('||length(value)||' byte)' from config_value;" 2>/dev/null | sed 's/^/     /'
  # 🔴 CHỤP TRƯỚC KHI GIẾT. `bench.js` ghi lỗi của `dump()` ra ô `#__bench__` bằng
  # `paint('ĐỔ TRƯỢT: ' + e)` — đó là kênh DUY NHẤT nói được vì sao lượt đổ số hỏng, và nó
  # chết cùng tiến trình. Hai lượt chẩn đoán trước đã mất đúng thông tin này vì hàng rào
  # giết app trước khi đọc.
  screencapture -o -R 200,25,1200,900 "$SCRATCH/fail-$LABEL.png" 2>/dev/null \
    && echo "     ảnh màn hình lúc hỏng: $SCRATCH/fail-$LABEL.png"
  kill -9 $PID 2>/dev/null; exit 1
fi
printf 'wal_project=%s wal_global=%s\n' \
  "$(stat -f '%z' "$DB-wal" 2>/dev/null || echo 0)" \
  "$(stat -f '%z' "$APPDATA/global.db-wal" 2>/dev/null || echo 0)"

snap_env "SAU phien $LABEL"
echo "   trạng thái máy (AC22): $ENVF"

kill -9 $PID 2>/dev/null
echo "═══ xong ═══"
