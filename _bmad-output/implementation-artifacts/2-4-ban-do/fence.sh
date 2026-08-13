#!/bin/zsh
# Hàng rào dữ liệu thật của Story 2.4 — Quyết định #1 đường (d).
#
# CHIỀU ÂM: ~/Documents/AuraTranslate/ và $APPDATA thật phải Y NGUYÊN TỪNG BYTE sau cả bộ đo.
# CHIỀU DƯƠNG: mọi .atproj/global.db sinh ra phải nằm trong HOME nháp.
#
# Khuôn lấy từ `onComplete` của e2e/wdio.conf.mjs (Story 1.22 C2).
# Dùng: fence.sh snap <nhãn>   |   fence.sh diff <nhãn-a> <nhãn-b>   |   fence.sh positive
set -u

SCRATCH="${0:A:h}"
REAL_DOCS="$HOME/Documents/AuraTranslate"
REAL_APPDATA="$HOME/Library/Application Support/com.auratranslate.desktop"
DRAFT_HOME="$SCRATCH/draft-home"

snap() {
  local label="$1"
  local out="$SCRATCH/fence-$label.txt"
  {
    echo "# snapshot: $label  @ $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "## $REAL_DOCS"
    # Băm nội dung, không chỉ mtime: một lượt ghi rồi ghi lại y hệt vẫn phải lộ ra ở mtime,
    # còn một lượt đổi nội dung giữ nguyên mtime thì chỉ băm mới bắt được. Lấy CẢ HAI.
    if [ -d "$REAL_DOCS" ]; then
      find "$REAL_DOCS" -type f -print0 2>/dev/null | sort -z | while IFS= read -r -d '' f; do
        printf '%s  %s  %s\n' "$(shasum -a 256 "$f" | cut -d' ' -f1)" "$(stat -f '%z %m' "$f")" "${f#$HOME/}"
      done
    else
      echo "(vắng mặt)"
    fi
    echo "## $REAL_APPDATA"
    if [ -d "$REAL_APPDATA" ]; then
      find "$REAL_APPDATA" -type f -print0 2>/dev/null | sort -z | while IFS= read -r -d '' f; do
        printf '%s  %s  %s\n' "$(shasum -a 256 "$f" | cut -d' ' -f1)" "$(stat -f '%z %m' "$f")" "${f#$HOME/}"
      done
    else
      echo "(vắng mặt)"
    fi
  } > "$out"
  echo "snap → $out  ($(grep -c . "$out") dòng)"
}

diffsnap() {
  if diff -u "$SCRATCH/fence-$1.txt" "$SCRATCH/fence-$2.txt" | grep -vE '^[+-]# snapshot' | grep -qE '^[+-][^+-]'; then
    echo "🔴 HÀNG RÀO CHIỀU ÂM VỠ — dữ liệu thật đã đổi:"
    diff -u "$SCRATCH/fence-$1.txt" "$SCRATCH/fence-$2.txt" | grep -E '^[+-][^+-]'
    return 1
  fi
  echo "🟢 hàng rào chiều âm ĐỨNG — $REAL_DOCS và \$APPDATA thật y nguyên từng byte"
  return 0
}

positive() {
  echo "## chiều dương — mọi tạo tác phải nằm trong $DRAFT_HOME"
  local found_in_draft
  found_in_draft=$(find "$DRAFT_HOME" \( -name 'project.db' -o -name 'global.db' -o -name '*.atproj' \) 2>/dev/null | wc -l | tr -d ' ')
  echo "trong HOME nháp: $found_in_draft tạo tác"
  find "$DRAFT_HOME" \( -name 'project.db' -o -name 'global.db' \) 2>/dev/null | sed "s|$DRAFT_HOME|\$DRAFT_HOME|"
  if [ "$found_in_draft" -eq 0 ]; then
    echo "🔴 CHIỀU DƯƠNG VỠ — không tạo tác nào trong HOME nháp; app đã ghi Ở CHỖ KHÁC"
    return 1
  fi
  echo "🟢 chiều dương ĐỨNG"
  return 0
}

case "${1:-}" in
  snap)     snap "$2" ;;
  diff)     diffsnap "$2" "$3" ;;
  positive) positive ;;
  *) echo "dùng: fence.sh snap <nhãn> | fence.sh diff <a> <b> | fence.sh positive"; exit 2 ;;
esac
