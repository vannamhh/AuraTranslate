#!/usr/bin/env bash
#
# Kiểm 3 của Story 1.2 — AC3: phạm vi filesystem tĩnh, cưỡng chế bởi Tauri.
#
# Chạy ứng dụng thật, để `src/selftest/scopeCheck.ts` thử HAI CHIỀU trong webview:
#   - trong scope  → `$RESOURCE/fonts/**` nạp THÀNH CÔNG
#   - ngoài scope  → `/etc/hosts` (macOS) / `C:\Windows\win.ini` (Windows) BỊ TỪ CHỐI
#
# ⚠️ **Vì sao cần lớp bọc này thay vì chỉ `app.exit(1)` trong Rust.** Đã đo thật
# 2026-08-03: `tauri dev` **nuốt mã thoát của ứng dụng** — app thoát 1, `npm run
# check:scope` vẫn trả 0. Một phép kiểm luôn trả 0 là phép kiểm không cưỡng chế được
# gì, đúng thứ §Testing standards của story cấm. Nên phán quyết đọc từ dòng `VERDICT:`
# mà chính self-check in ra, và mã thoát do script này quyết.
#
# Story 1.3 gắn thẳng script này vào pipeline — KHÔNG dựng pipeline thứ hai.
#
# Chạy:  npm run check:scope

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOG="$(mktemp -t aura-scope-check)"

cleanup() { rm -f "$LOG"; }
trap cleanup EXIT

echo "Kiểm 3 — phạm vi asset protocol, hai chiều (AC3)"
echo

cd "$REPO_ROOT"
AURA_SCOPE_SELFTEST=1 VITE_SCOPE_SELFTEST=1 npx tauri dev 2>&1 | tee "$LOG"

VERDICT_LINE="$(grep -E '^VERDICT: (PASS|FAIL)$' "$LOG" | tail -n 1)"

echo
if [ -z "$VERDICT_LINE" ]; then
  printf '\033[31mKhông tìm thấy dòng VERDICT trong log.\033[0m\n'
  echo "Self-check chưa chạy tới nơi — cửa sổ không mở được, hoặc frontend gãy trước"
  echo "khi phát event. Đọc log ở trên; đừng coi đây là 'đạt'."
  exit 1
fi

if [ "$VERDICT_LINE" = "VERDICT: PASS" ]; then
  printf '\033[32mKiểm 3 đạt — cả chiều cho phép lẫn chiều từ chối.\033[0m\n'
  exit 0
fi

printf '\033[31mKiểm 3 THẤT BẠI.\033[0m\n'
grep -E '^\[(PASS|FAIL)\]' "$LOG" | sed 's/^/  /'
exit 1
