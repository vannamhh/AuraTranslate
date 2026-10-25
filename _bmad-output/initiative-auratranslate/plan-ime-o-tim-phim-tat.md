---
title: 'Ô tìm phím tắt không ăn phím bỏ dấu của bộ gõ tiếng Việt'
type: 'bugfix'
ticket: ''
created: '2026-10-25'
status: 'built'
baseline_revision: 'bfe8334f008e66ce2b8bdbcdf2d7c5e43f7a170a'
route: 'oneshot'
route_source: 'auto'
risk: 'low'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Gõ tiếng Việt vào ô tìm của Cài đặt › Phím tắt, lúc bộ gõ thêm dấu thì chữ đã gõ bị xoá, ô hiện tổ hợp `A` và câu "Không lệnh nào dùng A.". Bộ gõ phát một `keydown` không mang `isComposing`, `code` vật lý là phím chữ (`KeyA`) mà `key` không phải một ký tự đơn (chuỗi nhiều ký tự hoặc `Unidentified`), nên `handleSearchKey` (`src/config/shortcutsState.ts`) coi nó là tổ hợp tra ngược, `preventDefault()` và đặt `searchText = ''`.

**Approach:** Không kèm Mod/Ctrl/Alt, một phím có `code` in được (cùng tập `PRINTABLE_CODE_RE` mà luật vùng gõ ở `src/commands/keys.ts` đã dùng) luôn là chữ, bất kể `key` mang gì. Tra ngược bằng phím trần chỉ còn cho phím không in được (F-key, mũi tên…).

</frozen-after-approval>

## Code Map

- `src/config/shortcutsState.ts::handleSearchKey` -- chỗ sửa: thêm vế `code` in được vào nhánh "vẫn là chữ".
- `src/commands/keys.ts::PRINTABLE_CODE_RE` -- tái dùng qua một hàm export `isPrintableCode`, đi ra ngoài qua `src/commands/index.ts` như `chordFromEvent`.
- `tests/frontend/shortcutsSearchExchange.test.ts` -- thêm ca `KeyA` với `key` không phải một ký tự.

## Implementation Notes

Oneshot: ~15 dòng ở ba tệp, một nhánh điều kiện và một ca test.
- Đối chứng đỏ: ca mới đỏ ở `key` NFD `e\u0302\u0301` trước khi có vế `isPrintableCode`; `ế` dựng sẵn dài 1 nên vốn đã qua nhánh cũ.
- `key` thật mà bộ gõ của Ice phát chưa đo; bản sửa dựa vào `code` (ảnh chụp chứng minh `code` là `KeyA`), nên đúng với mọi giá trị `key`.

## Verification

**Commands:**
- `npx vitest run tests/frontend/shortcutsSearchExchange.test.ts` -- expected: xanh; đỏ khi gỡ vế `isPrintableCode` khỏi `handleSearchKey`.

**Manual checks (if no CLI):**
- Ice gõ "tiếng" bằng bộ gõ thường dùng vào ô tìm phím tắt: chữ giữ nguyên, không hiện tổ hợp.

## Review Triage Log

- low, reject: Numpad với NumLock tắt (Windows) nay là chữ thay vì tra ngược `Numpad4`; không lệnh nào gán phím Numpad, macOS không có NumLock.
- low, patch: hai `ế` trông như trùng trong nguồn; đã viết thành escape `ế` / `ế`.
- false: "không có ca phím trần không in được còn tra ngược" — ca `F3` trần ⇒ `true` đã có ở `shortcutsSearchExchange.test.ts` (it "phím không in được…").
- false: nghiệm thu tay chưa làm — đó là bước của Ice sau commit, không phải lỗi mã.
