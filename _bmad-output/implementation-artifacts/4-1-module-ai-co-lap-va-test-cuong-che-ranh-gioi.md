---
title: 'Story 4.1 — Module `ai/` cô lập và test cưỡng chế ranh giới'
type: 'feature'
created: '2026-08-26'
status: 'done'
review_loop_iteration: 0
baseline_commit: 'e663705738dbf62ee5f5e5805542e95fa01709f7'
context:
  - '{project-root}/_bmad-output/implementation-artifacts/epic-4-context.md'
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/tests/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** AD-13 — *"không module nào ngoài `ai/` được phụ thuộc `ai/`"* — là điều kiện để FR77 không thoái hoá thành kỷ luật cá nhân, và hôm nay **chưa ai canh nó**. `core/ai/mod.rs` tự khai *"Test cưỡng chế ranh giới này thuộc Story 4.1"*, và `project-context.md` đã phải sửa mệnh đề *"Có test cưỡng chế"* thành **chưa có** ngày 2026-08-19. Ranh giới này rẻ nếu dựng từ dòng mã đầu tiên và rất đắt nếu vá sau — đó chính là lý do Story 4.1 được tách ra chạy ở thứ tự 3½ thay vì lùi cùng Epic 4 (32 story sẽ được viết trước khi ranh giới có người canh).

**Approach:** Dựng `src-tauri/tests/ai_boundary.rs` theo đúng khuôn sáu tệp `*_boundary.rs` đã có. Vì `core/ai/` hôm nay có **0 dòng mã** và **0 chỗ gọi từ bên ngoài**, đối chứng dương kiểu *"module thật sự mang từ vựng của nó"* (khuôn `scope_boundary`/`matching_boundary`) **dựng không được** — đối chứng bắt buộc ở đây là **ca gieo vi phạm tổng hợp**: gọi thẳng vị từ trên chuỗi dựng tay để chứng minh nó bắt được, độc lập với việc cây hôm nay có gì. Song song, đóng nốt vế còn thiếu của AC panel bằng một lượt sửa giá trị `panel.ai_translation.status` và di trú `.status` sang token `ui-md-wrap` (Ice chốt 2026-08-26). Năm AC còn lại của story **đúng một cách rỗng** hôm nay: nghiệm thu bằng ảnh chụp nền có số cộng nợ có chủ, không tự chấm đạt.

## Boundaries & Constraints

**Always:**
- 🔴 **Đối chứng GỠ-CHỖ-NỐI cho mỗi phép kiểm mới:** gieo một vi phạm thật vào cây (thêm `use crate::core::ai;` vào một tệp ngoài `core/ai/`) thì ca phải **ĐỎ**; gỡ ra thì **XANH**. §Completion Notes ghi **tên ca** và **số ca đỏ thật**, rồi khôi phục tệp về nguyên trạng và xác nhận bằng `diff`.
- 🔴 **Ca gieo vi phạm TỔNG HỢP là bắt buộc**, không thay thế được bằng lượt gỡ-chỗ-nối ở trên: nó gọi vị từ trên chuỗi dựng tay (một chuỗi vi phạm ⇒ bắt; một chuỗi sạch ⇒ không bắt), chứng minh vị từ nổ được **độc lập với cây hiện tại**. Đây là thứ phân biệt *"không ai vi phạm"* với *"không có gì để vi phạm"*.
- 🔴 **Quét BARE token, không chỉ dạng có tiền tố `use `.** Một lời gọi đủ điều kiện viết thẳng trong thân hàm (`crate::core::ai::foo()`) không có dòng `use` nào — đúng khuyết tật `MATCHING_FORBIDDEN_USES` đã bị bắt ở một lượt review trước. Phủ cả `super::ai`.
- **Sàn quần thể đo LẠI, không chép:** hôm nay `src-tauri/src/**` có **55** tệp `.rs` (bốn tệp boundary cũ còn ghi 53 — số đã trôi). `core/ai/` có **1** tệp.
- Chuẩn hoá đường dẫn qua `rel_posix` (`\` → `/`) — thiếu nó thì miễn trừ `core/ai` ngừng khớp trên Windows CI và chính `mod.rs` tự tố cáo mình. `walk` dùng `fs::symlink_metadata`, không `metadata`.
- Tên hàm test là một **CÂU khẳng định**. Thông báo `assert!` viết tiếng Việt **có dấu** — `src-tauri/tests/**` là miễn trừ CÓ TÊN trong `EXEMPT` của `check-i18n`.
- 🔴 **Năm AC rỗng giữ nguyên trong story, không sửa `epics.md`.** *"Năng lực chưa dựng ≠ lệch spec"* — ghi ảnh chụp nền có số và một mục nợ **có chủ**, không tự chấm đạt.
- Câu `panel.ai_translation.status` mới giữ giọng **MỜI**, không giọng CẢNH BÁO; qua Kiểm D (`check-i18n`) cấm `chúng tôi`/`bạn`.

**Ask First:**
- 🔴 **Phép quét bắt được một vi phạm THẬT đang tồn tại trong cây ⇒ DỪNG và trình danh sách.** Kỳ vọng là 0; một kết quả khác 0 nghĩa là AD-13 đã bị phá từ trước và đó là quyết định phạm vi của Ice, không phải một lượt vá tiện tay.
- Nếu di trú `.status` sang `ui-md-wrap` làm đỏ bất kỳ cổng nào (`check:tokens`, `check:panel-refs`) hoặc một ca vitest: trình lỗi, đừng nới cổng.

**Never:**
- **Không viết một dòng mã sản phẩm nào vào `core/ai/`.** Nó ở lại stub doc-comment. Chiều ngược (`ai/` đọc `glossary/`/`tm/`/`segment/`) là Story 4.6.
- **Không tách `ai/` thành crate riêng** (Ice chốt 2026-08-26: cổng tĩnh). Đó là một `AD` mới, không phải một dòng mã.
- **Không gọi `keyring` hay `reqwest`.** Hai crate đã ghim trong `Cargo.toml` với **0 chỗ gọi**; lượt này giữ nguyên con số đó. Không thêm phụ thuộc nào (NFR15).
- Không đụng `src-tauri/src/ports/**` — `TranslationProvider` khai *"chưa khai, Epic 4"* và cắm nó là Story 4.2 trở đi.
- Không thêm khoá `vi.json` mới, không token mới, không sửa `DESIGN.md`, không thêm mục `deviations`.
- Không đổi `[profile.release]`, không đổi `EXPECTED_COUNTS` của `check-tokens`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Cây sạch (hôm nay) | 55 tệp `.rs`, 0 tệp ngoài `core/ai/` nhắc `core::ai` | Mọi ca xanh | N/A |
| Gieo `use crate::core::ai;` vào một module khác | 1 tệp vi phạm | Ca ranh giới ĐỎ, nêu đích danh `file:line` | N/A |
| Gieo lời gọi đủ điều kiện trong thân hàm, không `use` | `crate::core::ai::foo()` | Ca ranh giới ĐỎ — bare token bắt được | N/A |
| Gieo `super::ai::foo()` từ một module `core/*` khác | 1 tệp vi phạm | Ca ranh giới ĐỎ | N/A |
| Nhắc `core::ai` trong một dòng chú thích | `// core::ai sẽ ...` | **KHÔNG** đỏ — `code_lines` bỏ dòng bắt đầu bằng `//` | N/A |
| Gốc quét sai / thư mục bị cắt | `walk` khớp 0 tệp | Ca sàn quần thể ĐỎ kèm câu *"cây quá nhỏ để là thật"* | N/A |
| Chạy trên Windows | đường dẫn `\` | Miễn trừ `core/ai` vẫn khớp qua `rel_posix` | N/A |
| Vị từ trên chuỗi dựng tay | chuỗi vi phạm / chuỗi sạch | Bắt / không bắt — chứng minh vị từ nổ được | N/A |

</frozen-after-approval>

## Code Map

- `src-tauri/tests/ai_boundary.rs` — **tệp mới**, sản phẩm chính. Khuôn: `scope_boundary.rs` (307 dòng) là bản gần nhất về hình dạng; `matching_boundary.rs:the_warm_jieba_check_would_actually_flag_a_removed_call` là khuôn ca gieo tổng hợp; `glossary_boundary.rs` là tiền lệ *dựng cổng trước khi consumer tồn tại*.
- `src-tauri/tests/scope_boundary.rs` — **chỉ đọc**. Lấy nguyên `src_root()` · `rel_posix()` · `walk()` · `all_rust_sources()` · `code_lines()`; khối doc-comment §*"HÔM NAY CHƯA CÓ CONSUMER NÀO — VÀ ĐÓ KHÔNG LÀM PHÉP KIỂM NÀY THÀNH VÒNG"* áp thẳng được vào ca của ta.
- `src-tauri/tests/matching_boundary.rs` — **chỉ đọc**. `contains_forbidden_token` + `is_word_byte` (khớp theo biên từ, không phân biệt hoa thường), và doc-comment `MATCHING_FORBIDDEN_USES` giải thích vì sao phải quét **bare** token.
- `src-tauri/src/core/mod.rs` — **chỉ đọc**. Khai `pub mod ai;` trần, không re-export. ⚠️ Một `pub use ai::…` thêm vào đây về sau sẽ cho module khác viết `crate::core::Foo` mà không hề đánh vần `ai` — điểm mù có tên, ghi vào doc-comment.
- `src-tauri/src/core/ai/mod.rs` — **chỉ đọc, không sửa**. 10 dòng, 100 % doc-comment, 0 dòng mã.
- `src/i18n/vi.json:190` — sửa **giá trị** khoá `panel.ai_translation.status` (94 ký tự hiện tại). Không thêm khoá. Khoá này là chuỗi frontend thuần: `grep` `src-tauri/` cho **0 hit**, nó không thuộc danh mục đóng `message_keys!`.
- `src/panels/PanelFrame.vue:227-234` — đổi ba biến của `.status` sang họ `ui-md-wrap`. *(🔵 Sửa 2026-08-26 sau vòng rà 1: bản đầu ghi `:215-222`, trỏ vào khối chú thích chứ không vào ba dòng khai báo — người theo con trỏ sẽ đọc phần lý do rồi tưởng mình đã thấy chỗ sửa.)* CSS **scoped**; chỉ ba panel truyền `status-key` (AI Translation · Lookup · Grid). Cùng 13px ⇒ đổi duy nhất giãn dòng 1,5 → 1,66.
- `src/panels/AiTranslationPanel.vue:49` — **chỉ đọc**. `status-key` phải ở lại một chuỗi literal (Kiểm E của `check-commands` đọc tĩnh).
- `src/tokens/tokens.json:469-475` — **chỉ đọc**. `ui-md-wrap` đã tồn tại (token thứ 17, Story 1.17 Quyết định #7). Cổng đếm **định nghĩa**, không đếm chỗ dùng ⇒ `EXPECTED_COUNTS` không đổi.
- `_bmad-output/implementation-artifacts/deferred-work.md` — nối mục nợ mới ở **cuối tệp** (EOF hiện tại 7845). Khuôn: `- source_spec:` / `summary:` / `evidence:` / `**(Chủ: …)**`. Gate `check-debt-owner` đòi mục mở phải có `Chủ:` dương; hiện 580 mục, 379 mở, **0 mồ côi**, `ITEM_FLOOR` 490. Mục `:116` là món nợ `.status`/`ui-md` mà lượt này **đóng** — nối tiếp bằng `→ ✅`, không xoá.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/tests/ai_boundary.rs` — tạo mới: doc-comment theo khuôn (nêu rõ vì sao cổng đứng trước consumer, và vì sao đối chứng dương ở đây phải là ca gieo tổng hợp), helper chép từ `scope_boundary.rs`, hằng `AI_DIR`/`AI_FLOOR = 1`/`SRC_RS_FLOOR = 44` (44/55 = 80 %), danh sách token cấm dạng **bare** (`crate::core::ai`, `super::ai`) — rationale: đây là toàn bộ phần dựng được và đỏ được của story.
- [x] `src-tauri/tests/ai_boundary.rs` — ca sàn quần thể + ca ranh giới + **ca gieo vi phạm tổng hợp** + ca khẳng định `core/mod.rs` khai `ai` — rationale: bốn ca là bốn mệnh đề khác nhau; thiếu ca gieo thì ba ca kia xanh rỗng.
- [x] `src/i18n/vi.json` — sửa giá trị `panel.ai_translation.status`: giữ vế mời cấu hình, **thêm vế "mọi thứ khác vẫn chạy đầy đủ"**, giọng mời — rationale: đóng nốt nửa còn thiếu của AC panel thay vì chấm đạt bằng suy luận.
- [x] `src/panels/PanelFrame.vue` — `.status` đổi sang `--face-ui-md-wrap` / `--font-ui-md-wrap` / `--leading-ui-md-wrap` — rationale: Ice chốt 2026-08-26; câu vừa dài thêm mà giữ `ui-md` (`wraps: false`, 1,5) là làm nặng một món nợ đã ký thay vì đóng nó.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` — nối bốn mục có chủ: ① năm AC rỗng, kèm ảnh chụp nền có số (chủ: Story 4.2); ② chiều ngược `ai/` → `glossary/`/`tm/`/`segment/` chưa chứng minh được (chủ: Story 4.6); ③ AC ranh giới phải chạy LẠI trên bộ test Epic 5/6 khi 4.2 tới lượt (chủ: Story 4.2); ④ điểm mù re-export ở `core/mod.rs` (chủ: Story 4.2). Và nối `→ ✅` đóng mục `:116` — rationale: sổ nợ là bằng chứng cho quyết định kế tiếp; không mục nào mồ côi, không mục đã đóng nào bị xoá.

**Acceptance Criteria:**
- Given một tệp bất kỳ ngoài `core/ai/**` mang `crate::core::ai` hoặc `super::ai` ở **vị trí mã**, when chạy `cargo test --locked --test ai_boundary`, then ca ranh giới ĐỎ và nêu đích danh `file:line`.
- Given cây sạch như hôm nay, when chạy cùng lệnh, then mọi ca XANH **và** ca gieo tổng hợp vẫn chứng minh vị từ nổ được — hai mệnh đề tách rời, không suy ra nhau.
- Given `walk` khớp dưới sàn (gốc quét sai, thư mục bị cắt), when chạy, then ca sàn ĐỎ chứ không phải mọi ca xanh rỗng.
- Given panel AI Translation lúc chưa cấu hình, when hiển thị, then câu trạng thái vừa **mời cấu hình** vừa nói rõ **mọi năng lực khác chạy đầy đủ**, và không mang màu `error` nào.
- Given toàn bộ cổng và bộ test của kho, when chạy sau lượt vá, then `cargo test --locked` ≥ 717 ca / 0 đỏ, `npx vitest run` 567 ca / 0 đỏ, và mười một cổng `pre-push` exit 0.
- Given năm AC rỗng của story, when nghiệm thu, then chúng **không** được chấm đạt bằng suy luận — §Completion Notes ghi ảnh chụp nền có số và trỏ vào mục nợ có chủ tương ứng.

## Spec Change Log

## Design Notes

**Vì sao đối chứng dương ở đây khác bốn tệp boundary kia.** `scope_boundary.rs` chứng minh phép quét là thật bằng cách khẳng định `core/scope/**` **thật sự** mang `ScopeKind` ở ≥ 3 tệp; `matching_boundary.rs` khẳng định `core/matching/**` thật sự dùng cả hai crate ngôn ngữ. Cả hai dựa vào việc module chủ **có từ vựng để tìm**. `core/ai/mod.rs` có **0 dòng mã** — không từ vựng nào tồn tại, nên khuôn đó dựng không được và **đừng cố ép nó**. Thay vào đó dùng khuôn thứ hai, cũng đã có tiền lệ trong chính kho: gọi vị từ trên chuỗi dựng tay, đúng cách `the_warm_jieba_check_would_actually_flag_a_removed_call` và `the_non_manual_origin_token_check_catches_term_origin_but_not_candidate_origin` làm. Ghi lý do bỏ khuôn thứ nhất **vào doc-comment** thay vì để người sau tưởng là quên.

**Điểm mù có tên.** Phép quét bắt chuỗi `crate::core::ai` / `super::ai`. Nếu một ngày `core/mod.rs` thêm `pub use ai::Foo;` thì module khác viết được `crate::core::Foo` mà **không đánh vần `ai` một lần nào** — cổng này xanh trên một AD-13 đã bị phá. Ghi ra ở doc-comment, đúng cách bốn tệp kia ghi giới hạn của chúng, và giao thành một mục nợ có chủ.

## Verification

**Commands:**
- `cd src-tauri && cargo test --locked --test ai_boundary` — expected: toàn bộ ca mới xanh.
- `cd src-tauri && cargo test --locked` — expected: ≥ 717 ca, 0 đỏ (ảnh chụp nền trước lượt vá: 717 ca / 26 nhị phân).
- `npx vitest run` — expected: 43 tệp / 567 ca / 0 đỏ. ⚠️ Chạy **một mình**, không xen `cargo test` — chập chờn tranh CPU đã có mục nợ (cụm E).
- `npm run check:i18n` — expected: exit 0 (Kiểm B khoá phẳng + giá trị không rỗng, Kiểm D giọng UX-DR47).
- `npm run check:tokens && npm run check:commands && npm run check:panel-refs && npm run check:debt-owner` — expected: exit 0 cả bốn; `check:debt-owner` báo ≥ 584 mục tổng, **0 mục mở thiếu `Chủ:`**.
- `npm run build` — expected: thành công.

**Phép GỠ-CHỖ-NỐI (bắt buộc, ghi số vào §Completion Notes):**
- Thêm `use crate::core::ai;` vào một tệp ngoài `core/ai/` → chạy lại → ghi **tên ca đỏ** và **số ca đỏ thật** → khôi phục → `diff` xác nhận nguyên trạng → chạy lại xanh.
- Lặp cho dạng đủ điều kiện trong thân hàm (`crate::core::ai::foo()`, không `use`) và cho `super::ai::foo()`.

**Manual checks:**
- Mở `npm run tauri dev`, nhìn panel Đề xuất AI: câu trạng thái đọc như một **lời mời**, xuống dòng ở giãn dòng 1,66, không màu `error`. Nhìn luôn câu trạng thái của Lookup và Grid — hai chỗ cùng đổi theo `.status`.

## Completion Notes

⚠️ **Agent thi hành chết giữa chừng vì lỗi API** — nó ghi xong bốn tệp rồi dừng đúng lúc đang chạy nghiệm thu. Mọi con số dưới đây do người điều phối đo LẠI từ đầu, không lấy từ báo cáo của nó (nó chưa kịp viết báo cáo nào). Hai mệnh đề trong sổ nợ mà nó viết trước là **mô tả dự định chứ không phải phép đo** — đã sửa tại chỗ kèm 🔵, giữ nguyên văn chỗ sai.

**Bốn việc đã làm:**

1. **`src-tauri/tests/ai_boundary.rs`** — tệp mới, năm ca. `AI_DIR = "core/ai"` · `AI_FLOOR = 1` · `SRC_RS_FLOOR = 44` (44/55 = 80 %) · `FORBIDDEN_BARE_TOKENS = ["crate::core::ai", "super::ai"]`. Sáu ca, tên là câu khẳng định. Vị từ `line_names_a_forbidden_ai_dependency` tách khỏi thân test và được **cả cổng thật lẫn ca gieo tổng hợp** gọi — hai bên không lệch nhau được bằng cách chép logic ở hai chỗ.
2. **`src/i18n/vi.json:190`** — giá trị `panel.ai_translation.status`: 94 → 135 ký tự. Thêm vế *"mọi năng lực khác vẫn chạy đầy đủ"*, giữ giọng MỜI.
   ⚠️ **Một lượt đổi chữ NGOÀI phạm vi được giao, khai ra thay vì để lọt:** cùng lượt đó, *"bật đề xuất tự động"* thành *"bật đề xuất **dịch** tự động"*. Task chỉ giao thêm vế còn thiếu, không giao sửa nửa câu kia. Vòng rà 1 bắt được. **Giữ lại** — panel này sẽ mang bản dịch AI chứ không phải kết quả tra cứu (mệnh đề đã thu hẹp ở Sprint Change Proposal 2026-08-13, ghi trong `AiTranslationPanel.vue:30-34`), nên *"đề xuất dịch"* đúng hơn *"đề xuất"* trần. Nhưng nó là một quyết định, không phải một lượt gõ tiện tay, nên nó được viết ra ở đây.
3. **`src/panels/PanelFrame.vue`** — `.status` chuyển `ui-md` → `ui-md-wrap`, kèm chú thích 🔵 nêu ngày và lý do.
4. **`deferred-work.md`** — bốn mục nợ mới có chủ, và mục `:116` được đóng trọn bằng `→ ✅` nối tiếp (không xoá).

**NĂM phép GỠ-CHỖ-NỐI, số ca đỏ THẬT** *(mỗi lượt: gieo → chạy → khôi phục → `diff` xác nhận nguyên trạng)*:

| # | Gieo gì | Vào đâu | Kết quả |
|---|---|---|---|
| 1 | `use crate::core::ai;` | `src-tauri/src/lib.rs` | **1 ca đỏ** — `no_file_outside_core_ai_names_a_bare_dependency_on_the_ai_module`; 3 ca kia xanh |
| 2 | `stringify!(crate::core::ai::assemble())` — đủ điều kiện, **không** `use` | `src-tauri/src/lib.rs` | **1 ca đỏ** — cùng ca. Xác nhận quét **bare** token, không chỉ dạng có `use` |
| 3 | `stringify!(super::ai::assemble())` | `src-tauri/src/core/scope/mod.rs` | **1 ca đỏ** — cùng ca |
| 4 | *(đối chứng ÂM)* `// crate::core::ai se duoc dung o Story 4.6` | `src-tauri/src/lib.rs` | **4/4 XANH** — `code_lines` bỏ đúng dòng chú thích |
| 5 | `AI_DIR` gõ sai thành `"core/aix"` | chính tệp test | **2 ca đỏ** — ca sàn quần thể *và* mệnh đề *"miễn trừ phải khớp thứ gì đó"* |
| 6 | gỡ `.replace('\\', "/")` khỏi `rel_posix` | chính tệp test | **1 ca đỏ** — `the_core_ai_exemption_still_matches_when_the_path_arrives_windows_shaped`; **4 ca kia vẫn xanh**, tức trước ca này không ai canh vế Windows |

Lượt 5 đóng hàng *"gốc quét sai"* của bảng I/O — sàn quần thể **đã được kích hoạt thật**, không chỉ đúng về mặt logic.

🔵 **Ca thứ NĂM thêm sau lượt soát bảng I/O.** Soát bảng phát hiện hàng *"chạy trên Windows"* không có phép kiểm nào chạm tới — chỉ có một lượt đọc bằng mắt. Đã vá bằng `the_core_ai_exemption_still_matches_when_the_path_arrives_windows_shaped`: kiểm `rel_posix` trên một đường dẫn DỰNG TAY mang hình dạng Windows, cộng một đối chứng ÂM (`core\\dict\\mod.rs` **không** được khớp miễn trừ). Lượt gieo 6 chứng minh nó đỏ được. Giới hạn thật đã ghi trong doc-comment của chính ca: nó nghiệm thu phép **chuẩn hoá**, không nghiệm thu `walk` trên một hệ tệp Windows thật — vế đó vẫn thuộc CI.

**Nghiệm thu cuối (chạy sau khi mọi lượt gieo đã khôi phục):**
- `cargo test --locked` toàn `src-tauri`: **723 ca xanh / 0 đỏ** *(số cuối, sau vòng rà 1)*. Ảnh chụp nền trước lượt vá là 717 ⇒ đúng +6 ca mới, không ca cũ nào lung lay.
- `npx vitest run` (chạy **một mình**): **43 tệp / 567 ca / 0 đỏ** — không đổi so với nền.
- Mười một cổng + `check:lint`: `check:i18n` · `check:tokens` · `check:commands` · `check:panel-refs` · `check:debt-owner` · `check:gates` · `check:deps` · `check:layout` · `check:lint` · `check:dict` · `check:dict-manifest` — **exit 0 cả mười một**.
- `npm run build`: thành công.

**Còn lại / rủi ro Ice cần biết:**
- ⚠️ **Vế Windows nay có MỘT NỬA được canh bằng máy.** Phép chuẩn hoá `\\` → `/` của `rel_posix` đã có ca riêng và ca đó đỏ được (lượt gieo 6). Nhưng `walk` trên một hệ tệp Windows THẬT thì vẫn chưa ai chạy — máy chạy là macOS, và vế đó chỉ được canh ở job `windows-2025` của CI, tức **sau khi push**. `pre-push` xanh ở đây không phủ được nó.
- ⚠️ **`check:scope` và `check:scope:bundled` KHÔNG chạy** — chúng dựng cửa sổ Tauri thật và cần cổng 1420 trống, cố ý nằm ngoài `pre-push`. Lượt này không đụng `capabilities/`, nhưng đó là một suy luận, không phải một phép đo.
- ⚠️ **Vế thị giác của lượt di trú token chưa ai nhìn.** `.status` đổi giãn dòng 1,5 → 1,66 ảnh hưởng **ba** câu trạng thái (AI Translation · Lookup · Grid), và kho không có đường nghiệm thu thị giác tự động. Cần một lượt `npm run tauri dev` của Ice.
- Một chỗ lệch văn phong nhỏ: vài thông báo `assert!` trong ca gieo tổng hợp viết tiếng Việt **không dấu** trong khi phần còn lại của tệp viết có dấu. `tests/**` được miễn trừ nên cả hai đều hợp lệ, nhưng nó không nhất quán trong cùng một tệp.

## Completion Notes — Vòng rà 1 (2026-08-26, `review_loop_iteration` giữ nguyên **0**)

Ba lớp rà đối kháng chạy song song trên diff 73 KB. **21 mục thô → 11 phát hiện riêng biệt** sau khử trùng lặp. **Không mục nào là `intent_gap` hay `bad_spec`** ⇒ không loopback, `review_loop_iteration` giữ **0**.

⚠️ **Một phán đoán phân loại có thể cãi, ghi ra thay vì giấu:** phát hiện ① dưới đây **gần** một lượt lệch spec — §Code Map đã trỏ đích danh `matching_boundary.rs::contains_forbidden_token` + `is_word_byte` *(khớp theo biên từ)* mà bản thi hành chỉ mượn bài học "quét BARE token", bỏ quên bài học "neo biên" nằm ngay cạnh. Xếp `patch` chứ không `bad_spec` vì cách sửa hiển nhiên và cục bộ trong một tệp test; hoàn tác rồi dựng lại sẽ vứt luôn sáu phép gỡ-chỗ-nối đã đo được. Ice lật lại được nếu thấy sai.

**BA khuyết tật cùng một họ — THIẾU NEO BIÊN. Hai lớp rà độc lập cùng chỉ vào ②.**

| | Chỗ | Hạng | Hậu quả |
|---|---|---|---|
| ① | `line_names_a_forbidden_ai_dependency` dùng `contains` trần | ĐỎ OAN | `crate::core::aiven` bị bắt nhầm vì `crate::core::ai` là tiền tố thật của nó |
| ② | miễn trừ thư mục dùng `rel.starts_with("core/ai")` | 🔴 **XANH GIẢ** | `core/aim/` · `core/ai_providers/` được miễn trừ **im lặng**; một `use crate::core::ai;` viết trong đó đi lọt hoàn toàn |
| ③ | `code.contains("ai::")` cho phép kiểm re-export | ĐỎ OAN | `pub use domain_ai::Config;` · `pub use foo::Chai::Bar;` bị bắt nhầm |

② là mục nặng nhất của cả lượt: nó là **xanh giả**, không phải đỏ oan — cổng báo an toàn trên một AD-13 đã bị phá. Và trong đúng một epic tên `ai`, một thư mục anh em bắt đầu bằng "ai" là chuyện dễ xảy ra chứ không phải một khả năng lý thuyết.

**Đã vá:** ① neo biên ký tự sau token · ② `is_inside_ai_module()` khớp theo **biên thư mục** (`rel == AI_DIR || rel.starts_with("core/ai/")`), và **cả bốn** chỗ dùng đi qua nó · ③ + ④ tách `statement_of()` và `statement_reexports_the_ai_module()` thành hàm thuần (đúng nguyên tắc chính tệp này đã khai: vị từ dùng chung thì hai bên không lệch nhau được), neo theo **đoạn định danh**, và cắt chú thích đuôi dòng trước khi so `pub mod ai;` — bản đầu so nguyên văn nên `pub mod ai; // AD-13` làm ca báo đỏ oan rằng khai báo đã biến mất.

**Ca thứ SÁU khoá cả ba, và ba phép gỡ chứng minh nó đỏ được:**

| # | Gỡ gì | Kết quả |
|---|---|---|
| 7 | gỡ neo biên khỏi vị từ token | **1 ca đỏ** — `the_three_predicates_anchor_on_boundaries_and_do_not_fire_on_prefix_neighbours`; 5 ca kia xanh |
| 8 | gỡ neo thư mục khỏi miễn trừ *(ca xanh giả)* | **1 ca đỏ** — cùng ca; 5 ca kia xanh |
| 9 | gỡ neo đoạn khỏi vị từ re-export | **1 ca đỏ** — cùng ca; 5 ca kia xanh |

🔴 **Năm ca kia xanh ở CẢ BA lượt gỡ** — đó là bằng chứng rằng trước vòng rà này không phép kiểm nào canh được ba chỗ đó, và ca thứ sáu không phải một bản chép thừa.

**Năm phát hiện nhỏ đã vá:** số AC bịa trong sổ nợ *(§Acceptance Criteria của story là danh sách không đánh số — nay trỏ bằng lời của AC)* · `Code Map` trỏ `PanelFrame.vue:215-222` vào khối chú thích thay vì ba dòng khai báo, sửa thành `:227-234` · một lượt đổi chữ ngoài phạm vi ở `vi.json` nay được khai và biện minh · hai rủi ro mở chỉ nằm ở §Completion Notes nay thành mục nợ có chủ.

**Ba mục xếp `defer`, đã ghi vào sổ nợ kèm chủ:** `code_lines()` không hiểu khối `/* … */` *(hành vi dùng chung của cả bảy tệp `*_boundary.rs`, sửa riêng ở đây là dựng hành vi thứ hai)* · năm helper đọc-cây nay có **bảy** bản chép và bản vá NFR14 của lượt này **không lan** sang sáu tệp kia · sàn quần thể của bảy tệp là ảnh chụp gõ tay không có cơ chế chống trôi, và bốn sàn cũ **đã trôi** (ghi 53, thật 55).

🔵 **Nghi vấn "sáu tệp kia có cùng lỗ hổng ② không" đã được ĐO, không để lại dạng nghi vấn.** Kết quả hai vế:

- **Vế xấu:** **năm** tệp (`scope_boundary` · `glossary_boundary` · `matching_boundary` · `store_boundary` · `segment_boundary`, tổng **17 chỗ**) đều khớp miễn trừ thư mục bằng `starts_with(<DIR>)` trần — đúng cùng lỗ hổng xanh giả ② vừa vá ở đây.
- **Vế làm nhẹ:** `core/` hôm nay có 12 thư mục con và **0 cặp nào là tiền tố của cặp kia** ⇒ lỗ hổng **tiềm ẩn, chưa kích hoạt ở bất kỳ tệp nào**. Nó nổ vào ngày ai đó đặt `core/dictionary/` cạnh `core/dict/`, `core/segmentation/` cạnh `core/segment/`, hay `core/glossary_import/` cạnh `core/glossary/`.

Phép đo cũng sửa một mệnh đề tôi vừa viết: `code_lines` **không** phải hành vi thống nhất của cả bảy tệp. `segment_boundary.rs:143::is_comment` đã xử thêm `* ` và `*/`, tức bắt được thân và phần đóng của khối `/* … */`, chỉ hở dòng mở — một bản vá nửa vời mà không ai biết là nó tồn tại. Cả hai mục nợ đã được sửa lại theo số đo.

## Suggested Review Order

**Điểm vào — vì sao cổng này đứng trước khi `core/ai/` có một dòng mã**

- Đọc đây trước: đối chứng dương quen thuộc dựng không được ở đây, và khuôn thay thế.
  [`ai_boundary.rs:19`](../../src-tauri/tests/ai_boundary.rs#L19)

**🔴 Ba vị từ — và ba khuyết tật "thiếu neo biên" vòng rà 1 vá**

- 🔴 Nặng nhất: miễn trừ khớp theo BIÊN THƯ MỤC. Bản đầu dùng `starts_with` ⇒ xanh giả.
  [`ai_boundary.rs:197`](../../src-tauri/tests/ai_boundary.rs#L197)

- Vị từ token neo ký tự sau needle — `crate::core::aiven` không còn bị bắt oan.
  [`ai_boundary.rs:178`](../../src-tauri/tests/ai_boundary.rs#L178)

- Vị từ re-export neo theo đoạn định danh — `pub use domain_ai::Config;` không còn đỏ oan.
  [`ai_boundary.rs:212`](../../src-tauri/tests/ai_boundary.rs#L212)

**Bốn mệnh đề của cổng, mỗi ca một mệnh đề**

- Cổng thật: quét toàn cây trừ `core/ai/**`, và đòi miễn trừ phải khớp thứ gì đó.
  [`ai_boundary.rs:252`](../../src-tauri/tests/ai_boundary.rs#L252)

- 🔴 Đối chứng dương bắt buộc: vị từ nổ được trên chuỗi dựng tay, độc lập với cây.
  [`ai_boundary.rs:302`](../../src-tauri/tests/ai_boundary.rs#L302)

- Ca vòng rà 1 khoá cả ba neo — gỡ neo nào ra cũng đúng ca này đỏ.
  [`ai_boundary.rs:469`](../../src-tauri/tests/ai_boundary.rs#L469)

- Điểm mù có tên: `core/mod.rs` khai `pub mod ai;` trần, không re-export.
  [`ai_boundary.rs:373`](../../src-tauri/tests/ai_boundary.rs#L373)

- Sàn quần thể — "cây rỗng đọc thành sạch"; đo lại 55 tệp, sàn 44.
  [`ai_boundary.rs:225`](../../src-tauri/tests/ai_boundary.rs#L225)

- NFR14: miễn trừ vẫn khớp khi đường dẫn tới ở hình dạng Windows.
  [`ai_boundary.rs:433`](../../src-tauri/tests/ai_boundary.rs#L433)

**Nửa giao diện — đóng nốt AC panel và một món nợ đã ký**

- Câu trạng thái nay nói cả hai vế: mời cấu hình, VÀ mọi năng lực khác chạy đầy đủ.
  [`vi.json:190`](../../src/i18n/vi.json#L190)

- `.status` sang `ui-md-wrap` — cùng 13px, giãn dòng 1,5 → 1,66. Ba panel đổi theo.
  [`PanelFrame.vue:227`](../../src/panels/PanelFrame.vue#L227)

**Sổ nợ — thứ story này giao lại, và thứ nó đóng**

- Món nợ Story 1.17 đóng TRỌN sau bốn tháng treo ở `PanelFrame.vue`.
  [`deferred-work.md:117`](./deferred-work.md#L117)

- Bảy mục mới có chủ — gồm phép đo cho thấy năm tệp boundary kia mang cùng lỗ hổng ②.
  [`deferred-work.md:7848`](./deferred-work.md#L7848)
