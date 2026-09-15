#!/bin/zsh
# Dựng app release Story 6.18: cùng probe production + command `nfr-bench` mà 5-14-ban-do
# dùng, CỘNG bốn tệp `resources/dict/*.db` thật qua một `--config` merge lúc build — điều
# 5-14-ban-do KHÔNG có (Epic 5 retro F5: NFR4/NFR5 đo trên 0 lớp từ điển "không phải một con
# số về sản phẩm"). `tauri.conf.json` KHÔNG bị sửa (Code Map: "bundle.resources -- fonts/
# license only, do not edit"); merge chỉ sống trong lệnh build này.
set -euo pipefail

SCRIPT_DIR="${0:A:h}"
REPO="${SCRIPT_DIR:h:h:h}"
PROBE="$SCRIPT_DIR/probe.js"
DICT_DIR="$REPO/src-tauri/resources/dict"
cd "$REPO"

[[ -f "$PROBE" ]] || { print -u2 "thiếu $PROBE"; exit 1; }

typeset -a dict_files
dict_files=("${(@f)$(find "$DICT_DIR" -maxdepth 1 -type f -iname '*.db' 2>/dev/null)}")
[[ "${#dict_files[@]}" -ge 1 ]] || {
  print -u2 "thiếu tệp *.db thật dưới $DICT_DIR -- Story 6.18 đo trên 0 lớp là ĐÚNG khuyết tật Epic 5 retro F5 muốn sửa, không phải một hàng rào để nới"
  exit 1
}
print "== ${#dict_files[@]} tệp *.db từ điển thật sẽ vào bundle: =="
for f in "${dict_files[@]}"; do print "  $f"; done

print '== build frontend production =='
npm run build

print '== nối probe vào đúng một bundle production =='
node - "$PROBE" <<'NODE'
const fs = require('node:fs')
const path = require('node:path')
const probePath = process.argv[2]
const assets = path.join(process.cwd(), 'dist', 'assets')
const bundles = fs.readdirSync(assets).filter((name) => /^index-.*\.js$/.test(name))
if (bundles.length !== 1) throw new Error(`cần đúng một bundle index, nhận ${JSON.stringify(bundles)}`)
const bundle = path.join(assets, bundles[0])
const source = fs.readFileSync(bundle, 'utf8')
if (source.includes('__nfr_bench_alive_6_18__')) throw new Error('bundle đã có probe Story 6.18')
const probe = fs.readFileSync(probePath, 'utf8')
fs.writeFileSync(bundle, `${source}\n;/* BÀN ĐO STORY 6.18 — KHÔNG VÀO MÃ SẢN PHẨM */\n${probe}\n`)
process.stdout.write(`đã nối ${probePath} vào ${bundle}\n`)
NODE

# `build.rs` chỉ theo dõi danh sách rerun-if-changed của chính nó. Chạm đúng hai đầu vào đã
# khai buộc tauri-build nhúng lại dist; mtime không tạo diff Git.
touch src-tauri/windows-app-manifest.xml src-tauri/build.rs

print '== build Tauri release (feature nfr-bench + lớp từ điển thật qua --config) =='
# `beforeBuildCommand` rỗng: dist production đã build ở trên, không cho Tauri chạy `npm run
# build` lần hai (nó sẽ xoá lượt nối probe). `bundle.resources` được MERGE thêm đúng một khoá
# `resources/dict/*.db` -> `dict/` -- khớp `DICT_RESOURCE_DIR = "dict"` mà
# `lib.rs::open_dict_layers` đọc qua `resource_dir()`. Không đụng CSP/`assetProtocol` (dict
# không đi qua asset protocol, chỉ đọc bằng Rust filesystem) và không đụng `tauri.conf.json`
# trên đĩa.
npx tauri build --bundles app --features nfr-bench --config \
  '{"build":{"beforeBuildCommand":""},"bundle":{"resources":{"resources/dict/*.db":"dict/"}}}'

APP="$REPO/src-tauri/target/release/bundle/macos/AuraTranslate.app"
[[ -x "$APP/Contents/MacOS/auratranslate" ]] || { print -u2 "không có app release ở $APP"; exit 1; }
BUNDLED_DICT="$APP/Contents/Resources/dict"
typeset -a bundled_files
bundled_files=("${(@f)$(find "$BUNDLED_DICT" -maxdepth 1 -type f -iname '*.db' 2>/dev/null)}")
[[ "${#bundled_files[@]}" -ge 1 ]] || {
  print -u2 "app release không mang lớp từ điển nào ở $BUNDLED_DICT -- --config merge thất bại im lặng"
  exit 1
}
print "== ${#bundled_files[@]} lớp từ điển đã vào bundle, tại $BUNDLED_DICT =="
print "APP_RELEASE=$APP"
