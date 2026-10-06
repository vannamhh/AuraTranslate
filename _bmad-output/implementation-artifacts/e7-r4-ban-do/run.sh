#!/bin/zsh
[ -n "${ZSH_VERSION:-}" ] || exec zsh "$0" "$@"
# One measurement run for the Epic 7 retro R-4: how long six TM-reading commands hold
# OpenWorkState in the packaged release app (build.sh output). Touches only a HOME marked
# auratranslate-nfr-bench-; the trap kills the app and deletes that HOME.
set -euo pipefail

SCRIPT_DIR="${0:A:h}"
REPO="${SCRIPT_DIR:h:h:h}"
APP="$REPO/src-tauri/target/release/bundle/macos/AuraTranslate.app"
APP_BIN="$APP/Contents/MacOS/auratranslate"
STARTED_AT="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
START_LOAD="$(uptime)"
WORK_NAME='E7 R4 Work'
RUN_LOG="$SCRIPT_DIR/latest-run.log"
RAW="$SCRIPT_DIR/raw-result.json"

exec > >(tee "$RUN_LOG") 2>&1

die() {
  print -u2 "ERROR: $*"
  if [[ -n "${SCRATCH:-}" && -f "$SCRATCH/app.log" ]]; then
    print -u2 '-- app.log tail --'
    tail -20 "$SCRATCH/app.log" >&2 || true
  fi
  exit 1
}

cd "$REPO"
# Idle/display sleep and App Nap-style throttling would stretch the held windows being measured.
caffeinate -dimsu -w $$ &
[[ -x "$APP_BIN" ]] || die "no release app at $APP; run build.sh first"

git diff --check
git diff --cached --check
typeset -a changed_paths
changed_paths=("${(@f)$( { git diff --name-only; git diff --cached --name-only; git ls-files --others --exclude-standard; } | LC_ALL=C sort -u )}")
for changed in "${changed_paths[@]}"; do
  [[ -z "$changed" ]] && continue
  case "$changed" in
    src-tauri/src/lib.rs|src-tauri/src/core/aiconfig/keychain.rs) ;;
    src-tauri/tests/config_invariants.rs|src-tauri/tests/story_e7_r4_population.rs) ;;
    _bmad-output/implementation-artifacts/spec-epic-7-retro-r-4-*.md) ;;
    _bmad-output/implementation-artifacts/e7-r4-ban-do/*) ;;
    _bmad-output/implementation-artifacts/deferred-work.md|_bmad-output/implementation-artifacts/sprint-status.yaml) ;;
    *) die "product tree is not clean: $changed" ;;
  esac
done

SCRATCH="$(mktemp -d /tmp/auratranslate-nfr-bench-XXXXXX)"
BENCH_HOME="$SCRATCH/home"
APPDATA="$BENCH_HOME/Library/Application Support/com.auratranslate.desktop"
GLOBAL_DB="$APPDATA/global.db"
PHASE_STATE="$BENCH_HOME/.auratranslate-nfr-bench-phase"
APP_PID=''
mkdir -p "$BENCH_HOME/Documents"

cleanup() {
  if [[ -n "$APP_PID" ]] && kill -0 "$APP_PID" 2>/dev/null; then
    kill "$APP_PID" 2>/dev/null || true
    for _ in {1..20}; do kill -0 "$APP_PID" 2>/dev/null || break; sleep 0.1; done
    kill -9 "$APP_PID" 2>/dev/null || true
  fi
  case "$SCRATCH" in
    /tmp/auratranslate-nfr-bench-*) rm -rf "$SCRATCH" ;;
    *) print -u2 "refusing to delete a scratch without the marker: $SCRATCH" ;;
  esac
}
trap cleanup EXIT INT TERM HUP

print '== build the population (100k Work-tier + 100k Global TM pairs, 300-segment Chapter) =='
build_log="$SCRATCH/population-build.log"
AURA_E7_R4_HOME="$BENCH_HOME" AURA_E7_R4_WORK_NAME="$WORK_NAME" \
  cargo test --profile bench-release --locked --manifest-path src-tauri/Cargo.toml \
    --test story_e7_r4_population -- --ignored --nocapture 2>&1 | tee "$build_log"
grep -q '^test builds_the_e7_r4_population_in_the_scratch_bench_home ... ok$' "$build_log" \
  || die 'population builder is not green'
POPULATION_LINE="$(grep '^E7_R4_POPULATION' "$build_log" || true)"
[[ -n "$POPULATION_LINE" ]] || die 'builder printed no E7_R4_POPULATION line'
for expected in 'chapters=1' 'segments=300' 'draft_segments=300' 'distinct_sources=300' 'work_tm_pairs=100000' \
  'global_tm_pairs=100000' 'chapter_exact_hits=150' 'verdict=matches_declaration'; do
  [[ "$POPULATION_LINE" == *"$expected"* ]] || die "population differs from the declaration, missing $expected: $POPULATION_LINE"
done

printf 'library\n' > "$PHASE_STATE"

print '== launch the packaged release app =='
# LaunchServices launch: a binary spawned from a background shell never becomes the active app,
# and its WKWebView stalled IPC mid-run.
open -n "$APP" \
  --env "HOME=$BENCH_HOME" \
  --env "AURA_NFR_BENCH_PHASE_BUDGET_SECS=${AURA_E7_R4_TIMEOUT_SECS:-1800}" \
  --env "AURA_NFR_BENCH_WORK_NAME=$WORK_NAME" \
  --env "AURA_NFR_BENCH_WORKS=1" \
  --stdout "$SCRATCH/app.log" --stderr "$SCRATCH/app.log"
for _ in {1..100}; do
  APP_PID="$(pgrep -f "$APP_BIN" | head -1 || true)"
  [[ -z "$APP_PID" ]] || break
  sleep 0.1
done
[[ -n "$APP_PID" ]] || die 'the app did not start'

read_marker() {
  sqlite3 -readonly "$GLOBAL_DB" "SELECT value FROM config_value WHERE kind='app_config' AND key='$1';" 2>/dev/null || true
}

DEADLINE=$(( $(date +%s) + ${AURA_E7_R4_TIMEOUT_SECS:-1800} ))
RESULT=''
LAST_PROGRESS=0
while true; do
  if [[ -f "$GLOBAL_DB" ]]; then
    invalid="$(read_marker '__nfr_bench_invalid__')"
    [[ -z "$invalid" ]] || die "probe invalid: $invalid"
    error="$(read_marker '__nfr_bench_e7_probe_error__')"
    [[ -z "$error" ]] || die "probe error: $error"
    RESULT="$(read_marker '__nfr_bench_e7_probe_done__')"
    [[ -z "$RESULT" ]] || break
  fi
  if (( $(date +%s) - LAST_PROGRESS >= 120 )); then
    LAST_PROGRESS=$(date +%s)
    print "stage $(read_marker '__nfr_bench_e7_progress__')"
  fi
  kill -0 "$APP_PID" 2>/dev/null || die "app died before the probe finished; see $SCRATCH/app.log"
  (( $(date +%s) < DEADLINE )) || die 'probe did not finish within the time cap'
  sleep 0.5
done
ENDED_AT="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
END_LOAD="$(uptime)"
print -r -- "$RESULT" > "$RAW"

{
  print "measurement_started_utc=$STARTED_AT"
  print "measurement_ended_utc=$ENDED_AT"
  print "baseline_commit=$(git rev-parse HEAD)"
  print "working_diff_sha256=$( { git diff --binary HEAD; git ls-files --others --exclude-standard | LC_ALL=C sort | while IFS= read -r item; do printf '%s\0' "$item"; shasum -a 256 "$item"; done; } | shasum -a 256 | awk '{print $1}')"
  print "release_app_sha256=$(shasum -a 256 "$APP_BIN" | awk '{print $1}')"
  print 'profile=release, feature nfr-bench, no dictionary layers bundled'
  print "os=$(sw_vers -productName) $(sw_vers -productVersion) ($(sw_vers -buildVersion))"
  print "model=$(sysctl -n hw.model)"
  print "cpu=$(sysctl -n machdep.cpu.brand_string)"
  print "logical_cpu=$(sysctl -n hw.logicalcpu)"
  print "ram_bytes=$(sysctl -n hw.memsize)"
  print "rustc=$(rustc --version)"
  print "load_before=$START_LOAD"
  print "load_after=$END_LOAD"
  print 'watcher=try_lock poll, yield_now spin; hold = longest single held window per round'
  print 'ai_endpoint=http://127.0.0.1:1/v1; keychain read stubbed under nfr-bench'
} > "$SCRIPT_DIR/environment.txt"

node "$SCRIPT_DIR/summarize.mjs" "$RAW" "$POPULATION_LINE"
print "\nDONE: raw data in $RAW, conditions in $SCRIPT_DIR/environment.txt"
