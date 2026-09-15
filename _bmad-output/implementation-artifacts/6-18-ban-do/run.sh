#!/bin/zsh
# Một lượt đo truy nguyên được cho Story 6.18 — NFR3/NFR4/NFR5 trên thư viện 5.000 Chương
# THẬT (50 Tác phẩm × 100 Chương × 10 segment), dựng qua ĐÚNG đường sản phẩm
# (`story_6_18_library.rs`), không phải fixture SQL thô của Story 5.14. Chỉ tạo dữ liệu dưới
# HOME có marker `auratranslate-nfr-bench-`; trap luôn giết app và xoá HOME ấy.
set -euo pipefail

SCRIPT_DIR="${0:A:h}"
REPO="${SCRIPT_DIR:h:h:h}"
APP="$REPO/src-tauri/target/release/bundle/macos/AuraTranslate.app"
APP_BIN="$APP/Contents/MacOS/auratranslate"
STARTED_AT="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
START_LOAD="$(uptime)"

die() {
  print -u2 "LỖI: $*"
  exit 1
}

cd "$REPO"

# Cây mã sản phẩm chỉ được lệch ở đúng móc feature-gated của bàn đo VÀ tài liệu/tracking của
# chính story — lượt đo chạy trước commit (memory "Cây bẩn trước story → commit riêng").
git diff --check
git diff --cached --check
typeset -a changed_paths
changed_paths=("${(@f)$( { git diff --name-only; git diff --cached --name-only; git ls-files --others --exclude-standard; } | LC_ALL=C sort -u )}")
for changed in "${changed_paths[@]}"; do
  [[ -z "$changed" ]] && continue
  case "$changed" in
    src-tauri/Cargo.toml|src-tauri/src/lib.rs) ;;
    src-tauri/tests/library_index_contract.rs|src-tauri/tests/segment_contract.rs) ;;
    src-tauri/tests/config_invariants.rs|src-tauri/tests/story_6_18_library.rs) ;;
    src-tauri/tests/story_6_18_bench_transition.rs) ;;
    # Story 6.18 task 6 (debt probes) -- ba tệp mang perf_probe_* mới và ca frontend
    # `chaptersShowAll` mới; cùng lý do allowlist ở trên: artefact hợp lệ của chính story,
    # không phải cây bẩn.
    src-tauri/tests/cleanup_contract.rs|src-tauri/tests/story_6_18_debt_probes.rs) ;;
    src-tauri/tests/webimport_contract.rs|tests/frontend/importPreviewChapters.test.ts) ;;
    _bmad-output/implementation-artifacts/spec-5-14-*.md) ;;
    _bmad-output/implementation-artifacts/spec-6-18-*.md) ;;
    _bmad-output/implementation-artifacts/sprint-status.yaml) ;;
    _bmad-output/implementation-artifacts/deferred-work.md) ;;
    _bmad-output/implementation-artifacts/5-14-ban-do/*) ;;
    _bmad-output/implementation-artifacts/6-18-ban-do/*) ;;
    _bmad-output/specs/spec-AuraTranslate/SPEC.md) ;;
    _bmad-output/specs/spec-AuraTranslate/requirements.md) ;;
    _bmad-output/planning-artifacts/prds/prd-AuraTranslate-2026-08-02/prd.md) ;;
    *) die "cây mã sản phẩm không sạch: $changed" ;;
  esac
done

SCRATCH="$(mktemp -d /tmp/auratranslate-nfr-bench-XXXXXX)"
BENCH_HOME="$SCRATCH/home"
LIBRARY_ROOT="$BENCH_HOME/Documents/AuraTranslate"
APPDATA="$BENCH_HOME/Library/Application Support/com.auratranslate.desktop"
GLOBAL_DB="$APPDATA/global.db"
ACTIVE_APP_PID=''
ACTIVE_WEBKIT_EXPECTED=''
# 🔴 Command feature-gated chỉ đọc phase-file DUY NHẤT nằm trong HOME nhập — cùng ràng buộc
# Ice ký 2026-09-02 mà Story 6.18 Quyết định 5 tái dùng nguyên qua tên `nfr-bench`.
PHASE_STATE="$BENCH_HOME/.auratranslate-nfr-bench-phase"
mkdir -p "$BENCH_HOME/Documents"

# Scratch phải biến mất kể cả lượt đỏ, nhưng một lỗi harness không được biến mất cùng nó.
RUN_LOG="$SCRIPT_DIR/latest-run.log"
exec >"$RUN_LOG" 2>&1

cleanup() {
  if [[ -n "$ACTIVE_APP_PID" ]] && kill -0 "$ACTIVE_APP_PID" 2>/dev/null; then
    kill "$ACTIVE_APP_PID" 2>/dev/null || true
    for _ in {1..20}; do
      kill -0 "$ACTIVE_APP_PID" 2>/dev/null || break
      sleep 0.1
    done
    kill -9 "$ACTIVE_APP_PID" 2>/dev/null || true
  fi
  ACTIVE_APP_PID=''
  case "$SCRATCH" in
    /tmp/auratranslate-nfr-bench-*) rm -rf "$SCRATCH" ;;
    *) print -u2 "không xoá scratch không mang marker: $SCRATCH" ;;
  esac
}
trap cleanup EXIT INT TERM HUP

# ─────────────────────────────────────────────────────────────────────────────
# Hằng số quần thể — PHẢI khớp `story_6_18_library.rs` (spec 6.18 Quyết định 2)
# ─────────────────────────────────────────────────────────────────────────────
WORKS=50
CHAPTERS_PER_WORK=100
SEGMENTS_PER_CHAPTER=10
TOTAL_CHAPTERS=$((WORKS * CHAPTERS_PER_WORK))
TOTAL_SEGMENTS=$((TOTAL_CHAPTERS * SEGMENTS_PER_CHAPTER))
TARGET_WORK_NAME='NFR Story 6.18 Work 00'

NFR3_RAW="$SCRIPT_DIR/nfr3-raw.tsv"
STARTUP_RAW="$SCRIPT_DIR/startup-raw.tsv"
MEMORY_RAW="$SCRIPT_DIR/memory-raw.tsv"
TRANSITION_RAW="$SCRIPT_DIR/transition-raw.tsv"
# Trần liveness của MỘT lượt chuyển pha. Xem `5-14-ban-do/run.sh` cho lý do nới mặc định.
PHASE_BUDGET_S="${AURA_NFR_BENCH_PHASE_BUDGET_SECS:-600}"
export AURA_NFR_BENCH_PHASE_BUDGET_SECS="$PHASE_BUDGET_S"
# §Always spec 6.18: NFR4 đòi ≥10 session. Cho phép thu hẹp lượt chẩn đoán mà KHÔNG sửa mã —
# `summarize.mjs` vẫn đòi đúng 10 cho một report được coi là ĐẦY ĐỦ (không sơ bộ hơn nữa).
SESSION_LIST="${AURA_NFR_BENCH_SESSIONS:-1 2 3 4 5 6 7 8 9 10}"
FIXTURE_LIST="${AURA_NFR_BENCH_FIXTURES:-full frontier}"
[[ "${AURA_NFR_BENCH_RESUME:-0}" != 1 ]] || die 'Story 6.18 từ chối AURA_NFR_BENCH_RESUME: chạy mới toàn bộ để giữ provenance'
printf 'session\trecord\tcase\tquery\twarmups\tsamples\tp50_ms\tp95_ms\tp99_ms\tworst_ms\n' > "$NFR3_RAW"
printf 'session\tfixture\ttemperature\telapsed_ms\tstatus\tnote\n' > "$STARTUP_RAW"
printf 'session\tfixture\tphase\tsample\tapp_pid\twebkit_pids\tpid_count\tphys_footprint_bytes\trss_bytes\tstatus\tnote\n' > "$MEMORY_RAW"
printf 'session\tfixture\ttransition\tbudget_s\telapsed_ms\tstatus\tnote\n' > "$TRANSITION_RAW"

# ─────────────────────────────────────────────────────────────────────────────
# Dựng thư viện 5.000 Chương THẬT qua đúng đường sản phẩm (task 3) — MỘT lần, xuất ra
# `$LIBRARY_ROOT`. Không resume, cùng lý do NFR3/NFR4/NFR5 dưới đây: provenance.
# ─────────────────────────────────────────────────────────────────────────────
print '== dựng thư viện 6.18 (50 Tác phẩm x 100 Chương x 10 segment) qua product import =='
build_log="$SCRATCH/library-build.log"
AURA_6_18_EXPORT_LIBRARY_ROOT="$LIBRARY_ROOT" \
  cargo test --profile bench-release --locked --manifest-path src-tauri/Cargo.toml \
    --test story_6_18_library -- --ignored --nocapture 2>&1 | tee "$build_log"
grep -q '^test builds_the_6_18_library_through_product_import_and_lifecycle_code ... ok$' "$build_log" \
  || die 'builder thư viện 6.18 không xanh'
# Matrix row "Build library": "build time + peak RSS recorded" -- dòng `STORY_6_18_BUILD_STATS`
# do chính builder in ra (audit 2026-09-14 P3, trước bản vá này không dòng nào ghi lại hai số
# này). Thiếu dòng ⇒ `die` cùng chỗ, không lặng lẽ để `fixture.txt` thiếu trường.
BUILD_STATS_LINE="$(grep '^STORY_6_18_BUILD_STATS\t' "$build_log" || true)"
[[ -n "$BUILD_STATS_LINE" ]] || die 'builder không in STORY_6_18_BUILD_STATS -- build time/peak RSS chưa ghi lại'
BUILD_WALL_MS="$(print -r -- "$BUILD_STATS_LINE" | awk -F '\t' '{ for (i=1;i<=NF;i++) if ($i ~ /^build_wall_ms=/) { split($i,a,"="); print a[2] } }')"
BUILD_PEAK_RSS_KB="$(print -r -- "$BUILD_STATS_LINE" | awk -F '\t' '{ for (i=1;i<=NF;i++) if ($i ~ /^peak_rss_kb=/) { split($i,a,"="); print a[2] } }')"
BUILD_PEAK_RSS_MB="$(print -r -- "$BUILD_STATS_LINE" | awk -F '\t' '{ for (i=1;i<=NF;i++) if ($i ~ /^peak_rss_mb=/) { split($i,a,"="); print a[2] } }')"
[[ -n "$BUILD_WALL_MS" && -n "$BUILD_PEAK_RSS_KB" && -n "$BUILD_PEAK_RSS_MB" ]] \
  || die "STORY_6_18_BUILD_STATS thiếu trường: $BUILD_STATS_LINE"

print '== đọc lại quần thể ĐỘC LẬP với builder — §Always: kiểm trước bất kỳ lượt đo nào =='
typeset -a project_dbs
project_dbs=("${(@f)$(find "$LIBRARY_ROOT" -type f -name project.db -print)}")
[[ "${#project_dbs[@]}" == "$WORKS" ]] || die "thư viện cần đúng $WORKS project.db, nhận ${#project_dbs[@]}"
READ_CHAPTERS=0
READ_LIVE_SEGMENTS=0
READ_DONE=0
READ_NOT_STARTED=0
for db in "${project_dbs[@]}"; do
  c="$(sqlite3 "file:$db?immutable=1" 'SELECT COUNT(*) FROM chapter;')"
  live="$(sqlite3 "file:$db?immutable=1" 'SELECT COUNT(*) FROM segment WHERE retired_at IS NULL;')"
  done_n="$(sqlite3 "file:$db?immutable=1" "SELECT COUNT(*) FROM chapter WHERE status='done';")"
  not_started_n="$(sqlite3 "file:$db?immutable=1" "SELECT COUNT(*) FROM chapter WHERE status='not_started';")"
  READ_CHAPTERS=$((READ_CHAPTERS + c))
  READ_LIVE_SEGMENTS=$((READ_LIVE_SEGMENTS + live))
  READ_DONE=$((READ_DONE + done_n))
  READ_NOT_STARTED=$((READ_NOT_STARTED + not_started_n))
done
[[ "$READ_CHAPTERS" == "$TOTAL_CHAPTERS" && "$READ_LIVE_SEGMENTS" == "$TOTAL_SEGMENTS" ]] \
  || die "quần thể đọc lại sai: works=${#project_dbs[@]} chapters=$READ_CHAPTERS segments=$READ_LIVE_SEGMENTS (kỳ vọng $WORKS/$TOTAL_CHAPTERS/$TOTAL_SEGMENTS)"
[[ "$READ_DONE" == "$TOTAL_CHAPTERS" && "$READ_NOT_STARTED" == 0 ]] \
  || die "trạng thái Chương lúc export sai: done=$READ_DONE not_started=$READ_NOT_STARTED (builder phải để lại trạng thái full sạch)"
LIBRARY_BYTES="$(find "$LIBRARY_ROOT" -type f -exec stat -f '%z' {} + | awk '{ total += $1 } END { print total+0 }')"
{
  print "works=${#project_dbs[@]}"
  print "chapters=$READ_CHAPTERS"
  print "segments=$READ_LIVE_SEGMENTS"
  print "library_bytes=$LIBRARY_BYTES"
  print "library_MB=$(awk -v n="$LIBRARY_BYTES" 'BEGIN { printf "%.3f", n/1000000 }')"
  print "library_MiB=$(awk -v n="$LIBRARY_BYTES" 'BEGIN { printf "%.3f", n/1048576 }')"
  print "shape=50 Tác phẩm; 100 Chương/Tác phẩm; 10 segment/Chương; nhập qua confirm_bilingual_import + lifecycle::set_chapter_status + Indexer::rebuild"
  print "build_wall_ms=$BUILD_WALL_MS"
  print "build_peak_rss_kb=$BUILD_PEAK_RSS_KB"
  print "build_peak_rss_mb=$BUILD_PEAK_RSS_MB"
  print 'build_stats_note=ps -o rss= mau moi 20ms tren tien trinh cargo test builder, khong phai footprint; bao trum dung tu luc bat dau import den luc quan the full duoc kiem xong'
} > "$SCRIPT_DIR/fixture.txt"

print '== NFR3: ba session release, năm ca tách biệt, đọc thư viện 6.18 =='
NFR3_BIN=''
for session in 1 2 3; do
  log="$SCRATCH/nfr3-$session.log"
  if [[ "$session" == 1 ]]; then
    AURA_6_18_LIBRARY_ROOT="$LIBRARY_ROOT" \
      cargo test --profile bench-release --locked --manifest-path src-tauri/Cargo.toml \
        --test library_index_contract bench_p95_of_a_library_search_over_the_story_6_18_library \
        -- --ignored --nocapture 2>&1 | tee "$log"
    NFR3_BIN="$(awk -F '[()]' '/Running tests\/library_index_contract\.rs/ { path=$2 } END { print path }' "$log")"
    [[ -x "$NFR3_BIN" ]] || die "Cargo không in executable NFR3 vừa chạy: $NFR3_BIN"
  else
    AURA_6_18_LIBRARY_ROOT="$LIBRARY_ROOT" \
      "$NFR3_BIN" bench_p95_of_a_library_search_over_the_story_6_18_library \
        --exact --ignored --nocapture 2>&1 | tee "$log"
  fi
  awk -F '\t' -v session="$session" '
    /^NFR3_CASE\t/ {
      for (i=3; i<=NF; i++) { split($i, a, "="); value[a[1]]=substr($i, index($i, "=")+1) }
      printf "%s\tcase\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n", session, $2, value["query"], value["warmups"], value["samples"], value["p50_ms"], value["p95_ms"], value["p99_ms"], value["worst_ms"]
      delete value
    }
  ' "$log" >> "$NFR3_RAW"
  [[ "$(awk -F '\t' -v s="$session" '$1 == s && $2 == "case" { n++ } END { print n+0 }' "$NFR3_RAW")" == 5 ]] \
    || die "session NFR3 $session không có đúng năm ca"
done

# Thư viện đã "full" sạch từ builder — NFR3_BIN cũng cần một binary chuyển trạng thái riêng,
# dựng MỘT lần ở đây để hai lượt chuyển đổi sau (mỗi lượt full/frontier) không build lại.
TRANSITION_BIN=''
set_target_work_status() {
  local target_status="$1"
  local log="$SCRATCH/transition-$target_status-$RANDOM.log"
  if [[ -z "$TRANSITION_BIN" ]]; then
    AURA_6_18_LIBRARY_ROOT="$LIBRARY_ROOT" AURA_6_18_BENCH_TARGET_STATUS="$target_status" \
      cargo test --profile bench-release --locked --manifest-path src-tauri/Cargo.toml \
        --test story_6_18_bench_transition -- --ignored --nocapture 2>&1 | tee "$log"
    TRANSITION_BIN="$(awk -F '[()]' '/Running tests\/story_6_18_bench_transition\.rs/ { path=$2 } END { print path }' "$log")"
    [[ -x "$TRANSITION_BIN" ]] || die "Cargo không in executable transition vừa chạy: $TRANSITION_BIN"
  else
    AURA_6_18_LIBRARY_ROOT="$LIBRARY_ROOT" AURA_6_18_BENCH_TARGET_STATUS="$target_status" \
      "$TRANSITION_BIN" toggles_the_frontier_work_between_full_and_frontier_through_product_lifecycle_code \
        --exact --ignored --nocapture 2>&1 | tee "$log"
  fi
  grep -q "verdict=matches_target" "$log" \
    || die "chuyển trạng thái Tác phẩm đích sang $target_status không khớp (không phải sqlite3 UPDATE — §Always cấm SQL ghi ở đường dựng, xem log $log)"
}

print '== app release + probe production + lớp từ điển thật =='
"$SCRIPT_DIR/build.sh" 2>&1 | tee "$SCRATCH/build.log"
[[ -x "$APP_BIN" ]] || die "thiếu app release: $APP_BIN"

set_phase() {
  local phase="$1"
  case "$phase" in
    library|reading-full|reading-frontier|back-library|discard) ;;
    *) die "pha harness ngoài danh mục: $phase" ;;
  esac
  local next="$PHASE_STATE.next-$$"
  printf '%s\n' "$phase" > "$next"
  mv -f "$next" "$PHASE_STATE"
}

set_phase library

monotonic_ns() {
  python3 -c 'import time; print(time.monotonic_ns())'
}

webkit_pids() {
  ps -axo pid=,comm= | awk 'index($0, "com.apple.WebKit.") { print $1 }' | LC_ALL=C sort -u
}

new_webkit_pids() {
  local baseline="$1"
  local current="$SCRATCH/webkit-current.txt"
  webkit_pids > "$current"
  comm -13 "$baseline" "$current"
}

stop_app() {
  if [[ -n "$ACTIVE_APP_PID" ]] && kill -0 "$ACTIVE_APP_PID" 2>/dev/null; then
    kill "$ACTIVE_APP_PID" 2>/dev/null || true
    for _ in {1..30}; do
      kill -0 "$ACTIVE_APP_PID" 2>/dev/null || break
      sleep 0.1
    done
    kill -9 "$ACTIVE_APP_PID" 2>/dev/null || true
    wait "$ACTIVE_APP_PID" 2>/dev/null || true
  fi
  ACTIVE_APP_PID=''
}

reset_probe_markers() {
  [[ -f "$GLOBAL_DB" ]] || return 0
  sqlite3 "$GLOBAL_DB" "DELETE FROM config_value WHERE kind='app_config' AND key LIKE '__nfr_bench_%'; \
    INSERT INTO config_value(kind,key,value,updated_at) VALUES('app_config','mode','library',strftime('%Y-%m-%dT%H:%M:%fZ','now')) \
    ON CONFLICT(kind,key) DO UPDATE SET value='library', updated_at=excluded.updated_at;"
}

clear_library_index() {
  case "$APPDATA" in
    /tmp/auratranslate-nfr-bench-*/home/Library/Application\ Support/com.auratranslate.desktop)
      rm -f "$APPDATA/library-index.db" "$APPDATA/library-index.db-wal" "$APPDATA/library-index.db-shm"
      ;;
    *) die "từ chối xoá index ngoài HOME nháp: $APPDATA" ;;
  esac
}

wait_marker() {
  local key="$1"
  local timeout_s="$2"
  local loops=$((timeout_s * 20))
  local value invalid
  for _ in $(seq 1 "$loops"); do
    if [[ -f "$GLOBAL_DB" ]]; then
      invalid="$(sqlite3 -readonly "$GLOBAL_DB" "SELECT value FROM config_value WHERE kind='app_config' AND key='__nfr_bench_invalid__';" 2>/dev/null || true)"
      [[ -z "$invalid" ]] || { print -u2 "probe invalid: $invalid"; return 2; }
      value="$(sqlite3 -readonly "$GLOBAL_DB" "SELECT value FROM config_value WHERE kind='app_config' AND key='$key';" 2>/dev/null || true)"
      [[ -z "$value" ]] || { print -r -- "$value"; return 0; }
    fi
    kill -0 "$ACTIVE_APP_PID" 2>/dev/null || return 3
    sleep 0.05
  done
  return 1
}

launch_to_usable() {
  local session="$1"
  local fixture="$2"
  local temperature="$3"
  local keep_alive="$4"
  local baseline="$SCRATCH/webkit-before-${session}-${fixture}-${temperature}.txt"
  local app_log="$SCRATCH/app-${session}-${fixture}-${temperature}.log"
  local started_ns ended_ns elapsed marker

  set_phase library
  reset_probe_markers
  [[ "$temperature" != cold ]] || clear_library_index
  webkit_pids > "$baseline"
  started_ns="$(monotonic_ns)"
  # Ba biến môi trường đúng Quyết định 5/task 5: tên Tác phẩm đích, số Tác phẩm grid phải
  # mang, và số segment kỳ vọng ở pha `reading-full` — tất cả đọc bởi `nfr_bench` (lib.rs),
  # KHÔNG hardcode "5.14 Fixture"/50.000 như bản gốc Story 5.14.
  HOME="$BENCH_HOME" \
    AURA_NFR_BENCH_WORK_NAME="$TARGET_WORK_NAME" \
    AURA_NFR_BENCH_WORKS="$WORKS" \
    AURA_NFR_BENCH_READING_FULL_SEGMENTS="$((CHAPTERS_PER_WORK * SEGMENTS_PER_CHAPTER))" \
    "$APP_BIN" >> "$app_log" 2>&1 &
  ACTIVE_APP_PID=$!
  if ! marker="$(wait_marker '__nfr_bench_usable__' 180)"; then
    printf '%s\t%s\t%s\t\tunknown\tusable marker vắng hoặc invalid\n' "$session" "$fixture" "$temperature" >> "$STARTUP_RAW"
    stop_app
    return 1
  fi
  ended_ns="$(monotonic_ns)"
  elapsed="$(awk -v a="$started_ns" -v b="$ended_ns" 'BEGIN { printf "%.3f", (b-a)/1000000 }')"
  node -e 'const v=JSON.parse(process.argv[1]); if(v.works!==Number(process.argv[2]) || v.work_name!==process.argv[3]) process.exit(2)' \
    "$marker" "$WORKS" "$TARGET_WORK_NAME" \
    || die "usable marker không khớp thư viện 6.18: $marker"
  # §Always spec 6.18: "The run is invalid unless the probe reads a loaded-layer count > 0
  # before the first sample" — HARNESS từ chối ở đây, trước mẫu NFR5 đầu tiên (§I/O Matrix
  # "Red controls": "Layers stripped ... Harness refuses before sampling"). Command chia sẻ
  # `nfr-bench` không tự chặn (5.14 vẫn hợp lệ với 0 lớp, lịch sử) -- xem `lib.rs` nhánh
  # `"usable"`.
  node -e 'const v=JSON.parse(process.argv[1]); if(!(Number(v.layers) > 0)) process.exit(2)' "$marker" \
    || die "0 lớp từ điển đã nạp trước mẫu đầu tiên — refusal đặt tên: dict layers stripped or resource_dir() wrong, marker=$marker"
  kill -0 "$ACTIVE_APP_PID" 2>/dev/null || die 'app chết ngay sau usable'
  printf '%s\t%s\t%s\t%s\tok\tpre-spawn tới marker grid có đủ %s Tác phẩm + %s lớp từ điển\n' \
    "$session" "$fixture" "$temperature" "$elapsed" "$WORKS" \
    "$(node -e 'console.log(JSON.parse(process.argv[1]).layers)' "$marker")" >> "$STARTUP_RAW"

  if [[ "$keep_alive" == yes ]]; then
    ACTIVE_BASELINE="$baseline"
    ACTIVE_WEBKIT_EXPECTED="$SCRATCH/webkit-expected-${session}-${fixture}-${temperature}.txt"
    new_webkit_pids "$ACTIVE_BASELINE" > "$ACTIVE_WEBKIT_EXPECTED"
    [[ -s "$ACTIVE_WEBKIT_EXPECTED" ]] || die 'usable không sinh WebKit mới; app PID đơn lẻ bị từ chối'
  else
    set_phase discard
    stop_app
    set_phase library
  fi
}

append_memory_error() {
  local session="$1" fixture="$2" phase="$3" sample="$4" note="$5"
  printf '%s\t%s\t%s\t%s\t%s\t\t1\t\t\terror\t%s\n' \
    "$session" "$fixture" "$phase" "$sample" "$ACTIVE_APP_PID" "$note" >> "$MEMORY_RAW"
}

sample_memory_once() {
  local session="$1" fixture="$2" phase="$3" sample="$4" baseline="$5" expected="$6"
  local new pid csv footprint_file footprint_result footprint_count phys rss_value rss_total current
  typeset -a pids

  new="$(new_webkit_pids "$baseline")"
  current="$SCRATCH/webkit-observed-${session}-${fixture}-${phase}-${sample}.txt"
  print -r -- "$new" > "$current"
  cmp -s "$expected" "$current" || { append_memory_error "$session" "$fixture" "$phase" "$sample" 'tập WebKit mới sinh đổi sau usable; từ chối trộn PID ngoài phạm vi'; return; }
  [[ -n "$new" ]] || { append_memory_error "$session" "$fixture" "$phase" "$sample" 'không có WebKit mới sinh; PID app đơn lẻ bị từ chối'; return; }
  pids=("$ACTIVE_APP_PID")
  for pid in ${(f)new}; do pids+=("$pid"); done
  for pid in "${pids[@]}"; do
    kill -0 "$pid" 2>/dev/null || { append_memory_error "$session" "$fixture" "$phase" "$sample" "PID $pid đã chết"; return; }
  done
  csv="${(j:,:)pids}"
  footprint_file="$SCRATCH/footprint-${session}-${fixture}-${phase}-${sample}.txt"
  if ! /usr/bin/footprint -f bytes --noCategories "${pids[@]}" > "$footprint_file" 2>&1; then
    append_memory_error "$session" "$fixture" "$phase" "$sample" 'footprint trả lỗi'
    return
  fi
  footprint_result="$(awk '/^[[:space:]]*phys_footprint:/ { total += $2; count++ } END { print count+0, total+0 }' "$footprint_file")"
  footprint_count="${footprint_result%% *}"
  phys="${footprint_result#* }"
  [[ "$footprint_count" == "${#pids[@]}" && "$phys" -gt 0 ]] \
    || { append_memory_error "$session" "$fixture" "$phase" "$sample" "thiếu phys_footprint: $footprint_count/${#pids[@]} PID"; return; }

  rss_total=0
  for pid in "${pids[@]}"; do
    rss_value="$(ps -o rss= -p "$pid" | tr -d ' ')"
    [[ -n "$rss_value" ]] || { append_memory_error "$session" "$fixture" "$phase" "$sample" "thiếu RSS PID $pid"; return; }
    rss_total=$((rss_total + rss_value * 1024))
  done
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\tok\tapp + WebKit mới sinh\n' \
    "$session" "$fixture" "$phase" "$sample" "$ACTIVE_APP_PID" "${(j:,:)pids[2,-1]}" \
    "${#pids[@]}" "$phys" "$rss_total" >> "$MEMORY_RAW"
}

sample_phase() {
  local session="$1" fixture="$2" phase="$3" baseline="$4" expected="$5"
  local started_ns ended_ns elapsed_ms
  started_ns="$(monotonic_ns)"
  for sample in {1..10}; do
    sample_memory_once "$session" "$fixture" "$phase" "$sample" "$baseline" "$expected"
    sleep 0.35
  done
  ended_ns="$(monotonic_ns)"
  elapsed_ms=$(( (ended_ns - started_ns) / 1000000 ))
  [[ "$elapsed_ms" -lt 60000 ]] \
    || die "pha $session/$fixture/$phase mất ${elapsed_ms} ms, vượt trần liveness 60.000 ms"
}

measure_memory_session() {
  local session="$1" fixture="$2" baseline="$3" marker expected_status expected_segments expected_frontier
  expected_status=content
  expected_segments=$((CHAPTERS_PER_WORK * SEGMENTS_PER_CHAPTER))
  expected_frontier=end-of-work
  if [[ "$fixture" == frontier ]]; then
    expected_status=frontier-only
    expected_segments=0
    expected_frontier=next-not-done
  fi

  sample_phase "$session" "$fixture" library "$baseline" "$ACTIVE_WEBKIT_EXPECTED"
  set_phase "reading-$fixture"
  local t0_ns t1_ns t_ms
  t0_ns="$(monotonic_ns)"
  if marker="$(wait_marker '__nfr_bench_reading__' "$PHASE_BUDGET_S")"; then
    t1_ns="$(monotonic_ns)"
    t_ms=$(( (t1_ns - t0_ns) / 1000000 ))
    printf '%s\t%s\treading\t%s\t%s\tok\t\n' \
      "$session" "$fixture" "$PHASE_BUDGET_S" "$t_ms" >> "$TRANSITION_RAW"
  else
    t1_ns="$(monotonic_ns)"
    t_ms=$(( (t1_ns - t0_ns) / 1000000 ))
    printf '%s\t%s\treading\t%s\t%s\ttimeout\tvượt trần liveness; KHÔNG kết luận treo\n' \
      "$session" "$fixture" "$PHASE_BUDGET_S" "$t_ms" >> "$TRANSITION_RAW"
    die "session $session/$fixture: Reading chưa xong trong ${t_ms} ms (trần ${PHASE_BUDGET_S} s)"
  fi
  node -e 'const v=JSON.parse(process.argv[1]); if(v.status!==process.argv[2] || v.segments!==Number(process.argv[3]) || v.frontier!==process.argv[4]) process.exit(2)' \
    "$marker" "$expected_status" "$expected_segments" "$expected_frontier" \
    || die "Reading marker sai fixture $fixture: $marker (kỳ vọng status=$expected_status segments=$expected_segments frontier=$expected_frontier)"
  sample_phase "$session" "$fixture" reading "$baseline" "$ACTIVE_WEBKIT_EXPECTED"
  set_phase back-library
  t0_ns="$(monotonic_ns)"
  if marker="$(wait_marker '__nfr_bench_back_library__' "$PHASE_BUDGET_S")"; then
    t1_ns="$(monotonic_ns)"
    t_ms=$(( (t1_ns - t0_ns) / 1000000 ))
    printf '%s\t%s\tback_library\t%s\t%s\tok\t\n' \
      "$session" "$fixture" "$PHASE_BUDGET_S" "$t_ms" >> "$TRANSITION_RAW"
  else
    t1_ns="$(monotonic_ns)"
    t_ms=$(( (t1_ns - t0_ns) / 1000000 ))
    printf '%s\t%s\tback_library\t%s\t%s\ttimeout\tvượt trần liveness; KHÔNG kết luận treo\n' \
      "$session" "$fixture" "$PHASE_BUDGET_S" "$t_ms" >> "$TRANSITION_RAW"
    die "session $session/$fixture: quay lại Library chưa xong trong ${t_ms} ms (trần ${PHASE_BUDGET_S} s)"
  fi
  node -e 'const v=JSON.parse(process.argv[1]); if(v.works!==Number(process.argv[2]) || v.work_name!==process.argv[3]) process.exit(2)' \
    "$marker" "$WORKS" "$TARGET_WORK_NAME" \
    || die "Library marker sai fixture $fixture: $marker"
  sample_phase "$session" "$fixture" back_library_keepalive "$baseline" "$ACTIVE_WEBKIT_EXPECTED"
  stop_app
}

print "== NFR4/NFR5: session [$SESSION_LIST] x fixture [$FIXTURE_LIST], trần pha ${PHASE_BUDGET_S}s =="
for session in ${=SESSION_LIST}; do
  if [[ "$FIXTURE_LIST" == *full* ]]; then
  set_target_work_status done
  launch_to_usable "$session" full cold no
  launch_to_usable "$session" full warm yes
  webkit_pids > "$SCRATCH/all-webkit-now.txt"
  : > "$SCRATCH/empty-webkit.txt"
  sample_memory_once "$session" full app_pid_only_guard 0 "$SCRATCH/all-webkit-now.txt" "$SCRATCH/empty-webkit.txt"
  [[ "$(tail -1 "$MEMORY_RAW" | awk -F '\t' '{print $10}')" == error ]] \
    || die 'hàng rào app-PID-only không tự kiểm đỏ'
  sed -i '' '$d' "$MEMORY_RAW"
  measure_memory_session "$session" full "$ACTIVE_BASELINE"
  fi

  if [[ "$FIXTURE_LIST" == *frontier* ]]; then
  set_target_work_status not_started
  launch_to_usable "$session" frontier warm yes
  measure_memory_session "$session" frontier "$ACTIVE_BASELINE"
  fi
done

# Thư viện trên đĩa (scratch, sắp bị `trap` xoá) kết thúc ở trạng thái "full" — cùng nghệ
# thuật `story_6_18_library.rs` để lại, không thiết yếu vì cả cây sắp mất, nhưng rẻ và nhất
# quán nếu ai đó gắn `AURA_6_18_EXPORT_LIBRARY_ROOT` ra ngoài SCRATCH để soi tay.
[[ "$FIXTURE_LIST" != *frontier* ]] || set_target_work_status done

ENDED_AT="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
{
  print "measurement_started_utc=$STARTED_AT"
  print "measurement_ended_utc=$ENDED_AT"
  print "baseline_commit=$(git rev-parse HEAD)"
  print "working_diff_sha256=$( { git diff --binary HEAD; git ls-files --others --exclude-standard | LC_ALL=C sort | while IFS= read -r item; do printf '%s\\0' "$item"; shasum -a 256 "$item"; done; } | shasum -a 256 | awk '{print $1}')"
  print "release_app_sha256=$(shasum -a 256 "$APP_BIN" | awk '{print $1}')"
  print 'product_tree_guard=only Story 6.18 (and 5.14 history) tests/artifacts/tracking plus feature-gated phase command allowed'
  print 'profile=release'
  print "sessions_nfr3=3"
  print "sessions_nfr4_nfr5=$SESSION_LIST"
  print 'nfr3_warmups_per_case=10'
  print 'nfr3_samples_per_case=200'
  print 'nfr5_idle_samples_per_phase=10'
  print "phase_liveness_budget_s=$PHASE_BUDGET_S"
  print "fixtures_run=$FIXTURE_LIST"
  print "library_shape=${WORKS} works x ${CHAPTERS_PER_WORK} chapters x ${SEGMENTS_PER_CHAPTER} segments = ${TOTAL_CHAPTERS} chapters / ${TOTAL_SEGMENTS} segments"
  print "reading_target_work=$TARGET_WORK_NAME"
  print "os=$(sw_vers -productName) $(sw_vers -productVersion) ($(sw_vers -buildVersion))"
  print "model=$(sysctl -n hw.model)"
  print "cpu=$(sysctl -n machdep.cpu.brand_string)"
  print "logical_cpu=$(sysctl -n hw.logicalcpu)"
  print "ram_bytes=$(sysctl -n hw.memsize)"
  print "rustc=$(rustc --version)"
  print "cargo=$(cargo --version)"
  print "node=$(node --version)"
  print "npm=$(npm --version)"
  print "tauri_cli=$(npx tauri --version)"
  print "sqlite=$(sqlite3 --version)"
  print "webkit=$(defaults read /System/Library/Frameworks/WebKit.framework/Resources/Info CFBundleShortVersionString 2>/dev/null || print unknown)"
  print "load_before=$START_LOAD"
  print "load_after=$(uptime)"
  print 'startup_clock=python time.monotonic_ns; end after usable marker persisted'
  print 'memory_primary=/usr/bin/footprint phys_footprint bytes summed over app + new WebKit PIDs'
  print 'memory_countercheck=ps RSS KiB multiplied by 1024, same PID set'
  print 'phase_control=feature-gated Tauri command persists a whitelisted marker then native-evals product tabs/DOM from HOME/.auratranslate-nfr-bench-phase; usable requires grid to hold all 50 real Works incl. reading target, plus loaded-layer count > 0; full proves content+1000 segments+end-of-work, frontier proves frontier-only+0 segments+next-not-done; absent from default build; no network/CSP/ATS override'
  print 'status_transitions=lifecycle::set_chapter_status per Chapter + Indexer::rebuild via story_6_18_bench_transition.rs, no raw SQL UPDATE against project.db (unlike the 5.14 harness this superseded)'
} > "$SCRIPT_DIR/environment.txt"

node "$SCRIPT_DIR/summarize.mjs"
print "\nĐÃ XONG — raw data và REPORT.md ở $SCRIPT_DIR"
