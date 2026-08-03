#!/usr/bin/env bash
#
# Kiểm 1 + Kiểm 2 của Story 1.2 — cưỡng chế AC2 và nửa "không crash reporter,
# không analytics" của AC5, bằng lệnh, trên CẢ HAI cây phụ thuộc.
#
# Vì sao là script chứ không phải soát bằng mắt: hai điều kiện này chỉ đúng vào ngày
# ai đó nhìn. Một story sau cài `tauri-plugin-fs` "cho tiện" thì không có gì báo.
#
# ⚠️ Script này PHẢI trả mã thoát khác 0 khi thất bại. Một script in ra cảnh báo rồi
# trả 0 là script không cưỡng chế được gì.
#
# Story 1.3 gắn thẳng script này vào pipeline — KHÔNG dựng pipeline thứ hai.
#
# Chạy:  npm run check:deps        (hoặc ./scripts/check-deps.sh)

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CARGO_MANIFEST="$REPO_ROOT/src-tauri/Cargo.toml"
FAILURES=0

pass() { printf '  \033[32mOK\033[0m   %s\n' "$1"; }
fail() { printf '  \033[31mFAIL\033[0m %s\n' "$1"; FAILURES=$((FAILURES + 1)); }

# ─────────────────────────────────────────────────────────────────────────────────
# Kiểm 1 — bốn phụ thuộc đã loại phải VẮNG MẶT trong cây phụ thuộc (AC2)
#
# Ba tên đầu là nguyên văn AC2. `tauri-plugin-fs` là quyết định của Ice 2026-08-03,
# cùng hạng lý do (AD-1 + AD-29): plugin tồn tại để phơi API ra JavaScript, mà
# frontend chỉ render và giữ state UI — nó không có việc gì với hệ thống file.
# ─────────────────────────────────────────────────────────────────────────────────
echo
echo "Kiểm 1 — phụ thuộc đã loại phải vắng mặt (AC2)"

BANNED_CRATES=(
  tauri-plugin-stronghold   # đã khai tử
  tauri-plugin-keyring      # AD-29 — dùng crate `keyring` trực tiếp
  tauri-wire                # payload 679 byte
  tauri-plugin-fs           # AD-1 + AD-29 — Ice chốt 2026-08-03
  tauri-plugin-sql          # AD-11 — dùng `rusqlite` trực tiếp
  tauri-plugin-dialog       # cùng lý do: không phơi filesystem ra JS
)

for crate in "${BANNED_CRATES[@]}"; do
  if cargo tree --manifest-path "$CARGO_MANIFEST" -i "$crate" >/dev/null 2>&1; then
    fail "crate \`$crate\` CÓ MẶT trong cây phụ thuộc Rust"
    cargo tree --manifest-path "$CARGO_MANIFEST" -i "$crate" 2>&1 | sed 's/^/       /'
  else
    pass "crate \`$crate\` vắng mặt"
  fi
done

BANNED_NPM=(
  @tauri-apps/plugin-fs
  @tauri-apps/plugin-sql
  @tauri-apps/plugin-dialog
  @tauri-apps/plugin-stronghold
)

for pkg in "${BANNED_NPM[@]}"; do
  if [ -d "$REPO_ROOT/node_modules/$pkg" ]; then
    fail "gói npm \`$pkg\` CÓ MẶT trong node_modules"
  else
    pass "gói npm \`$pkg\` vắng mặt"
  fi
done

# ─────────────────────────────────────────────────────────────────────────────────
# Kiểm 2 — không crash reporter, không analytics, trên CẢ HAI cây (AC5)
#
# ⚠️ `segment-io` khác `segment`. Module Rust `core/segment/` của chính dự án tên là
# *segment* — mẫu quét phải bắt thư viện thật mà không tự báo động vào chính mình.
# ─────────────────────────────────────────────────────────────────────────────────
echo
echo "Kiểm 2 — không crash reporter, không analytics (AC5)"

PATTERN='sentry|bugsnag|rollbar|crashlytics|datadog|newrelic|posthog|amplitude|mixpanel|segment-io|telemetry|analytics|opentelemetry|google-analytics|firebase'

RUST_TREE="$(cargo tree --manifest-path "$CARGO_MANIFEST" --prefix none --no-dedupe 2>/dev/null | sort -u)"
RUST_HITS="$(printf '%s\n' "$RUST_TREE" | grep -Ei "$PATTERN" || true)"
if [ -n "$RUST_HITS" ]; then
  fail "cây Rust có thư viện thu thập dữ liệu:"
  printf '%s\n' "$RUST_HITS" | sed 's/^/       /'
else
  pass "cây Rust sạch ($(printf '%s\n' "$RUST_TREE" | grep -c . ) mục đã quét)"
fi

NPM_TREE="$(npm ls --all --parseable --prefix "$REPO_ROOT" 2>/dev/null | sort -u)"
NPM_HITS="$(printf '%s\n' "$NPM_TREE" | grep -Ei "$PATTERN" || true)"
if [ -n "$NPM_HITS" ]; then
  fail "cây npm có thư viện thu thập dữ liệu:"
  printf '%s\n' "$NPM_HITS" | sed 's/^/       /'
else
  pass "cây npm sạch ($(printf '%s\n' "$NPM_TREE" | grep -c . ) mục đã quét)"
fi

# NFR13 — không tài khoản, không đăng nhập, không đồng bộ đám mây. Cùng cách nghiệm
# thu: bằng VẮNG MẶT. Không SDK auth, không client đồng bộ trong cả hai cây.
AUTH_PATTERN='auth0|okta|firebase-auth|supabase|clerk|cognito|oauth-client|dropbox|googleapis|onedrive|icloud'
AUTH_HITS="$( { printf '%s\n' "$RUST_TREE"; printf '%s\n' "$NPM_TREE"; } | grep -Ei "$AUTH_PATTERN" || true)"
if [ -n "$AUTH_HITS" ]; then
  fail "có SDK tài khoản / đồng bộ đám mây (NFR13):"
  printf '%s\n' "$AUTH_HITS" | sed 's/^/       /'
else
  pass "không SDK tài khoản, không client đồng bộ đám mây (NFR13)"
fi

# ─────────────────────────────────────────────────────────────────────────────────
echo
if [ "$FAILURES" -ne 0 ]; then
  printf '\033[31m%d phép kiểm thất bại.\033[0m\n' "$FAILURES"
  exit 1
fi
printf '\033[32mTất cả phép kiểm phụ thuộc đạt.\033[0m\n'
echo
echo "Ghi chú cho người rà soát: \`reqwest\` CÓ trong cây phụ thuộc và đó KHÔNG phải"
echo "vi phạm AC5. Bảng Stack cài trọn ở Story 1.2, nhưng chưa một dòng mã nào gọi tới."
echo "AC5 nói 'không có LỜI GỌI ra ngoài nào' — một crate không được gọi thì không gọi"
echo "đi đâu cả. Ba điểm ra mạng của AD-15 mở ở Story 4.x, 6.7, 10.7."
exit 0
