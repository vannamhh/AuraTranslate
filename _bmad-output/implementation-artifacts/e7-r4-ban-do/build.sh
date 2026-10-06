#!/bin/zsh
[ -n "${ZSH_VERSION:-}" ] || exec zsh "$0" "$@"
# Builds the packaged release app with feature nfr-bench and appends probe.js to the one
# production bundle. No dictionary layers are bundled: tauri.conf.json ships fonts/ and license/ only.
set -euo pipefail

SCRIPT_DIR="${0:A:h}"
REPO="${SCRIPT_DIR:h:h:h}"
PROBE="$SCRIPT_DIR/probe.js"
cd "$REPO"

[[ -f "$PROBE" ]] || { print -u2 "missing $PROBE"; exit 1; }

print '== build frontend production =='
npm run build

print '== append the probe to the single production bundle =='
node - "$PROBE" <<'NODE'
const fs = require('node:fs')
const path = require('node:path')
const probePath = process.argv[2]
const assets = path.join(process.cwd(), 'dist', 'assets')
const bundles = fs.readdirSync(assets).filter((name) => /^index-.*\.js$/.test(name))
if (bundles.length !== 1) throw new Error(`need exactly one index bundle, got ${JSON.stringify(bundles)}`)
const bundle = path.join(assets, bundles[0])
const source = fs.readFileSync(bundle, 'utf8')
if (source.includes('__nfr_bench_alive_e7_r4__')) throw new Error('bundle already carries the e7-r4 probe')
fs.writeFileSync(bundle, `${source}\n;/* E7 R-4 BENCH PROBE, NOT PRODUCT CODE */\n${fs.readFileSync(probePath, 'utf8')}\n`)
process.stdout.write(`appended ${probePath} to ${bundle}\n`)
NODE

# build.rs tracks only its own rerun-if-changed list; touching two declared inputs forces a dist re-embed.
touch src-tauri/windows-app-manifest.xml src-tauri/build.rs

print '== build Tauri release (feature nfr-bench) =='
npx tauri build --bundles app --features nfr-bench --config '{"build":{"beforeBuildCommand":""}}'

APP="$REPO/src-tauri/target/release/bundle/macos/AuraTranslate.app"
[[ -x "$APP/Contents/MacOS/auratranslate" ]] || { print -u2 "no release app at $APP"; exit 1; }
print "APP_RELEASE=$APP"
