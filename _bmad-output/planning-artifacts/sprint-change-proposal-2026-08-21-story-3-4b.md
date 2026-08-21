---
title: 'Sprint Change Proposal — tách nửa giao diện FR50 thành Story 3.4b'
date: '2026-08-21'
trigger_story: '3-4-khop-thuat-ngu-theo-ngon-ngu-qua-matcher-dung-chung'
scope_classification: 'Moderate'
status: 'approved'
approved_by: 'Ice'
workflow: 'bmad-correct-course'
mode: 'incremental'
---

# Sprint Change Proposal — 2026-08-21

## 1. Tóm tắt vấn đề

Story 3.4 đã `done` và xanh cả hai nền tảng, nhưng nó đóng **FR51 và nửa khớp + bề mặt
IPC của FR50** — **không** đóng FR50. Nửa GIAO DIỆN của FR50 *(vẽ dấu ở cột nguyên văn của
lưới trên cả hai đường render, cộng dòng `StatusBar` chở bản dịch đã chốt)* không có một mục
quy hoạch nào đứng sau nó.

**Hai loại vấn đề, không phải một:**

**⓵ Giới hạn kỹ thuật phát hiện lúc LẬP SPEC.** Spec một mảnh của Story 3.4 đo được
**17.408 ký tự ≈ 5.000–5.800 token** so với trần **1.600** của cửa `bmad-build` — vượt
**3,1–3,6×**, trên đúng story mà `deferred-work.md` tự gọi là *"rủi ro nhất của Epic 3"*.
Ice ký tách theo **TẦNG** ngày 2026-08-21, và ràng buộc kèm theo là ⚠️ *"Mở story này phải
đi qua `bmad-correct-course`, không phải một dòng thêm tay vào `sprint-status.yaml`"* — vì
mọi story hậu tố `b` của kho (`1.10b` · `1.11b` · `1.18b` · `2.5b` · `2.5c` · `2.5d`) là một
mục **ĐẦY ĐỦ** trong `epics.md`, và `epics.md:684` ghi thẳng tiền lệ *"thêm 2026-08-05 qua
`correct-course`"*.

**⓶ Giới hạn kỹ thuật phát hiện lúc THỰC THI.** Nửa Rust vừa giao mang về một số đo **vượt
trần NFR2**, và nó chưa được quyết. Đo `marks_for_source_text`, `cargo test --release`,
`rustc 1.97.1`, macOS/darwin 24.6.0, Intel i9-9980HK, 6 lượt/điểm đo, Glossary **5.000 mục**
tiếng Trung *(dựng cố ý đặc để mô phỏng ca xấu hơn mức thường — một **cận trên**)*:

| Cỡ Chương (ký tự) | Trung vị (ms) | Thấp nhất | Cao nhất |
|---:|---:|---:|---:|
| 3.000 | 23,6 | 22,9 | 25,5 |
| 10.000 | 55,5 | 49,9 | 57,0 |
| 20.000 | 93,9 | 90,1 | 104,7 |
| **48.640** *(Chương lớn nhất có thật, 9.850 câu)* | **214,0** | 194,1 | 248,5 |

⇒ Vượt trần NFR2 (**50 ms**) ngay từ **10.000 ký tự**, và **~4,3×** ở quy mô lớn nhất có thật.

## 2. Phân tích tác động

### Tác động Epic
Epic 3 hoàn thành được **như quy hoạch**. Không FR nào bị cắt, không AC nào đổi nội dung,
không epic nào thêm/bỏ. Cái đổi là **số lượng mục**: một story thành hai.

### Tác động Story
- **Story 3.4** — thu hẹp: `Covers: FR51 · nửa khớp + bề mặt IPC của FR50`. Bốn AC ở lại
  (khớp Trung · khớp Anh · dùng `Matcher` dùng chung · hai tầng), cả bốn **đã xanh**.
- **Story 3.4b** — mới, `backlog`. `Covers: FR50` *(nửa giao diện)*. Bảy AC.
- **Story 3.5 → 3.10 · Epic 4 · Epic 7** — **không mục nào bị chạm**. Kiểm được: Epic 4 phụ
  thuộc `entries_eligible_for_injection` (Story 3.1), không phụ thuộc đường đánh dấu; Story
  7.5/7.6 dùng chung `find_terms` nhưng `resolve_overlaps` là **luật riêng của kênh đánh
  dấu**, đặt ở `core::glossary`, **không** sửa `core::matching`.
- **Thứ tự thực thi** — không đổi. `build-sequence.md` chỉ có độ hạt **epic**.

### Xung đột artefact
- **PRD — không xung đột.** FR50 (`prd.md:548`) và FR51 (`:550`) **không đổi một chữ**. Chúng
  mô tả đích đến; tách story là chuyện đường đi. Luật kho: *"Năng lực chưa dựng ≠ lệch spec"*.
  MVP **không** bị chạm.
- **Kiến trúc — không xung đột.** Không AD nào đổi. AD-17 · AD-18 · AD-36 đã được nửa Rust
  nghiệm thu. Không port thứ tư, không phụ thuộc mới (NFR15 không kích hoạt).
- **UX — không xung đột, và nó RÀNG BUỘC 3.4b bằng ba mệnh đề đã có sẵn:** `DESIGN.md:165`
  (`primary #2f5d63` là màu nhấn duy nhất, đúng ba việc) · `DESIGN.md:221` (kiểm toán
  2026-08-03: lùi hàng bằng `opacity` **trượt AA** ⇒ mục chờ chốt phân biệt bằng **kiểu gạch
  chân**) · `EXPERIENCE.md:144` + `DESIGN.md:327` (thanh trạng thái **34px** đã chở *"Đã lưu N
  giây trước"* — dòng bản dịch là người ở trọ **thứ hai**, không một bề mặt mới).

### Tác động kỹ thuật

**🔴 Một ràng buộc CẤU TRÚC chưa ai ghi ra, và nó đổi phạm vi 3.4b.** Tìm ra khi đọc mã
trong chính lượt này — không có trong sổ nợ:

`glossary_marks_for_chapter(text, source_lang)` trả offset **TUYỆT ĐỐI** vào `text`, danh
sách **PHẲNG**, không mang `segment_id`. Nhưng lưới render **theo từng segment**
(`GridPanel.vue:1391`), và `ChapterSegment` (`src/config/segment.ts:66-95`) **không có
trường offset cấp Chương nào**. Ba đường ra, cả ba có giá:

| Đường | Phán quyết |
|---|---|
| Gọi **mỗi segment** một lượt | 🔴 **LOẠI bằng phép đo** — `marks_for_source_text` gọi `load_tier` cho **cả hai tầng ở MỖI lượt** (`store.rs:788-789`); Chương 9.850 câu ⇒ 9.850 lần nạp 5.000 mục |
| Truyền `chapter.source_text` | ⚠️ Ánh xạ ngược **không** phải phép cộng dồn: `push_segment` **`trim()`** mỗi câu và bỏ câu rỗng (`split.rs:393-402`), `skip_gap` nuốt trọn khe trắng (`:374-388`); và sau gộp/tách segment (2.8/2.9) segment không còn dẫn xuất được từ văn bản Chương |
| Nối `segment.source_text` bằng một chất nối rồi cộng dồn ở frontend | Khả dĩ nhất, nhưng dựa trên một giả định **CHƯA ĐO**: không dấu nào bắc cầu qua chất nối. Nhánh `En` từ chối bắc cầu qua `\n`; nhánh `Zh` phụ thuộc jieba cắt `\n` thành token riêng — **chưa ai đo** |

⇒ **3.4b không phải một story thuần frontend.** Tuỳ đường được chốt, nó có thể phải đổi hình
dạng dây (thêm `segment_id`, hoặc nhận một lát `Vec<&str>`) — tức chạm lại `src-tauri/`.
**Quyết định của Ice 2026-08-21: ghi thành một AC mở bằng PHÉP ĐO**, không chốt đường ở đây.

**Bốn mục sổ nợ MỒ CÔI mà cổng không thấy được.** `check-debt-owner.mjs` chỉ kiểm *"có `Chủ:`
thật hay không"*; `detectOwner` (`:281`) trả một **boolean**, không một **tên**. Nên bốn mục
mang `Chủ: Story 3.4` — một story đã `done` — **xanh ở cổng mà mồ côi trên thực tế**.

## 3. Đường đi được chọn

**Direct Adjustment (Option 1).**

| Phương án | Phán quyết | Lý do |
|---|---|---|
| **Direct Adjustment** | ✅ **CHỌN** | Công: **thấp** (chỉ tài liệu quy hoạch). Rủi ro: **thấp**. Không mã sản phẩm nào bị chạm |
| Potential Rollback | ❌ Không khả thi | Không có gì để lùi: nửa Rust xanh cả hai nền tảng (run `32453862410` trên `56859a9`), không mệnh đề nào của nó sai. Lùi để "gộp lại làm một" là quay về đúng lượt vượt trần token 3,1–3,6× mà Ice đã ký tách |
| PRD MVP Review | ❌ Không khả thi | FR50/FR51 không đổi, MVP không bị chạm |

### Hai chữ ký của Ice, 2026-08-21

**🔴 ⓵ Tần suất gọi lại của nửa giao diện: MỘT lượt mỗi lần mở Chương**, cộng một lượt làm
mới khi Glossary đổi *(thêm nhanh 3.3)* hoặc khi segment gộp/tách — **không một lượt nào
trên đường gõ**.

⇒ 214 ms **không** rơi vào khung hình gõ ⇒ NFR2 **không** bị phá ⇒ **không** chỉ mục ngược,
**không** cache. Đây là điều kiện khởi hành mà `deferred-work.md` đặt đích danh, và nó nay
**đóng**. Lập luận đỡ lưng, kiểm được: cột được đánh dấu là cột **NGUYÊN VĂN** — nó không đổi
khi người dùng gõ bản dịch, nên một lượt khớp lại theo thời gian thực sẽ cần một lý do riêng
mà hôm nay không có.

⚠️ **Vế CÒN NỢ, và nó không phải vế trên:** đường mở Chương nay chở **CẢ** lượt hâm `Jieba`
*(179–329 ms, trung vị ~243)* **LẪN** lượt khớp *(214 ms)*. Tổng chưa ai đo. Thành một AC
của 3.4b: đo một **CẶP** số trên **CÙNG** một Chương tiếng Trung, tách lượt **LẠNH** ra khỏi
lượt ấm — vì `open_adjacent_chapter` lặp lại nhiều lần trong một phiên và lượt hâm chỉ tốn ở
lần ĐẦU, nên một phép đo chạy liên tiếp sẽ **giấu mất** chi phí thật.

**⓶ Ràng buộc hình dạng dây** ghi thành **một AC mở bằng phép đo** của 3.4b, không chốt
đường ở lượt này.

## 4. Đề xuất sửa chi tiết — ĐÃ ÁP

### `epics.md`

**⓵.a — thu hẹp mục Story 3.4** (`:2902`)

OLD:
```
### Story 3.4: Khớp thuật ngữ theo ngôn ngữ và đánh dấu ở cột nguyên văn của lưới

**Covers:** FR50, FR51
```
NEW:
```
### Story 3.4: Khớp thuật ngữ theo ngôn ngữ qua Matcher dùng chung

**Covers:** FR51 · **nửa khớp + bề mặt IPC của FR50**
🔵 (Thu hẹp 2026-08-21 qua `correct-course` — … phép tách theo TẦNG, không theo mục tiêu …)
```

Ba AC chuyển đi:

| AC cũ | Xử |
|---|---|
| ① *"đánh dấu bằng màu `primary`"* | **chuyển** trọn sang 3.4b |
| ⑤ *"mục chờ chốt cũng được đánh dấu **And** phân biệt được"* | **tách**: vế dữ liệu (`is_confirmed` trên dây) **ở lại** 3.4 — đã có `glossary_marks_contract.rs`; vế **VẼ** chuyển đi |
| ⑦ *"rê chuột hoặc đưa tiêu điểm ⇒ thấy bản dịch đã chốt"* | **chuyển** trọn sang 3.4b |

Bốn AC ở lại (②③④⑥) — **không đổi một chữ**, cả bốn đã xanh.

🔴 **Đây KHÔNG phải sửa `epics.md` cho khớp mã đã viết** *(thứ `AGENTS.md` cấm)*. Đây là bù
một **mục quy hoạch còn thiếu** đi qua đúng workflow đã có.

**⓵.b — thêm mục Story 3.4b** (`epics.md:2938`), 7 AC + 4 ghi chú cài đặt. Toàn văn ở
`epics.md`; ba quyết định của Ice ngày 2026-08-21 **đi thẳng vào AC** thay vì sống trong sổ nợ:
(a) kênh chở bản dịch là **một dòng `StatusBar`**, không một lớp nổi — **0** miễn trừ
`z-index`; (b) đánh dấu chạy ở **cả hai** đường render, phép cắt làm ở **tầng dữ liệu**
(`buildSegments`/`sourcePiecesOf` tự cắt tại biên thuật ngữ), **không** chèn node vào DOM;
(c) mục chờ chốt phân biệt bằng **kiểu gạch chân**, tuyệt đối **không** `opacity`.

**⓵.c — bảng truy vết FR50** (`:700`) — nay nêu **hai** story, khuôn chép từ hàng FR34 ở
`:685`, hàng duy nhất trong bảng đã mang cùng hình dạng *"một FR, hai story"*.

### `deferred-work.md` — tám lượt sửa

🔴 **Không lượt nào ĐÓNG một mục.** Kiểm được: `check-debt-owner.mjs:231-234` chỉ nhận
`→ ✅` · `→ 🟡` · `→ KHÔNG LÀM <ngày> (` làm trạng thái; `→ 🔵` **không** đổi trạng thái.

| # | Mục | Sửa |
|---|---|---|
| a | *"Story 3.4 … phải TỰ CẮT `.hv-unit`"* + bất biến `host.children[i] ↔ segments.value[i]` | chủ → **3.4b** |
| b | *"Story 3.4 là story rủi ro nhất của Epic 3"* | chủ → **3.4b** |
| c | Chuột kéo trên WKWebView chưa nghiệm thu | chủ → **3.4b** |
| d | ICU cắt sai **tên riêng** (bảng 10 ca đo thật) | chủ → **3.4b** · 3.7 |
| e | Tham chiếu `deferred-work.md:834` — dòng đó nay **TRỐNG** | trỏ bằng **TÊN mục** thay vì số dòng |
| f | Điều kiện khởi hành của 3.4b | nối `→ 🔵` ghi chữ ký của Ice; **món nợ ĐO ở lại MỞ** |
| g | `ScopeResolver` chưa cache | chủ → **3.4b** |
| h | `ngrams`/`find_terms` tokenize hai lần | chủ → **7.6** *(dòng `→ 🟡` đã bàn giao đích danh; nhãn chủ còn mang tên story đã đóng)* |

Nghiệm thu: `npm run check:debt-owner` — **0/330 mục mở thiếu `Chủ:`**, 507 tổng · 56 nửa ·
112 đóng. Kiểm B (tự kiểm) xanh 13 ca + 1 + 5.

### `sprint-status.yaml`
Thêm `3-4b-danh-dau-thuat-ngu-o-cot-nguyen-van-cua-luoi: backlog` kèm khối nhật ký 🔵 theo
khuôn *"Nhật ký sprint-status"*; `last_updated` cập nhật. YAML nghiệm thu bằng `yaml.safe_load`.

### `epic-3-context.md`
Đồng bộ §Stories và §Cross-Story Dependencies.
⚠️ Tệp này mang dòng *"Regenerate with compile-epic-context if planning docs change"* — một
lượt sinh lại từ `epics.md` **đã sửa** sẽ đè lên bản sửa tay này, và đó là kết quả **đúng**.

## 5. Bàn giao

**Phân loại phạm vi: MODERATE** — tổ chức lại backlog, không cần replan tầng PM/Architect.
Kiểm được: 0 dòng mã sản phẩm bị chạm · 0 AD đổi · 0 FR đổi · MVP không bị chạm.

| Người nhận | Trách nhiệm |
|---|---|
| **Ice** | Đã ký hai quyết định (tần suất gọi lại · xử lý ràng buộc hình dạng dây). Cửa tiếp theo: duyệt spec Story 3.4b lúc `bmad-build` soạn |
| **Developer agent** (`bmad-build`) | Soạn và thực thi spec Story 3.4b từ mục `epics.md` §Story 3.4b. ⚠️ Đọc **bốn** mục sổ nợ vừa chuyển chủ **TRƯỚC** khi viết dòng đầu |

**Điều kiện thành công của Story 3.4b:**
1. FR50 đóng — dấu hiện đúng trên **CẢ HAI** đường render, mục chờ chốt phân biệt bằng **kiểu
   gạch chân** (**không** `opacity`), bản dịch hiện ở **một dòng `StatusBar`** (**0** miễn trừ
   `z-index`).
2. Bất biến `host.children[i] ↔ segments.value[i]` giữ đúng **theo cấu tạo**; AC6/1.16 và
   AC11+AC12/1.18 **đo LẠI**, không suy từ số đo cũ.
3. Đường ánh xạ offset chốt bằng một **PHÉP ĐO** ghi số vào story, trên **cả hai** nhánh
   `Zh`/`En`.
4. **Cặp số mở Chương** đo và ghi — trước/sau, tách lượt **LẠNH**.
5. `pre-push` xanh **VÀ** đọc lượt CI thật *(pre-push chỉ chạy trên macOS của Ice)*.
   ⚠️ **Và "CI xanh" KHÔNG có nghĩa e2e đã chạy** — job `e2e` chỉ chạy theo nhịp đêm và khi
   bấm tay; ở run `32453862410` nó là `skipped`. Bề mặt này là bề mặt **DOM**, nên một mệnh
   đề *"webview thật"* của 3.4b **phải** có một lượt e2e chạy tay đỡ lưng, không một suy luận
   từ `check (macos-26) success`.

## 6. Hệ quả do chính lượt này gây ra — và một phát hiện nó phơi ra

Khối Story 3.4/3.4b dài thêm **53 dòng**, nên mọi mốc `epics.md:N` với `N ≥ 2942` trong toàn
kho lệch **+53**. Đúng lớp lỗi *"tham chiếu trôi"* mà mục ⓷.e của chính đề xuất này vừa sửa.

**Đo được:** 14 tệp · 28 mốc. Đối chứng bằng `git show HEAD:epics.md` — dòng cũ `4946`/`5169`
khớp **y nguyên** dòng mới `4999`/`5222`.

### 🔴 Phép đo lật một giả định

Bước +53 khôi phục **trung thành** trạng thái cũ — nhưng **trạng thái cũ đã sai sẵn**:

| Mốc (sau +53) | Khai là gì | Nội dung THẬT ở đó | Vị trí ĐÚNG | Lệch |
|---|---|---|---|---|
| `4999` · `5003` *(`mod.rs` ×4 · `matching_contract.rs` ×2)* | *"n-gram ký tự"* / *"token n-gram sau stemming"* — Story 7.6 | AC **ảnh/alt-text** của Epic 6 | §Story 7.6 | **+397** |
| `5087` | *"người dùng gọi lệnh Concordance"* | *"vào trạng thái Đang dịch"* | §Story 7.7 | **+336** |
| `6255-6257` | AC của **Story 10.4** | `### Story 9.2` | §Story 10.4 | **+373** |
| `3037-3038` | Story 3.5 — quét không sinh ứng viên trùng | dòng trống + `---` | §Story 3.5 | **−2** |
| `6168-6170` | *"xuất xứ thu hoạch từ bản review"* | ✅ đúng | §Story 8.14 | **0** |

⇒ **8/13 lượt xuất hiện của nhóm SỐNG trỏ lệch hàng trăm dòng, và cả tám lệch TRƯỚC lượt này.**
`epics.md` đã dài thêm ~400 dòng kể từ Story 1.12 *(2026-08-05)* mà **không mốc nào đỏ** — vì
không cổng nào canh: `check-layout.mjs`/`check-commands.mjs` chỉ **nhắc** `epics.md:N` trong
chú thích, không đọc tệp.

### Xử lý — Ice chốt 2026-08-21

**13 mốc SỐNG đổi sang §TÊN mục** *(mã Rust · test · sổ nợ đang mở)* — thứ không trôi ở lượt
sửa `epics.md` tiếp theo, cùng phép đã dùng ở ⓷.e:

| Tệp | Mốc |
|---|---|
| `src-tauri/src/core/matching/mod.rs` | 4 → §Story 7.6 |
| `src-tauri/tests/matching_contract.rs` | 2 → §Story 7.6 |
| `src-tauri/tests/glossary_contract.rs` | 1 → §Story 3.5 |
| `deferred-work.md` | 6 → §Story 3.5 · 7.7 · 8.14 · 10.4 |

**19 mốc LỊCH SỬ** *(story đã `done` · AD brief · đề xuất sprint cũ)* — ghi thành **hai món nợ
có chủ** ở `deferred-work.md` §*"lượt `correct-course` tách Story 3.4b"*, kèm bảng đầy đủ:
⓵ 19 mốc còn trôi, **Chủ: Ice** *(viết lại một bản ghi đã đóng là câu hỏi về tính toàn vẹn của
lịch sử, không phải một lượt sửa kỹ thuật)*; ⓶ không cổng nào canh lớp này, **Chủ: một story
hạ tầng cổng kế tiếp** *(thêm một cổng = sửa BA danh sách, ngoài phạm vi `correct-course`)*.

⚠️ **Một lỗi của chính lượt sửa, bắt được bằng đối chứng và đã sửa:** regex bước số chỉ chạm
**số đầu** của một khoảng, để lại ba cặp ngược (`6168-6117` · `6255-6204` · `3037-2985`).
Phát hiện bằng `awk` so hai đầu khoảng, sửa, rồi đối chứng lại: 0 khoảng ngược. Lượt đổi sang
§TÊN cũng để thừa một backtick lẻ ở cả 13 mốc — bắt bằng phép đếm backtick theo dòng, gỡ, đối
chứng lại: 0 dòng backtick lẻ, và **6/6** §TÊN được trích giải đúng **một** `### Story` heading.

## 7. Nghiệm thu lượt này

| Phép kiểm | Kết quả |
|---|---|
| `npm run build` | ✅ exit 0 |
| `cd src-tauri && cargo test --locked` | ✅ exit 0 — 23 bộ, **0 failed** |
| `npm run check:debt-owner` | ✅ **0/332** mục mở thiếu `Chủ:` — 509 tổng · 56 nửa · 112 đóng; Kiểm B xanh |
| `npm run check:i18n` | ✅ xanh |
| `npm run check:gates` | ✅ xanh — story này **không** thêm cổng |
| YAML `sprint-status.yaml` | ✅ `yaml.safe_load` — `3-4b-…: backlog` |
| Mốc đã bước | ✅ **9/9** giải về **đúng dòng cũ** của bản `HEAD` |
| §TÊN đã trích | ✅ **6/6** giải đúng **một** `### Story` heading |

⚠️ **Chưa chạy, và không được đọc thành đã chạy:** `npm run test` *(vitest)* và bộ **e2e**.
Lượt này không chạm một dòng `.ts`/`.vue` sản phẩm nào — chỉ chú thích Rust và tài liệu — nên
`cargo test` là cổng đúng. Nhưng `pre-push` sẽ chạy vitest, và **`pre-push` xanh trên macOS
của Ice không nói gì về nửa Windows**: đọc lượt CI trước khi kết luận.
