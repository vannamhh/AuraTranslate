---
title: '⌥⌘F đưa con trỏ vào ô tìm của Library'
type: 'bugfix'
ticket: ''
created: '2026-10-25'
status: 'built'
baseline_revision: 'aac99510cddc9a58225729cf28ca2825d8e27df2'
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

**Problem:** `library.search` ("Tìm kiếm trong Library", mặc định `Mod+Alt+F`) chỉ chạy lượt tìm theo chữ đang có trong ô (`runLibrarySearch`), nên bấm ⌥⌘F không đưa con trỏ vào ô tìm; ô trống thì phím không làm gì.

**Approach (Ice chọn 2026-10-25, giữa hai phương án "lệnh mới" và "sửa library.search tại chỗ"):** thêm lệnh `library.focus_search` mang nhãn "Tìm kiếm trong Library" và phím mặc định `Mod+Alt+F`: chuyển sang Library nếu đang ở chế độ khác, đóng trang Tác phẩm nếu đang mở, đặt con trỏ vào ô tìm và bôi đen chữ cũ. `library.search` giữ đúng việc chạy tìm (Enter trong ô, nút bỏ lọc Tác phẩm), đổi nhãn thành "Chạy tìm kiếm Library", không còn phím mặc định.

</frozen-after-approval>

## Implementation Notes

Oneshot: khoảng 60 dòng ở bảy tệp, mỗi tệp một chỗ.

- Thứ tự tiêu điểm: `onActivated` (`enterFocus` + thẻ đang chọn) cũng giành tiêu điểm ở `nextTick`, nên nó không giành lại khi ô tìm đang giữ tiêu điểm (gỡ ⇒ ca "from another mode" đỏ). Lượt đóng trang Tác phẩm KHÔNG cần guard: `requestLibrarySearchFocus` đóng trang TRƯỚC khi bật cờ, nên tick đặt về thẻ luôn chạy trước; đảo hai dòng ⇒ ca "with a Work page open" đỏ.
- Dây `main.ts` → `focusLibrarySearch` không ca vitest nào chạm (main.ts không mount trong test); nghiệm thu tay của Ice phủ nó.
- Màn nhập (URL / xem trước) đang mở thì ô tìm không được vẽ: lệnh vẫn chuyển sang Library nhưng không đóng màn nhập, vì màn đó giữ bản xem trước chưa lưu; cờ chờ bị xoá để không giành tiêu điểm muộn.
- Dòng gợi ý `{chord} tìm kiếm` dưới kết quả (`LibrarySearchResults.vue`) nay đọc phím của `library.focus_search`; giữ nguyên đọc `library.search` thì dòng đó biến mất.

## Verification

**Commands:**
- `npx vitest run tests/frontend/libraryTopBar.test.ts tests/frontend/librarySearchScope.test.ts` -- expected: xanh; ca mới đỏ khi gỡ lời gọi focus khỏi LibraryMode.
- `npm run -s check:commands` -- expected: exit 0.

**Manual checks (if no CLI):**
- Ice bấm ⌥⌘F ở Library, ở Workspace, và khi đang mở trang một Tác phẩm: con trỏ vào ô tìm, chữ cũ được bôi đen.

## Review Triage Log

- low, reject: một ghi đè đã lưu cho `library.search` có thể đụng `Mod+Alt+F` mặc định của lệnh mới — `global.db` của Ice có 0 hàng kind `shortcut` (đo 2026-10-25), và lớp đĩa xung đột đi qua `shortcutsDiskRejection` có câu báo, không im lặng.
- medium, patch: đường "trang Tác phẩm đang mở" không ca nào chạm — thêm ca; đo ra guard ở watcher đóng trang là thừa (gỡ vẫn xanh) nên đã bỏ, ca mới khoá thứ tự đóng-trước-bật-cờ (đảo ⇒ đỏ).
