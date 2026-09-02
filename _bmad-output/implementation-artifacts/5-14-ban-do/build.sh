#!/bin/zsh
# Dựng app release có probe Story 5.14 và đúng một command điều phối được gác bằng feature.
# `beforeBuildCommand` bị vô hiệu để Tauri không xoá lượt nối probe sau khi frontend build.
set -euo pipefail

SCRIPT_DIR="${0:A:h}"
REPO="${SCRIPT_DIR:h:h:h}"
PROBE="$SCRIPT_DIR/probe.js"
cd "$REPO"

[[ -f "$PROBE" ]] || { print -u2 "thiếu $PROBE"; exit 1; }

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
if (source.includes('__5_14_alive__')) throw new Error('bundle đã có probe Story 5.14')
const probe = fs.readFileSync(probePath, 'utf8')
fs.writeFileSync(bundle, `${source}\n;/* BÀN ĐO STORY 5.14 — KHÔNG VÀO MÃ SẢN PHẨM */\n${probe}\n`)
process.stdout.write(`đã nối ${probePath} vào ${bundle}\n`)
NODE

# `build.rs` chỉ theo dõi danh sách rerun-if-changed của chính nó. Chạm đúng hai đầu vào đã
# khai buộc tauri-build nhúng lại dist; mtime không tạo diff Git.
touch src-tauri/windows-app-manifest.xml src-tauri/build.rs

print '== build Tauri release =='
# Feature rỗng chỉ thêm command đọc tệp pha có marker dưới `/tmp`; bản dựng mặc định không
# có command này. Không nới CSP/ATS và không mở một cổng loopback trong app đo.
npx tauri build --bundles app --features story-5-14-bench --config '{"build":{"beforeBuildCommand":""}}'

APP="$REPO/src-tauri/target/release/bundle/macos/AuraTranslate.app"
[[ -x "$APP/Contents/MacOS/auratranslate" ]] || { print -u2 "không có app release ở $APP"; exit 1; }
print "APP_RELEASE=$APP"
