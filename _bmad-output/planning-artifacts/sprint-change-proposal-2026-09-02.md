# Sprint Change Proposal — 2026-09-02

**Dự án:** AuraTranslate · **Người chốt:** Ice · **Soạn qua:** `bmad-correct-course` (chế độ Incremental)
**Phạm vi phân loại:** **Minor** — bù tài liệu quy hoạch cho năng lực ĐÃ phát hành. Không một dòng mã sản phẩm nào đổi.

---

## 1. Tóm tắt vấn đề

Hai story của Epic 1 **đã dựng xong, đã `done`, đang chạy trong sản phẩm** nhưng **không tồn tại trong `epics.md`**:

| Story | Trạng thái | Ngày đóng | `grep` trong `epics.md` |
|---|---|---|---|
| **1.10c** — Âm Hán Việt: đúng nguồn và đúng nhãn | `done` | 2026-08-06 | **0 dòng** |
| **1.18b** — Tách từ tiếng Trung cho tab Hán Việt | `done` | 2026-08-07 | **0 dòng** |

### Phát hiện thế nào

Không phải từ một story kích hoạt, mà từ lượt `bmad-sprint-planning` intent `validate` ngày 2026-09-02. Lượt đó bắt một lỗi khuôn ở trường `generated`, và đường sửa (`fix` → dựng lại pristine từ `epics.md`) lộ ra rằng bản dựng lại **đánh rơi đúng hai khoá này**: 159/161 khoá của `sprint-status.yaml` ánh xạ sạch sang bộ khoá sinh từ `epics.md`, hai khoá còn lại **không có đối ứng nào** vì nguồn không mô tả chúng.

### Vì sao chúng thiếu — hai lý do khác nhau

- **1.10c sinh ra từ một PHÉP ĐO, không từ một mục backlog.** Task 0 của Story 1.16 đo `dict-core.db` thật và phát hiện `tools/dict-build/src/sources/unihan.rs:116` nạp `Unihan kVietnamese` thẳng vào `dict_entry.han_viet`. Unicode định nghĩa trường đó là *"the Vietnamese pronunciation(s) of this character"* — với chữ **Nôm** thì đó **chính là âm Nôm**, không phải âm Hán Việt. Đối chiếu Thiều Chửu trên phần giao **3.239** ký tự: **1.243 = 38,4 %** cho âm đầu khác nhau. Nó **chặn** Story 1.16 và **đầu độc** Story 3.7 (FR113). Ice chốt cùng ngày: đóng tầng dữ liệu trước, giao diện sau.
- **1.18b sinh ra từ một LƯỢT NGHIỆM THU TAY.** Ice bắt bằng mắt 2026-08-07: *"ở phần văn bản gốc thì double click sẽ chọn được cả cụm từ, vậy tại sao khi chuyển sang phần Hán Việt lại không chọn được"*. Lúc đó **không FR nào đặc tả nó**.

Cả hai đóng trong ngày chúng mở. Bước gấp ngược vào quy hoạch chưa bao giờ chạy.

---

## 2. Phân tích tác động

### Epic

Epic 1 (`in-progress`) **hoàn tất được như quy hoạch**. Thêm hai mục chỉ ghi lại thực tế đã có. Không epic mới, không epic nào bỏ, không đổi thứ tự thực thi. §Epic List **không đếm số story** nên không con số nào phải sửa.

### Story

Không story nào phải sửa nội dung. Hai story được **thêm mô tả** vào nguồn quy hoạch; tệp story trên đĩa giữ nguyên.

### Xung đột artifact

| Artifact | Tác động |
|---|---|
| `epics.md` | **Có** — thêm hai khối story + FR135 vào dòng `FRs covered` của Epic 1 |
| `prd.md` | **Có** — thêm **FR135** (Ice chốt mở FR thay vì ghi nợ) |
| `ARCHITECTURE-SPINE.md` | **Không** — cả hai story chỉ viện `AD` đã có (AD-10·19·25·30·2·26·27 và AD-1·17·34·16·4). **Không AD mới.** |
| UX designs | **Không** — 1.18b đổi hành vi vùng chọn, nhưng đó là hành vi engine, không phải một màn hình hay luồng mới |
| `sprint-status.yaml` | **Có** — đây là đích đến; xem §5 |

### Kỹ thuật

**Không.** Cả hai năng lực đã phát hành và xanh qua các cổng. Đề xuất này không đụng `src/`, `src-tauri/`, `scripts/`, `tests/`, `e2e/`.

---

## 3. Đường đi khuyến nghị

**Option 1 — Direct Adjustment.** Công sức **Thấp**, rủi ro **Thấp**.

| Phương án | Phán quyết |
|---|---|
| **Option 1 — Direct Adjustment** | ✅ **CHỌN.** Thêm hai khối story vào `epics.md` + một FR vào `prd.md`. Có tiền lệ cùng khuôn trong chính tệp: Story 1.10b mang dòng `➕ Story THÊM 2026-08-05 qua correct-course`. |
| Option 2 — Rollback | Không khả thi. Cả hai story đã xanh và đang chạy; hoàn tác sẽ xoá năng lực đang hoạt động để phục vụ một tài liệu. |
| Option 3 — PRD MVP review | N/A. MVP không đụng tới; FR135 ghi lại thứ đã phát hành, không mở phạm vi mới. |

### Chỗ phải cân trước khi chốt

`AGENTS.md` §Conventions ghi: *"Năng lực chưa dựng ≠ lệch spec. **Đừng sửa `epics.md`/`prd.md` cho khớp mã đã viết** — ghi một món nợ có chủ."*

Luật đó nhắm vào việc **hạ một AC** cho khớp mã chưa tới đích — spec bị làm yếu đi để mã trông như đạt. Ca này đi **chiều ngược lại**: không AC nào bị hạ, không lời hứa nào bị rút; hai năng lực đã dựng xong mà nguồn quy hoạch chưa bao giờ mô tả. Đây là **bù một mục thiếu**, và `epics.md` đã có tiền lệ được ký cho đúng hình dạng đó (Story 1.10b). ⇒ Luật đứng nguyên, ca này nằm ngoài phạm vi nó chặn.

---

## 4. Các bản sửa chi tiết

### 4.1 — `prd.md`: thêm FR135

**Vị trí:** §Nội dung theo ngôn ngữ, ngay sau FR33 (`prd.md:494`).

**FR135.** **Double-click ở tab Hán Việt chọn trọn CỤM TỪ, không một âm tiết.** Ở tab nguyên văn tiếng Trung, double-click đã chọn được cả cụm từ; tab Hán Việt phải giữ đúng hành vi ấy — cùng một văn bản, cùng một thao tác, cùng một đơn vị chọn. Đơn vị ở đây là **từ**, không phải **segment**: AD-4 *(ranh giới segment tính một lần lúc nhập)* nói về đơn vị khác và FR này không đụng tới nó.

> **Căn cứ — bộ tách từ đã có sẵn trong engine, không phải thứ phải xây.** Phân tích đầu tiên kết luận *"phải tự xây bộ tách từ, cần `jieba-rs` ở Rust, một story lớn"*. Sai, và Ice lật nó bằng một quan sát: engine đã tách từ đúng ở tab nguyên văn. Truy được qua `Intl.Segmenter` ⇒ **0 phụ thuộc mới · 0 dòng Rust** (NFR15). Vùng chọn nằm trong danh sách năng lực frontend được giữ theo AD-1, nên đây không phải một quy tắc nghiệp vụ rơi xuống TypeScript.
> ⚠️ **Vế còn hở:** `Intl.Segmenter` trên **WKWebView** chưa đo — NFR14 *(hai nền tảng)* chưa nghiệm thu cho đường này.
> ➕ **FR này ghi lại một năng lực ĐÃ PHÁT HÀNH** *(Story 1.18b, `done` 2026-08-07)* mà PRD chưa bao giờ đặc tả. Thêm 2026-09-02 qua `correct-course`.

**Cấp số FR135 bằng phép đo, không bằng ước lượng:** quét toàn bộ `_bmad-output/` (không riêng PRD, theo đúng cảnh báo `AGENTS.md` về cấp số `AD`) — FR134 là số cao nhất và xuất hiện ở ba tệp (`epics.md`, `ad-brief-2026-08-17-mo-hinh-hoan-tac.md`, `sprint-change-proposal-2026-08-14.md`); **FR135 trống ở mọi nơi**.

### 4.2 — `epics.md:843`: Epic 1 nhận FR135

Nối `, FR135` vào cuối dòng `**FRs covered:**` của Epic 1. Nối vào cuối theo đúng khuôn Epic 2 đang dùng cho FR133/FR134 — thứ tự ở dòng này ghi **lịch sử thêm**, không phải thứ tự số hiệu. Dòng `**NFRs:**` của Epic 1 **không đổi**: mọi NFR hai story viện đều đã có mặt.

### 4.3 — `epics.md`: khối Story 1.10c

Chèn sau khối Story 1.10b (kết ở `:1485`), trước `### Story 1.11:`. Mang dòng `➕ Story THÊM 2026-09-02 qua correct-course`, dòng `Vì sao nó thiếu`, `Covers: Không FR mới` (sửa chất lượng dữ liệu dưới FR33 và FR113), khối As/I want/So that trích nguyên từ tệp story, số đo 3.239 / 1.243 / 38,4 %, và chín dòng Given/When/Then dựng từ AC1–AC10.

### 4.4 — `epics.md`: khối Story 1.18b

Chèn sau khối Story 1.18, trước `### Story 1.19:` (`:1878`). Mang `Covers: FR135` — khác 1.10c ở chỗ này, vì Ice chốt mở FR mới. Bảy dòng Given/When/Then dựng từ AC1–AC10, cộng một khối riêng ghi **hai vế story tự khai là không nghiệm thu được** (AC11): `Intl.Segmenter` trên WKWebView chưa đo, và chất lượng tách từ trên văn bản tiểu thuyết thật chưa đo — mẫu đã đo chỉ là văn bản tin tức.

---

## 5. Bàn giao thi hành

**Phân loại: Minor** — thi hành trực tiếp bởi Dev, không cần PO/PM/Architect. Không AD mới nên không cần Winston.

### Việc phải làm, theo thứ tự

1. Áp bản sửa 4.1 vào `prd.md`
2. Áp bản sửa 4.2 · 4.3 · 4.4 vào `epics.md`
3. Dựng lại `sprint-status.yaml` từ `epics.md` đã bù — đây là việc đang chờ ở lượt `bmad-sprint-planning` gọi ra proposal này. Sau khi bù, cả **161** khoá ánh xạ được và **74** `--set` khôi phục trọn trạng thái (53 `done` · 12 `in-progress` · 9 `review`).

### Tiêu chí thành công

- `grep '1\.10c\|1\.18b' epics.md` ra **khác 0 dòng**
- `sprint_plan.py generate --fresh` sinh khoá cho **cả hai** story
- `sprint_plan.py validate` trả `valid: true`
- Không một trạng thái tiến độ nào bị hạ

### 🔴 Một mục CẦN CHỦ, phát hiện trong lượt này và KHÔNG đóng bởi đề xuất này

Tệp story `1-10c-am-han-viet-dung-nguon-va-dung-nhan.md` mang **hai con số NFR6 khác nhau**:

| Chỗ ghi | Payload | Dư |
|---|---|---|
| §Change Log | `373.239.808 / 400.000.000` | `26.760.192` |
| §Dev Agent Record | `396.895.366 / 400.000.000` | `3.104.634` (0,78 %) |

§Dev Agent Record chú *"baseline so sánh đổi, không đụng file `.db` nào (số byte không đổi)"*, nên hai số có thể cùng đúng ở hai mốc khác nhau — nhưng **chưa ai đo lại để nói cái nào đang sống**. Chênh lệch dư địa là **8,6 lần**, và NFR6 là trần phát hành. ⇒ Vì thế khối epic 4.3 viết AC thành *"NFR6 phải có số đo thật trước khi đánh dấu đạt"* thay vì chép một trong hai con số.

**Đề nghị:** ghi vào `deferred-work.md` kèm chủ. **Đề xuất này không tự chấm nó đạt và không tự chọn một con số.**
