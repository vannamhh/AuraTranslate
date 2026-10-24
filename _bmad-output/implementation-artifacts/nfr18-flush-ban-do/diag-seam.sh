#!/bin/zsh
# ══════════════════════════════════════════════════════════════════════════════════════
# VÒNG CHẨN ĐOÁN ③ — "bench.js không đóng được dấu sống", 2026-08-19
# ══════════════════════════════════════════════════════════════════════════════════════
# 🔴 ĐÂY LÀ VÒNG CUỐI mà luật dừng của Task 0 cho phép. Hai vòng trước đã bị bác:
#   ① "put_config ném, lỗi hiện ở log"      → log sạch, 0 dòng lỗi
#   ② "đoạn tiêm hỏng cú pháp ⇒ Vue chết"   → ảnh chụp cho thấy app render đầy đủ
# Vòng này KHÔNG hỏi thêm một câu; nó hỏi BA câu trong CÙNG một lượt dựng, vì một lượt
# dựng tốn ~10 phút và luật dừng không cho tôi ba lượt nữa.
#
# ── BA TÍN HIỆU, ĐỘC LẬP NHAU ─────────────────────────────────────────────────────────
# ⓐ ĐỔI TÊN bundle thành `index-SEAMPROBE.js`.
#    Tên tài nguyên nằm KHÔNG NÉN trong nhị phân (đã đo: `strings` tìm thấy
#    `index-8nGCpVMZ.js`), trong khi NỘI DUNG thì nén — nên tên là kênh duy nhất đọc được
#    từ ngoài. 🔴 Vì sao phép thử cũ vô nghĩa: `npm run build` băm nội dung TRƯỚC lượt tiêm,
#    nên tên trước và sau khi tiêm TRÙNG NHAU. Một bộ tài nguyên nhúng cũ vẫn mang đúng cái
#    tên đó. Đổi tên là cách duy nhất tách được "nhúng mới" khỏi "nhúng cũ".
#      thấy SEAMPROBE  ⇒ nhúng MỚI, đoạn tiêm CÓ trong nhị phân
#      không thấy      ⇒ nhúng CŨ — đó là gốc rễ, và nó lật lượt "bác" ngày 2026-08-13
#
# ⓑ MỘT lời gọi `put_config` đặt NGAY SAU `Uw();`, ngoài mọi IIFE, khoá `__seam__`.
#    Nó không phụ thuộc một dòng nào của `bench.js`. Trả lời: mã nối thêm có CHẠY không.
#
# ⓒ Dấu sống `__bench_alive__` của chính `bench.js` — giữ nguyên, không sửa.
#    Trả lời: thân `bench.js` có chạy tới cuối không.
#
# ── BẢNG ĐỌC KẾT QUẢ ──────────────────────────────────────────────────────────────────
#   ⓐ✗                 ⇒ tài nguyên nhúng CŨ. Gốc rễ. Không phải lỗi của `bench.js`.
#   ⓐ✓ ⓑ✗             ⇒ mã nối thêm KHÔNG chạy dù đã nhúng — nghi `Uw()` nuốt phần đuôi.
#   ⓐ✓ ⓑ✓ ⓒ✗         ⇒ thân `bench.js` ném ở giữa. Chia đôi để tìm, KHÔNG đoán.
#   ⓐ✓ ⓑ✓ ⓒ✓         ⇒ đã lành; lượt nghiệm thu cũ hỏng chứ không phải bench.
#
# ⚠️ GIỚI HẠN THẬT: tệp này KHÔNG dựng bản đo dùng được cho phiên NFR2 — nó đổi tên
# bundle, nên nó là một bản CHẨN ĐOÁN. Sau khi đọc xong phải chạy lại `build-bench.sh`.
set -u

R=/Users/hoangnam/LocalSites/addon/AuraTranslate
SCRATCH="${0:A:h}"
cd "$R"

say() { print -r -- "$@" }
die() { print -r -- "🔴 $@" >&2; exit 1 }

say "══ ① dist sạch ══"
npm run build >/dev/null 2>&1 || die "npm run build ĐỎ"
test -f dist/index.html || die "không có dist/index.html"

say "══ ② tiêm ⓑ dấu mối nối + ⓒ bench.js, rồi ⓐ đổi tên bundle ══"
python3 - <<'PY' || exit 1
import glob, io, os, sys
b = glob.glob('dist/assets/index-*.js')
if len(b) != 1:
    sys.exit('🔴 mong đúng một bundle, thấy %r' % b)
old = b[0]
src = io.open(old, encoding='utf-8').read()
if '__seam__' in src or '__bench_alive__' in src:
    sys.exit('🔴 đã tiêm rồi — chạy npm run build lại')

# ⓑ đứng TRƯỚC bench.js và không dùng một hàm nào của nó. Tự thử lại vì kho có thể chưa
# được `manage` ở mili-giây đầu — đúng giả thuyết mà chính bench.js gọi là "hàng đầu".
seam = """
;(function seam(n){
  var I = window.__TAURI_INTERNALS__
  if (I && typeof I.invoke === 'function') {
    I.invoke('put_config', {kind:'app_config', key:'__seam__', value:String(n)})
      .catch(function(){ if (n < 240) setTimeout(function(){ seam(n+1) }, 250) })
    return
  }
  if (n < 240) setTimeout(function(){ seam(n+1) }, 250)
})(0)
"""
bench = io.open('_bmad-output/implementation-artifacts/2-4-ban-do/bench.js',
                encoding='utf-8').read()
io.open(old, 'w', encoding='utf-8').write(
    src + '\n;/* ── ⓑ DẤU MỐI NỐI ── */' + seam +
          '\n;/* ── ⓒ BÀN ĐO STORY 2.4 ── */\n' + bench)

# ⓐ đổi tên, và sửa cả chỗ tham chiếu trong index.html
new = os.path.join(os.path.dirname(old), 'index-SEAMPROBE.js')
os.rename(old, new)
html_path = 'dist/index.html'
html = io.open(html_path, encoding='utf-8').read()
if os.path.basename(old) not in html:
    sys.exit('🔴 index.html không trỏ tới %s' % os.path.basename(old))
io.open(html_path, 'w', encoding='utf-8').write(
    html.replace(os.path.basename(old), 'index-SEAMPROBE.js'))
print('   ⓐ đổi tên:', os.path.basename(old), '→ index-SEAMPROBE.js')
print('   ⓑ+ⓒ đã tiêm')
PY

say "══ ③ cargo release (KHÔNG dựng lại dist) ══"
touch src-tauri/windows-app-manifest.xml src-tauri/build.rs
npx tauri build --bundles app --config '{"build":{"beforeBuildCommand":""}}' \
  > "$SCRATCH/build-diag-seam.log" 2>&1 \
  || { tail -20 "$SCRATCH/build-diag-seam.log"; die "dựng ĐỎ"; }

BIN="$R/src-tauri/target/release/bundle/macos/AuraTranslate.app/Contents/MacOS/auratranslate"
say "   nhị phân: $(stat -f '%z B  %Sm' -t '%H:%M:%S' "$BIN")"

say ""
say "══ ⓐ TÊN TÀI NGUYÊN NHÚNG ══"
NAMES=$(strings -a "$BIN" | grep -oE 'index-[A-Za-z0-9_-]+\.js' | sort -u)
print -r -- "$NAMES" | sed 's/^/   /'
if print -r -- "$NAMES" | grep -q 'index-SEAMPROBE.js'; then
  A=1; say "   🟢 ⓐ ĐẠT — nhúng MỚI, đoạn tiêm có trong nhị phân"
else
  A=0; say "   🔴 ⓐ TRƯỢT — nhúng CŨ. Đây là gốc rễ."
fi

say ""
say "══ chạy app trong HOME nháp, chờ 25 s ══"
P=/tmp/at-seam-home; rm -rf $P; mkdir -p "$P/Documents"
HOME="$P" "$BIN" > "$SCRATCH/app-diag-seam.log" 2>&1 &
PID=$!
DB="$P/Library/Application Support/com.auratranslate.desktop/global.db"
for i in $(seq 1 60); do sleep 0.5; [ -f "$DB" ] && break; done
sleep 25
SEAM=$(sqlite3 "$DB" "select value from config_value where key='__seam__';" 2>/dev/null)
ALIVE=$(sqlite3 "$DB" "select value from config_value where key='__bench_alive__';" 2>/dev/null)
ROWS=$(sqlite3 "$DB" "select kind||'/'||key from config_value;" 2>/dev/null | tr '\n' ' ')
kill -9 $PID 2>/dev/null

say ""
say "══ ⓑ DẤU MỐI NỐI ══";  [ -n "$SEAM" ]  && say "   🟢 ⓑ ĐẠT — mã nối thêm CHẠY (n=$SEAM)" || say "   🔴 ⓑ TRƯỢT — mã nối thêm KHÔNG chạy"
say "══ ⓒ DẤU SỐNG BENCH ══"; [ -n "$ALIVE" ] && say "   🟢 ⓒ ĐẠT — bench.js chạy tới cuối ($ALIVE)" || say "   🔴 ⓒ TRƯỢT — bench.js không tới được dấu sống"
say ""
say "   mọi hàng config_value: ${ROWS:-（rỗng）}"
say ""
say "══ PHÁN QUYẾT ══"
if [ "$A" = "0" ];                              then say "   ⇒ tài nguyên nhúng CŨ. Gốc rễ nằm ở khâu nhúng, KHÔNG ở bench.js."
elif [ -z "$SEAM" ];                            then say "   ⇒ đã nhúng nhưng mã nối thêm KHÔNG chạy — nghi phần đuôi module bị nuốt."
elif [ -z "$ALIVE" ];                           then say "   ⇒ mối nối sống, thân bench.js chết ở giữa. Chia đôi để tìm, ĐỪNG đoán."
else                                                 say "   ⇒ cả ba ĐẠT — lượt nghiệm thu cũ hỏng, không phải bench."
fi
say ""
say "🔴 NHỚ: bản này ĐỔI TÊN bundle nên nó là bản CHẨN ĐOÁN. Chạy lại ./build-bench.sh trước phiên đo."
