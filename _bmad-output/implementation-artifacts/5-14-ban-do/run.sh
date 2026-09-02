#!/bin/zsh
# Một lượt đo truy nguyên được cho Story 5.14. Chỉ tạo dữ liệu dưới HOME có marker
# `auratranslate-5-14-`; trap luôn giết app và xoá HOME ấy.
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

# Cây mã sản phẩm chỉ được lệch ở đúng móc feature-gated của bàn đo. Test đo và
# tài liệu/tracking của chính story được phép bẩn vì lượt đo chạy trước commit.
git diff --check
git diff --cached --check
typeset -a changed_paths
changed_paths=("${(@f)$( { git diff --name-only; git diff --cached --name-only; git ls-files --others --exclude-standard; } | LC_ALL=C sort -u )}")
for changed in "${changed_paths[@]}"; do
  [[ -z "$changed" ]] && continue
  case "$changed" in
    src-tauri/Cargo.toml|src-tauri/src/lib.rs) ;;
    src-tauri/tests/library_index_contract.rs|src-tauri/tests/segment_contract.rs) ;;
    _bmad-output/implementation-artifacts/spec-5-14-*.md) ;;
    _bmad-output/implementation-artifacts/sprint-status.yaml) ;;
    _bmad-output/implementation-artifacts/deferred-work.md) ;;
    _bmad-output/implementation-artifacts/5-14-ban-do/*) ;;
    _bmad-output/specs/spec-AuraTranslate/SPEC.md) ;;
    _bmad-output/specs/spec-AuraTranslate/requirements.md) ;;
    *) die "cây mã sản phẩm không sạch: $changed" ;;
  esac
done

SCRATCH="$(mktemp -d /tmp/auratranslate-5-14-XXXXXX)"
BENCH_HOME="$SCRATCH/home"
LIBRARY_ROOT="$BENCH_HOME/Documents/AuraTranslate"
APPDATA="$BENCH_HOME/Library/Application Support/com.auratranslate.desktop"
GLOBAL_DB="$APPDATA/global.db"
ACTIVE_APP_PID=''
ACTIVE_WEBKIT_EXPECTED=''
# 🔴 Command feature-gated chi doc phase-file DUY NHAT nam trong HOME nhap. De no o
# `$SCRATCH/phase.txt` dung la tach rieng hon, nhung vi pham rang buoc Ice ky 2026-09-02:
# command khong duoc doc/ghi tep ngoai HOME nhap cua phep do.
PHASE_STATE="$BENCH_HOME/.auratranslate-5-14-phase"
mkdir -p "$BENCH_HOME/Documents"

# Scratch phải biến mất kể cả lượt đỏ, nhưng một lỗi harness không được biến mất cùng nó.
# Log này bị `.gitignore` bằng `*.log`, bị ghi đè ở lượt kế và chỉ chứa stdout/stderr của
# runner; nó không mang DB, fixture hay số liệu nào được dùng để kết luận.
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
    /tmp/auratranslate-5-14-*) rm -rf "$SCRATCH" ;;
    *) print -u2 "không xoá scratch không mang marker: $SCRATCH" ;;
  esac
}
trap cleanup EXIT INT TERM HUP

NFR3_RAW="$SCRIPT_DIR/nfr3-raw.tsv"
READING_RAW="$SCRIPT_DIR/reading-run-raw.tsv"
STARTUP_RAW="$SCRIPT_DIR/startup-raw.tsv"
MEMORY_RAW="$SCRIPT_DIR/memory-raw.tsv"
TRANSITION_RAW="$SCRIPT_DIR/transition-raw.tsv"
# Trần liveness của MỘT lượt chuyển pha. Mặc định cũ 600 s biến "chưa xong trong 600 s"
# thành `die "không tới Reading"` — một mệnh đề về sản phẩm suy từ trần của bàn đo.
# Nới để ĐO được con số thật; mọi lượt đều ghi elapsed vào `transition-raw.tsv` kể cả khi đỏ.
PHASE_BUDGET_S="${AURA_5_14_PHASE_BUDGET_SECS:-600}"
export AURA_5_14_PHASE_BUDGET_SECS="$PHASE_BUDGET_S"
# Cho phép thu hẹp lượt chẩn đoán mà KHÔNG sửa mã: mặc định giữ nguyên 3 session, cả hai fixture.
SESSION_LIST="${AURA_5_14_SESSIONS:-1 2 3}"
FIXTURE_LIST="${AURA_5_14_FIXTURES:-full frontier}"
# 🔴 Không resume: số liệu cũ có thể được sinh bởi code/probe khác. Một lượt benchmark phải
# tự chứa đủ ba session, nên xoá-và-đo-lại rẻ hơn một report trộn revision không truy nguyên.
[[ "${AURA_5_14_RESUME:-0}" != 1 ]] || die 'Story 5.14 từ chối AURA_5_14_RESUME: chạy mới toàn bộ để giữ provenance'
printf 'session\trecord\tcase\tquery\twarmups\tsamples\tp50_ms\tp95_ms\tp99_ms\tworst_ms\n' > "$NFR3_RAW"
printf 'case\twarmups\tsamples\tp50_ms\tp95_ms\tp99_ms\tworst_ms\n' > "$READING_RAW"
printf 'session\tfixture\ttemperature\telapsed_ms\tstatus\tnote\n' > "$STARTUP_RAW"
printf 'session\tfixture\tphase\tsample\tapp_pid\twebkit_pids\tpid_count\tphys_footprint_bytes\trss_bytes\tstatus\tnote\n' > "$MEMORY_RAW"
printf 'session\tfixture\ttransition\tbudget_s\telapsed_ms\tstatus\tnote\n' > "$TRANSITION_RAW"

print '== NFR3: ba session release, năm ca tách biệt =='
NFR3_BIN=''
for session in 1 2 3; do
  log="$SCRATCH/nfr3-$session.log"
  if [[ "$session" == 1 ]]; then
    # `--profile bench-release`, KHÔNG `--release`: xem khối `[profile.bench-release]`
    # trong `src-tauri/Cargo.toml`. Dưới `release`, va chạm tên tệp đầu ra của
    # `auratranslate_lib` làm lượt biên dịch này đỏ/xanh theo thứ tự cache chứ không
    # theo mã. Bốn khoá tối ưu thừa kế nguyên; chỉ `panic` đổi, và test target vốn
    # luôn là `unwind`.
    cargo test --profile bench-release --locked --manifest-path src-tauri/Cargo.toml \
      --test library_index_contract bench_p95_of_a_library_search_over_five_thousand_chapters \
      -- --ignored --nocapture 2>&1 | tee "$log"
    # Không `find | head`: Cargo in CHÍNH executable mà nó vừa chạy; lấy đường này buộc hai
    # session sau cùng artifact bench-release, không thể rơi vào binary hash cũ còn trong deps/.
    NFR3_BIN="$(awk -F '[()]' '/Running tests\/library_index_contract\.rs/ { path=$2 } END { print path }' "$log")"
    [[ -x "$NFR3_BIN" ]] || die "Cargo không in executable NFR3 vừa chạy: $NFR3_BIN"
  else
    "$NFR3_BIN" bench_p95_of_a_library_search_over_five_thousand_chapters \
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

print '== read_reading_run + export fixture đã tự kiểm quần thể =='
reading_log="$SCRATCH/reading-run.log"
AURA_5_14_EXPORT_LIBRARY_ROOT="$LIBRARY_ROOT" \
  cargo test --profile bench-release --locked --manifest-path src-tauri/Cargo.toml \
    --test segment_contract bench_reading_run_over_five_thousand_chapters \
    -- --ignored --nocapture 2>&1 | tee "$reading_log"
awk -F '\t' '
  /^READING_RUN_CASE\t/ {
    for (i=3; i<=NF; i++) { split($i, a, "="); value[a[1]]=substr($i, index($i, "=")+1) }
    printf "%s\t%s\t%s\t%s\t%s\t%s\t%s\n", $2, value["warmups"], value["samples"], value["p50_ms"], value["p95_ms"], value["p99_ms"], value["worst_ms"]
    delete value
  }
' "$reading_log" >> "$READING_RAW"
[[ "$(($(wc -l < "$READING_RAW") - 1))" == 2 ]] || die 'không có đủ hai số read_reading_run'

typeset -a project_dbs
project_dbs=("${(@f)$(find "$LIBRARY_ROOT" -type f -name project.db -print)}")
[[ "${#project_dbs[@]}" == 1 ]] || die "fixture cần đúng một project.db, nhận ${#project_dbs[@]}"
PROJECT_DB="${project_dbs[1]}"
CHAPTERS="$(sqlite3 "file:$PROJECT_DB?immutable=1" 'SELECT COUNT(*) FROM chapter;')"
SEGMENTS="$(sqlite3 "file:$PROJECT_DB?immutable=1" 'SELECT COUNT(*) FROM segment WHERE retired_at IS NULL;')"
WORKS="$(find "$LIBRARY_ROOT" -mindepth 1 -maxdepth 1 -type d -name '*.atproj' | wc -l | tr -d ' ')"
[[ "$WORKS" == 1 && "$CHAPTERS" == 5000 && "$SEGMENTS" == 50000 ]] \
  || die "quần thể fixture sai: works=$WORKS chapters=$CHAPTERS segments=$SEGMENTS"
FIXTURE_BYTES="$(find "$LIBRARY_ROOT" -type f -exec stat -f '%z' {} + | awk '{ total += $1 } END { print total+0 }')"
PROJECT_DB_BYTES="$(stat -f '%z' "$PROJECT_DB")"
{
  print "works=$WORKS"
  print "chapters=$CHAPTERS"
  print "segments=$SEGMENTS"
  print "fixture_bytes=$FIXTURE_BYTES"
  print "fixture_MB=$(awk -v n="$FIXTURE_BYTES" 'BEGIN { printf "%.3f", n/1000000 }')"
  print "fixture_MiB=$(awk -v n="$FIXTURE_BYTES" 'BEGIN { printf "%.3f", n/1048576 }')"
  print "project_db_bytes=$PROJECT_DB_BYTES"
  print "shape=one synthetic Work; 5000 chapters; 10 live confirmed segments/chapter"
} > "$SCRIPT_DIR/fixture.txt"

print '== app release + probe production =='
"$SCRIPT_DIR/build.sh" 2>&1 | tee "$SCRATCH/build.log"
[[ -x "$APP_BIN" ]] || die "thiếu app release: $APP_BIN"

set_phase() {
  local phase="$1"
  case "$phase" in
    library|reading-full|reading-frontier|back-library|discard) ;;
    *) die "pha harness ngoài danh mục: $phase" ;;
  esac
  # Đổi file nguyên tử: command poll 20 ms, nên một `>` trực tiếp có thể để nó đọc đúng
  # khoảnh khắc rỗng rồi phán phase hỏng. `mv` cùng HOME nháp là rename nguyên tử trên APFS.
  local next="$PHASE_STATE.next-$$"
  printf '%s\n' "$phase" > "$next"
  mv -f "$next" "$PHASE_STATE"
}

set_phase library

monotonic_ns() {
  python3 -c 'import time; print(time.monotonic_ns())'
}

webkit_pids() {
  # `command=` làm dòng của CHÍNH `awk` chứa literal `com.apple.WebKit.` rồi tự nhận PID
  # ngắn ngủi ấy là WebKit. `comm=` chỉ chở executable path: đã đối chứng trên macOS 15.7.9
  # nó trả đúng ba XPC WebContent/GPU/Networking và không thể tự khớp awk/zsh.
  # `comm` so theo THỨ TỰ TỪ ĐIỂN; hai đầu vào phải cùng `LC_ALL=C sort`, không `sort -n`.
  # `sort -n` đã làm các PID cũ (623, 3472, …) bị báo nhầm là mới và cộng thành 3,6 GB.
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
  sqlite3 "$GLOBAL_DB" "DELETE FROM config_value WHERE kind='app_config' AND key LIKE '__5_14_%'; \
    INSERT INTO config_value(kind,key,value,updated_at) VALUES('app_config','mode','library',strftime('%Y-%m-%dT%H:%M:%fZ','now')) \
    ON CONFLICT(kind,key) DO UPDATE SET value='library', updated_at=excluded.updated_at;"
}

clear_library_index() {
  case "$APPDATA" in
    /tmp/auratranslate-5-14-*/home/Library/Application\ Support/com.auratranslate.desktop)
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
      invalid="$(sqlite3 -readonly "$GLOBAL_DB" "SELECT value FROM config_value WHERE kind='app_config' AND key='__5_14_invalid__';" 2>/dev/null || true)"
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
  HOME="$BENCH_HOME" "$APP_BIN" >> "$app_log" 2>&1 &
  ACTIVE_APP_PID=$!
  if ! marker="$(wait_marker '__5_14_usable__' 180)"; then
    printf '%s\t%s\t%s\t\tunknown\tusable marker vắng hoặc invalid\n' "$session" "$fixture" "$temperature" >> "$STARTUP_RAW"
    stop_app
    return 1
  fi
  ended_ns="$(monotonic_ns)"
  elapsed="$(awk -v a="$started_ns" -v b="$ended_ns" 'BEGIN { printf "%.3f", (b-a)/1000000 }')"
  node -e 'const v=JSON.parse(process.argv[1]); if(v.works!==1 || v.work_name!=="5.14 Fixture") process.exit(2)' "$marker" \
    || die "usable marker không khớp fixture: $marker"
  kill -0 "$ACTIVE_APP_PID" 2>/dev/null || die 'app chết ngay sau usable'
  printf '%s\t%s\t%s\t%s\tok\tpre-spawn tới marker grid có đúng fixture\n' \
    "$session" "$fixture" "$temperature" "$elapsed" >> "$STARTUP_RAW"

  if [[ "$keep_alive" == yes ]]; then
    ACTIVE_BASELINE="$baseline"
    ACTIVE_WEBKIT_EXPECTED="$SCRATCH/webkit-expected-${session}-${fixture}-${temperature}.txt"
    new_webkit_pids "$ACTIVE_BASELINE" > "$ACTIVE_WEBKIT_EXPECTED"
    [[ -s "$ACTIVE_WEBKIT_EXPECTED" ]] || die 'usable không sinh WebKit mới; app PID đơn lẻ bị từ chối'
  else
    # Thả command chờ của app cold trước khi giết tiến trình; response tới WebContent đã
    # chết không được dùng làm một marker, nên runner đặt pha discard rồi mới đóng.
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
  # 15.000 ms là một suy đoán của harness, không phải ngưỡng NFR: lượt 2026-09-02 đo 10 mẫu
  # Library đủ PID/footprint nhưng hết 15.095 ms chỉ vì `/usr/bin/footprint` bị máy khác tranh
  # CPU. 60.000 ms chỉ là trần liveness để một tiến trình thật sự treo không giữ lượt đo mãi;
  # nó không nới bất kỳ phán quyết nào vì từng hàng vẫn phải `ok`, đủ 4 PID và đủ 10 mẫu.
  [[ "$elapsed_ms" -lt 60000 ]] \
    || die "pha $session/$fixture/$phase mất ${elapsed_ms} ms, vượt trần liveness 60.000 ms"
}

measure_memory_session() {
  local session="$1" fixture="$2" baseline="$3" marker expected_status
  expected_status=content
  [[ "$fixture" != frontier ]] || expected_status=frontier-only

  sample_phase "$session" "$fixture" library "$baseline" "$ACTIVE_WEBKIT_EXPECTED"
  set_phase "reading-$fixture"
  # Bấm giờ lượt chuyển pha và ghi elapsed vào `transition-raw.tsv` ở CẢ hai nhánh. Một lượt
  # đỏ mà không để lại con số thì lần sau vẫn chỉ biết "vượt trần", đúng chỗ hổng của lượt 19:15.
  local t0_ns t1_ns t_ms
  t0_ns="$(monotonic_ns)"
  if marker="$(wait_marker '__5_14_reading__' "$PHASE_BUDGET_S")"; then
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
    "$marker" "$expected_status" "$([[ "$fixture" == full ]] && print 50000 || print 0)" \
    "$([[ "$fixture" == full ]] && print end-of-work || print next-not-done)" \
    || die "Reading marker sai fixture $fixture: $marker"
  sample_phase "$session" "$fixture" reading "$baseline" "$ACTIVE_WEBKIT_EXPECTED"
  set_phase back-library
  t0_ns="$(monotonic_ns)"
  if marker="$(wait_marker '__5_14_back_library__' "$PHASE_BUDGET_S")"; then
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
  node -e 'const v=JSON.parse(process.argv[1]); if(v.works!==1 || v.work_name!=="5.14 Fixture") process.exit(2)' "$marker" \
    || die "Library marker sai fixture $fixture: $marker"
  sample_phase "$session" "$fixture" back_library_keepalive "$baseline" "$ACTIVE_WEBKIT_EXPECTED"
  stop_app
}

set_fixture_status() {
  local target_status="$1"
  [[ "$target_status" == done || "$target_status" == not_started ]] \
    || die "status fixture ngoài danh mục: $target_status"
  [[ "$PROJECT_DB" == /tmp/auratranslate-5-14-* ]] || die "từ chối sửa DB ngoài HOME nháp: $PROJECT_DB"
  local changed
  changed="$(sqlite3 "$PROJECT_DB" "UPDATE chapter SET status='$target_status', updated_at='2026-09-01T00:00:00.000Z'; \
    SELECT COUNT(*) FROM chapter WHERE status='$target_status';")"
  [[ "$changed" == 5000 ]] \
    || die "không đặt đủ 5.000 Chương về $target_status"
}

print "== NFR4/NFR5: session [$SESSION_LIST] x fixture [$FIXTURE_LIST], trần pha ${PHASE_BUDGET_S}s =="
for session in ${=SESSION_LIST}; do
  if [[ "$FIXTURE_LIST" == *full* ]]; then
  set_fixture_status done
  launch_to_usable "$session" full cold no
  launch_to_usable "$session" full warm yes
  # Baseline bằng tập hiện tại làm hiệu WebKit rỗng: ca phải đỏ, chứng minh PID app đơn lẻ
  # không thể lọt thành một mẫu hợp lệ. Hàng tự kiểm bị bỏ khỏi dữ liệu đo ngay sau đó.
  webkit_pids > "$SCRATCH/all-webkit-now.txt"
  : > "$SCRATCH/empty-webkit.txt"
  sample_memory_once "$session" full app_pid_only_guard 0 "$SCRATCH/all-webkit-now.txt" "$SCRATCH/empty-webkit.txt"
  [[ "$(tail -1 "$MEMORY_RAW" | awk -F '\t' '{print $10}')" == error ]] \
    || die 'hàng rào app-PID-only không tự kiểm đỏ'
  sed -i '' '$d' "$MEMORY_RAW"
  measure_memory_session "$session" full "$ACTIVE_BASELINE"
  fi

  if [[ "$FIXTURE_LIST" == *frontier* ]]; then
  set_fixture_status not_started
  launch_to_usable "$session" frontier warm yes
  measure_memory_session "$session" frontier "$ACTIVE_BASELINE"
  fi
done

ENDED_AT="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
{
  print "measurement_started_utc=$STARTED_AT"
  print "measurement_ended_utc=$ENDED_AT"
  print "baseline_commit=$(git rev-parse HEAD)"
  print "working_diff_sha256=$( { git diff --binary HEAD; git ls-files --others --exclude-standard | LC_ALL=C sort | while IFS= read -r item; do printf '%s\\0' "$item"; shasum -a 256 "$item"; done; } | shasum -a 256 | awk '{print $1}')"
  print "release_app_sha256=$(shasum -a 256 "$APP_BIN" | awk '{print $1}')"
  print 'product_tree_guard=only Story 5.14 tests/artifacts/tracking plus feature-gated phase command allowed'
  print 'profile=release'
  print 'sessions=3'
  print 'nfr3_warmups_per_case=10'
  print 'nfr3_samples_per_case=200'
  print 'nfr5_idle_samples_per_phase=10'
  print "phase_liveness_budget_s=$PHASE_BUDGET_S"
  print "sessions_run=$SESSION_LIST"
  print "fixtures_run=$FIXTURE_LIST"
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
  print 'phase_control=feature-gated Tauri command persists a whitelisted marker then native-evals product tabs/DOM from HOME/.auratranslate-5-14-phase; full proves content+50000 segments+end-of-work, frontier proves frontier-only+0 segments+next-not-done; absent from default build; no network/CSP/ATS override'
} > "$SCRIPT_DIR/environment.txt"

node "$SCRIPT_DIR/summarize.mjs"
print "\nĐÃ XONG — raw data và REPORT.md ở $SCRIPT_DIR"
