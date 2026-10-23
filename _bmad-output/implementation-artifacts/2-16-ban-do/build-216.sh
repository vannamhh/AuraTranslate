#!/bin/zsh
# Usage: build-216.sh <repo-root> <out-dir>   Release build of <repo-root> with bench-216.js appended to its bundle.
set -eu
REPO="${1:?repo}"; OUT="${2:?out}"
BENCH="${0:A:h}/bench-216.js"
export CARGO_TARGET_DIR=/Users/hoangnam/LocalSites/addon/AuraTranslate/src-tauri/target
cd "$REPO"
npm run build >/dev/null 2>&1
BENCH="$BENCH" python3 - <<'PY'
import glob, io, os
b = glob.glob('dist/assets/index-*.js'); assert len(b) == 1, b
src = io.open(b[0], encoding='utf-8').read()
assert '__bench_alive__' not in src
io.open(b[0], 'w', encoding='utf-8').write(src + '\n;' + io.open(os.environ['BENCH'], encoding='utf-8').read())
PY
touch src-tauri/windows-app-manifest.xml src-tauri/build.rs
npx tauri build --bundles app --config '{"build":{"beforeBuildCommand":""}}' 2>&1 | tail -5
rm -rf "$OUT"; mkdir -p "$OUT"
cp -R "$CARGO_TARGET_DIR/release/bundle/macos/AuraTranslate.app" "$OUT/"
codesign --force --deep -s - "$OUT/AuraTranslate.app" 2>&1 | tail -1
git -C "$REPO" rev-parse HEAD > "$OUT/revision.txt"
git -C "$REPO" status --porcelain >> "$OUT/revision.txt"
