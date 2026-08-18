#!/bin/zsh
# Dựng bản ĐO của Story 2.4 — Quyết định #1 sau ĐÍNH CHÍNH.
#
# 🔴 Rust: release NGUYÊN VẸN. Không đổi Cargo.toml, không `wdio`, không `debug-assertions`.
# Frontend: bản Vite production, CỘNG `bench.js` — một script CỔ ĐIỂN, cùng origin
# (`script-src 'self'` của CSP cho phép; một script NỘI TUYẾN thì không).
#
# `--config` vô hiệu hoá `beforeBuildCommand` để `tauri build` KHÔNG dựng lại `dist/`
# và xoá mất lượt tiêm.
set -u
REPO=/Users/hoangnam/LocalSites/addon/AuraTranslate
SCRATCH="${0:A:h}"
cd "$REPO"

echo "== ① dist sạch =="
npm run build >/dev/null 2>&1
test -f dist/index.html

echo "== ② tiêm bench vào CHÍNH bundle của app =="
# 🔴 Đo được 2026-08-13: một tệp RIÊNG `dist/bench.js` + một thẻ <script src> KHÔNG chạy —
# nhị phân đã nhúng đúng tệp và thẻ có mặt, nhưng CSP của sản phẩm
# (`script-src 'self'`, tauri.conf.json) không cho nó thực thi. Đây là một dữ kiện về sản
# phẩm, không một trục trặc của bàn đo: CSP đang làm đúng việc của nó.
#
# ⇒ Tiêm vào ĐUÔI bundle mà app vốn đã nạp. Không thêm một nguồn script nào ⇒ không cửa
# CSP nào phải qua. Vá `__TAURI_INTERNALS__.invoke` vẫn chạy được vì `@tauri-apps/api`
# tra cái global đó LÚC GỌI, không giữ tham chiếu lúc nạp.
python3 - <<'PY'
import glob, io
b = glob.glob('dist/assets/index-*.js')
assert len(b) == 1, b
src = io.open(b[0], encoding='utf-8').read()
assert '__bench_alive__' not in src, 'đã tiêm rồi'
# 🔵 Sửa 2026-08-18: đường dẫn cũ trỏ vào scratchpad của phiên 2026-08-13, thư mục đó đã bị
# dọn. Một tạo tác đã lưu vào kho mà còn trỏ ra NGOÀI kho là một tạo tác không dựng lại được
# — đúng cái mà lượt lưu bàn đo vào kho tồn tại để chống. Trỏ vào chính kho.
bench = io.open('_bmad-output/implementation-artifacts/2-4-ban-do/bench.js', encoding='utf-8').read()
io.open(b[0], 'w', encoding='utf-8').write(src + '\n;/* ── BÀN ĐO STORY 2.4 — KHÔNG VÀO KHO ── */\n' + bench)
print('   đã tiêm vào', b[0])
PY

echo "== ③ cargo release (KHÔNG dựng lại dist) =="
# 🔴 ÉP dựng lại. Đo được 2026-08-13: một lượt đổi CHỈ trong `dist/` KHÔNG làm cargo dựng
# lại, nên nhị phân giữ nguyên bộ tài nguyên nhúng CŨ và lượt tiêm biến mất im lặng —
# đúng lớp lỗi "xanh rỗng". `touch` một tệp nguồn Rust là chỗ lật rẻ nhất.
# 🔴 CHẨN ĐOÁN 2026-08-13: `build.rs:66` khai `cargo:rerun-if-changed=windows-app-manifest.xml`.
# Một khi build script phát BẤT KỲ `rerun-if-changed` nào, cargo chỉ theo dõi ĐÚNG danh sách
# đó ⇒ `build.rs` — chỗ `tauri-build` NHÚNG `dist/` — không bao giờ chạy lại khi `dist/` đổi.
# `touch lib.rs` liên kết lại nhị phân nhưng giữ bộ tài nguyên nhúng CŨ. Phải chạm ĐÚNG tệp
# mà build.rs khai là đầu vào.
touch src-tauri/windows-app-manifest.xml src-tauri/build.rs
npx tauri build --bundles app --config '{"build":{"beforeBuildCommand":""}}' 2>&1 | tail -5

echo "== ④ nghiệm thu lượt tiêm — BẰNG MÁY, không bằng mắt =="
APP="$REPO/src-tauri/target/release/bundle/macos/AuraTranslate.app"
test -d "$APP" || { echo "   🔴 không có .app"; exit 1; }
# ⚠️ `strings` là cổng ÂM TÍNH GIẢ ở đây (`strip = true` + tài nguyên nhúng bị nén), và nó
# đã báo đỏ oan một lượt. Cổng thật: chạy app trong HOME nháp và hỏi `global.db` xem bench
# có ghi được DẤU SỐNG không.
PROBE="$SCRATCH/alive-home"; rm -rf "$PROBE"; mkdir -p "$PROBE/Documents"
HOME="$PROBE" "$APP/Contents/MacOS/auratranslate" >/dev/null 2>&1 &
PID=$!
DB="$PROBE/Library/Application Support/com.auratranslate.desktop/global.db"
ALIVE=""
for i in $(seq 1 60); do
  sleep 0.5
  [ -f "$DB" ] || continue
  ALIVE=$(sqlite3 "$DB" "select value from config_value where kind='app_config' and key='__bench_alive__';" 2>/dev/null) || true
  [ -n "$ALIVE" ] && break
done
PID2=$PID
TITLE=$(osascript -e 'tell application "System Events" to get title of front window of (first process whose name is "AuraTranslate")' 2>/dev/null)
kill -9 $PID2 2>/dev/null
echo "   tiêu đề cửa sổ = ${TITLE:-（không đọc được）}"
if [ -n "$ALIVE" ]; then
  echo "   🟢 bench ĐÃ chạy trong webview của bản release · dấu sống=$ALIVE"
else
  echo "   🔴 dấu sống VẮNG — đọc tiêu đề ở trên để biết mã có chạy không"; exit 1
fi
