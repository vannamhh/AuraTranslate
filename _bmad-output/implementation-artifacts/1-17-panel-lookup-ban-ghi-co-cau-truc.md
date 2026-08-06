---
baseline_commit: cb03974
---

# Story 1.17: Panel Lookup — bản ghi có cấu trúc

Status: done

> 🔴 **STORY NÀY MỞ MỘT ĐƯỜNG CHƯA TỒN TẠI.** `DictLayers` đã được `app.manage(...)` từ
> Story 1.13 nhưng ⛔ **chưa một `#[tauri::command]` nào lấy nó ra** — `deferred-work.md:453`
> ghi thẳng: *"đó là **Story 1.17**"*. Toàn bộ đường tra cứu (pha một `lookup_grouped`, pha
> hai `senses`) sống ở Rust, đã có test hành vi, và ⛔ **chưa một byte nào của nó từng đi qua
> biên IPC**. Story này là chỗ nó đi qua lần đầu.
>
> ✅ **Hai quyết định đã chốt 2026-08-06 (Ice):** **#4** — **thêm `LIMIT` pha một** *(⇒ đổi
> chữ ký cổng `DictionarySource::lookup`, đổi cả sáu hình dạng SQL, đổi mọi test hành vi —
> **đây là phần đắt nhất của story**)*. **#7** — **thêm token thứ 17 `ui-md-wrap`**, đóng
> `deferred-work.md:115` sau **bốn** lần bị gọi tên.
> ⬜ Còn **#1 · #2 · #3 · #5 · #6** — chốt ở Task 0; cả năm có mặc định đề xuất kèm lý do.
> **#2 chặn thật** *(hình dạng dữ liệu trên dây; sửa sau là mổ lại cả ba tầng)*.
>
> 🔴 **Và quyết định #4 mở ba nguy cơ mới mà ⛔ chưa quyết định nào phủ** — đọc §Quyết định
> #4 trước khi gõ dòng đầu tiên. Cái nguy hiểm nhất: **một `LIMIT` đặt sai chỗ làm một
> nguồn từ điển biến mất khỏi kết quả, im lặng, và FR31 vỡ mà ⛔ không cổng nào đỏ.**

## Change Log

| Ngày | Thay đổi |
|---|---|
| 2026-08-07 | 🔴 **Code review — ba lớp (Blind Hunter · Edge Case Hunter · Acceptance Auditor), baseline `cb03974`.** 17 phát hiện `patch` · 6 `defer` · 2 loại là nhiễu. Cả 9 cổng DoD xanh và `cargo test` 217/217 xanh **tại thời điểm review**, nên mọi phát hiện là khuyết tật ngữ nghĩa ⛔ cổng nào bắt được.<br><br>**Ice chốt sáu quyết định:** **(1)** thanh nhịp ⇒ **cài `COUNT`** như đã chốt ở Task 0 *(đường (a) §hệ quả ③ — nó ⛔ hề tồn tại trong mã, `grep COUNT` ra 0 kết quả, và việc bỏ nó ⛔ được ghi ở Completion Notes)*. **(2)** chip ngoại ngữ ⇒ **Rust trả cờ `is_foreign`** *(AD-1; `pos_lang = "vi"` từng bị dán chip `VI` — nhãn **tiếng Việt** ⛔ phải nhãn ngoại ngữ)*. **(3)** vùng đầu mục ⇒ **luôn render `.lookup-head`** *(bản đầu bọc nó trong `v-if="!neverLookedUp"` nên chiều cao đi 0 → 76px, tức hằng có tên thì có mà bất biến AC7 thì ⛔)*. **(4)** đầu mục ⇒ **render `EntryHit.headword` THẬT + gom nghĩa theo `entry_id`** *(bản đầu in truy vấn thô ⇒ khớp qua `headword_simp` hiện SAI CHỮ; và danh sách `senses` phẳng làm 18 ca trùng `headword` đọc thành một khối liên tục — gần "gộp" hơn Quyết định #5(a))*. **(5)** NFR1 ⇒ **ĐO LẠI trước khi đóng** *(⛔ đóng story trên một con số chưa hiểu)*. **(6)** dòng bất đồng ⇒ **đổi CHUỖI thành mệnh đề khả năng** *(vị từ chỉ đếm nhóm; ⛔ đẩy một phép so nội dung vào webview — AD-19)*.<br><br>🔴 **Kết quả (5) — `p99 70,742 ms` ⛔ TÁI LẬP.** Ba lượt đo lại độc lập cho p99 đường sản phẩm **0,566 / 1,136 / 1,793 ms**; tỉ lệ p95↔p99 về **1,1–2,1×** *(gốc: **10,8×**)*. ⇒ con số gốc là **nhiễu page-cache của lượt đo đầu**, ⛔ một thuộc tính của mã. **NFR1 ĐẠT, và lần này kết luận đứng trên p99.** Xem §Debug Log References.<br><br>✅ **Vết phê duyệt còn thiếu, bổ sung tại đây (mục `patch` #17):** lượt Ice xác nhận **đường (b)** cho AC12 *(nguồn thứ hai được phép vắng khỏi `groups` miễn là `truncated` nói ra)* được Completion Notes viện dẫn mà ⛔ có hàng Change Log nào ghi. Đường (b) **được** nhánh `Or` của AC12 cho phép nên đây ⛔ phải một vi phạm — nhưng nay §hệ quả ③ đã cài, `hidden_sources` **gọi TÊN** nguồn bị cắt sạch, nên vế FR31 *"mọi định nghĩa hiển thị nguồn"* được đóng chặt hơn cả đường (b) hứa.<br><br>**Sổ sách cổng sửa hai chỗ:** `DISPATCH_FLOOR` từng ghi *"trước story: 8"* để biện minh cho lượt nâng 6 → 10 — đếm lại bằng CHÍNH `DISPATCH_CALL_RE` của cổng ra **12 trước, 12 sau** *(story ⛔ thêm/bớt một `dispatch()` nào; `git diff src/**` ⛔ một dòng)*; sàn 10 giữ nguyên vì đúng theo số thật, chỉ **lý do** được sửa. Và hai sàn AC13 gọi đích danh mà bản đầu đánh dấu *"không đổi"* nay đã nâng: **`CLICK_FLOOR` 6 → 7** *(số thật 8)* · **`RS_FLOOR` 32 → 34** *(số thật 40)*.<br><br>**Sau lượt sửa:** `cargo test` **225/225** xanh *(+8 ca mới)*, cả 9 cổng DoD xanh, `npm run build` xanh. |
| 2026-08-06 | 🔴 **Task 0 — bảy quyết định chốt sau khi ĐO THẬT trên `tools/dict-build/out/*.db` (4 lớp, 10 nguồn — hôm nay KHÁC 3 tệp story kể lúc dựng: `dict-vietphrase.db` đã tách lớp riêng, `dict-core.db` mang 7 nguồn chứ không 6).**<br><br>**Baseline:** `git status` sạch, `cargo test --manifest-path src-tauri/Cargo.toml` xanh (155 test), cả 9 lệnh DoD xanh (`build`/`check:tokens`/`check:i18n`/`check:commands`/`check:layout`/`check:deps`/`check:dict-manifest`/`check:scope` exit 0).<br><br>**Đo lại `:419`** (`bench_the_grouped_path_on_the_real_dictionaries`, `--release`): nhánh xấu nhất vẫn là `char_idx` 1 ký tự (`山`) — pha một p95 **12,949 ms** (7 hàng/nhóm giữa 8 nhóm, 6.565 hàng), hydrate TOÀN BỘ lớp `vietphrase` (3.385 đầu mục) tốn thêm p95 **17,066 ms**. Vượt trần 10 ms — đúng `:419` báo, số đổi nhẹ vì tập lớp đổi từ 3→4 tệp.<br><br>**🔴 Quyết định #4, hệ quả ① — ĐO LẠI, KẾT LUẬN NGƯỢC với lo ngại ban đầu của story.** `EXPLAIN QUERY PLAN` thật trên `dict-vietphrase.db`/`dict-core.db`: nhánh `char_idx` (cả 1 và 2 ký tự) **KHÔNG** có bước `USE TEMP B-TREE FOR ORDER BY` — kế hoạch dùng `LIST SUBQUERY` (driven bởi `SEARCH char_idx USING PRIMARY KEY (ch=?)`, đã sắp theo `entry_id` tăng dần nhờ `PRIMARY KEY (ch, entry_id) WITHOUT ROWID`) rồi `SEARCH e USING INTEGER PRIMARY KEY (rowid=?)` — tức streaming, khớp `ORDER BY e.id` mà không cần sort riêng. Đo tay bằng `sqlite3 .timer on`: 1-ký-tự (`山`, vietphrase) NO LIMIT ≈ 9–12 ms → LIMIT 20 ≈ 1 ms (**~10×**); 2-ký-tự (`一人`, dict-core, 75 hàng) NO LIMIT ≈ 13 ms → LIMIT 20 ≈ 5 ms. **⇒ `LIMIT ?N` đơn giản gắn vào CUỐI câu hiện có (⛔ không cần đẩy vào subquery — Phương án C) đã đủ cắt thời gian THẬT.** Nhánh `fts_trigram` (nhánh 3) CÓ `USE TEMP B-TREE FOR ORDER BY` (đo tay: 4 ms → 3 ms, ⛔ không đáng kể) nhưng nhánh này vốn đã dưới trần (0,6–2,0 ms ở mọi ca đo) nên không quan trọng cho NFR1 — `LIMIT` vẫn giữ giá trị cắt băng thông IPC. **⇒ Điều kiện "nếu ① đúng thì DỪNG và hỏi Ice" ⛔ KHÔNG áp dụng** — ① sai trên số đo thật, nên **⛔ không cần khảo sát bốn phương án / không cần hỏi Ice**, đúng đúng chữ *"đo trước, rồi mới hỏi có cần không"* của story.<br><br>**🔴 Quyết định #4, hệ quả ② — ĐỔI Ý so với đề xuất mặc định (a) `ROW_NUMBER() OVER (PARTITION BY source_id)`, VÌ SỐ ĐO.** Trước tiên đo được lỗ hổng THẬT: `LIMIT 20` cấp-tệp trên `dict-core.db` (7 nguồn) cho ký tự `一` (rất phổ biến) trả **20/20 hàng đều thuộc `cvdict`** — bốn nguồn khác (`cc-cedict` 1.186 hàng, `unihan` 1, `viwiktionary` 11, `en-wiktionary` 1.590) **biến mất hoàn toàn**, đúng lỗi FR31 mà AC12 cảnh báo, tái lập được. Nhưng đo tiếp phương án (a) đề xuất trong story: `EXPLAIN QUERY PLAN` của `ROW_NUMBER() OVER (PARTITION BY e.source_id ORDER BY e.id)` cho ra **hai** bước `USE TEMP B-TREE FOR ORDER BY` (một cho window function, một cho `ORDER BY` ngoài) — tức vật liệu hoá toàn bộ trước khi cắt, ⛔ không dừng sớm được. Đo tay: 15 ms — **CHẬM HƠN** cả bản không có `LIMIT` nào (9–11 ms). ⇒ (a) tự đánh mất đúng lợi ích mà `LIMIT ?N` đơn giản vừa mua được ở hệ quả ①. **CHỐT: đường (b)** — `LIMIT` cấp-tệp giữ nguyên hình dạng đơn giản (khớp hệ quả ①), cộng một cờ `truncated: bool` khi số hàng trả về == `N` (dấu hiệu "có thể còn nữa, không rõ nguồn nào"), panel nói "danh sách nguồn ⛔ chưa đầy đủ" theo đúng nhánh AC12 cho phép. Test AC12 (fixture MỘT tệp/HAI nguồn, nguồn 2 toàn `id` lớn hơn nguồn 1) sẽ đỏ trên `LIMIT` cấp-tệp ngây thơ đúng như AC12 đòi — Task 1b phải làm nó ĐỎ trước rồi XANH bằng cờ `truncated`.<br><br>**Quyết định #4, hệ quả ③** — CHỐT đường (a): một truy vấn `COUNT(*) … GROUP BY source_id` riêng, chạy cùng lượt pha một CHỈ KHI `truncated = true` (tránh trả giá `COUNT` cho ca thường — phần lớn lượt tra không chạm trần). Cỡ trang `N` chốt SAU Task 8, theo đúng thứ tự story đòi.<br><br>**Quyết định #1** — CHỐT theo mặc định đề xuất **(a)**: `runLookup(query)` một đường vào, command `lookup.lookup_selection` + `deps.currentSelection` tiêm ở `main.ts`, cùng cửa `applyPreset`/`selectSourceTab`. Không lý do phản bác.<br><br>**Quyết định #2** — CHỐT theo mặc định đề xuất **(a)**: derive `Serialize` thẳng lên các kiểu bản ghi `core::dict` đã `pub`, ⛔ `rename_all`; `QueryBranch`/`QueryRoute` → chuỗi định danh máy; `SkipReason` rút gọn còn số lượng + mã máy (⛔ `path`/`detail`).<br><br>**Quyết định #3** — CHỐT theo mặc định đề xuất **(a)**: `LookupMode::Exact` cho một lượt bôi đen.<br><br>**Quyết định #5** — nhìn bằng mắt 18 hàng trùng `headword` của `dict-vietphrase.db` (đo lại ra **18** cặp, khớp số story kể). Hai ca đại diện: `未来` (id 538979/655980) — **hai đầu mục gloss GIỐNG HỆT NHAU** (`"tương lai"` cả hai, dữ liệu thô trùng lặp thật); `黄沙` (id 2/539865) — đầu mục thứ hai có **thêm** một gloss (`"cát vàng"` + `"hoàng sa"`) mà đầu mục đầu không có. CHỐT theo mặc định đề xuất **(a)**: hiện LIỀN NHAU, không đánh số, không gộp — dữ liệu thô có thể trùng hệt hoặc khác nhau, và (a) trung thực với cả hai ca mà ⛔ không cần một logic phát hiện "trùng vs khác" ở webview (chính là hình dạng tư duy hợp nhất mà AD-19 cấm, chỉ nhỏ hơn).<br><br>**Quyết định #6** — CHỐT theo mặc định đề xuất **(a)**: vùng đầu mục = đầu mục + thanh nhịp, hằng có tên cho chiều cao.<br><br>**Mâu thuẫn tài liệu #4 (vạch trái cấp nào)** — CHỐT theo **mockup** (`lookup-real-density.html:68`): vạch trái 2px + thụt 13px ở **CẤP NGHĨA** (`.sense`), khối nguồn (`.src`/`.srch`) chỉ có `border-bottom` dưới nhãn. `epics.md`/`DESIGN.md` nói "cấp nguồn" nhưng mockup là bản vẽ mật độ thật và nhiều vạch giúp mắt nhặt ranh giới nghĩa — đúng lý do story đề xuất. Ghi ra, ⛔ sửa tài liệu quy hoạch.<br><br>Quyết định #4/#7 giữ nguyên như Ice đã chốt, chỉ bổ sung số đo cho hai hệ quả còn mở. |
| 2026-08-06 | ✅ **Ice chốt câu #4: nếu `LIMIT` ⛔ không cắt được thời gian ⇒ DỪNG và tìm phương án** *(⛔ không đi tiếp chỉ vì `LIMIT` "đã được chốt")*. ⇒ story mang sẵn **bốn phương án ứng viên đã khảo sát**, kèm một dữ kiện lược đồ mà ⛔ chưa tài liệu nào ghi: `char_idx` khai **`PRIMARY KEY (ch, entry_id) WITHOUT ROWID`**, nên subquery `WHERE ch = ?1` **đã trả `entry_id` sắp sẵn** — **chỗ duy nhất trong cả sáu câu SQL mà `LIMIT` chắc chắn dừng sớm được** *(⇒ phương án **C** là đề xuất)*. Cộng **Bẫy 11** mới: `LIMIT` đặt **trước** `verify_substring` ở nhánh 2 ký tự cho ra trang **ít hơn `N`** và một dòng *"còn M nữa"* **nói dối**. ⚠️ Và điều kiện *"nếu cần"* phải xét **trước**: đầu-cuối < 100 ms ⇒ NFR1 đã đạt ⇒ ⛔ không phương án nào phải chạy. |
| 2026-08-06 | ✅ **Ice chốt ba câu hỏi.** **#1** *(mục từ tiếng Anh)* — **tạm dùng mặc định của story** *(cùng cấu trúc khối, khác token đầu mục)*; chủ sở hữu **vẫn là Sally (`bmad-ux`)** và mục `deferred-work.md:317` **⛔ không đóng** — story này ghi lại nó là một lựa chọn **tạm**, ⛔ không phải một chữ ký UX. **#2** ⇒ **Quyết định #7 = (b)**: thêm **token thứ 17 `ui-md-wrap`** (1.66) qua sổ `deviations`, và **áp cho cả ba chỗ dùng đã có** *(`.load-error` · `.parallel-note` của `SourcePanel.vue`, `.hv-notice` của `SourceHanViet.vue`)* — chỉ áp cho chuỗi mới thì món nợ `:115` ⛔ không đóng. **#3** ⇒ **Quyết định #4 = có `LIMIT` pha một**. 🔴 **Ba hệ quả mới phát hiện khi đối chiếu quyết định đó với `query.rs` thật, ⛔ không có trong bản story đầu:** *(i)* cả **sáu** hình dạng SQL kết bằng `ORDER BY e.id` — `LIMIT` chỉ cắt được thời gian nếu kế hoạch truy vấn **⛔ không** phải sắp toàn bộ trước, và điều đó **phải đo**, ⛔ không suy; *(ii)* `dict-core.db` mang **sáu** nguồn trong **một** tệp ⇒ một `LIMIT` cấp-tệp có thể **xoá sạch một nguồn** khỏi kết quả, im lặng — **FR31 vỡ, ⛔ không cổng nào đỏ** *(⇒ **AC12** mới)*; *(iii)* thanh nhịp của Quyết định #6 hứa **số đếm chính xác** *(`5 nguồn · 22 nghĩa`)*, mà `LIMIT` lấy mất chúng ⇒ phải chọn giữa một truy vấn `COUNT` thêm hay đổi ngữ nghĩa thanh nhịp. |
| 2026-08-06 | Tạo story. Baseline `cb03974`, cây làm việc **sạch**. Phân tích: `epics.md` §Story 1.17 + §Story 1.11/1.13/1.16/1.18/1.19/1.20 *(ranh giới hai đầu)* + FR28–FR41 · `ARCHITECTURE-SPINE.md` *(AD-2 · AD-10 · AD-19 · AD-21 · AD-25 · AD-26 · AD-34 · AD-44)* · `DESIGN.md` §Typography + §Motion + §Shapes + §Components · `EXPERIENCE.md` §Trạng thái + §Trạng thái rỗng · `mockups/lookup-real-density.html` *(279 dòng, mật độ THẬT — chữ `打`, 22 nghĩa, 5 nguồn)* · story `1-16`, `1-13`, `1-14` · **toàn bộ `deferred-work.md`** *(**tám** mục gọi đích danh Story 1.17)* · mã thật `src-tauri/src/core/dict/**` + `src-tauri/src/ports/dict_source.rs` + `src/panels/**` + `scripts/*.mjs`. **Phát hiện:** ⛔ **không** một kiểu bản ghi tra cứu nào derive `Serialize` *(⇒ đường dây chưa tồn tại, và hình dạng của nó là Quyết định #2)* · ba token `lookup-*` **có trong bảng nhưng 0 người tiêu thụ** *(story này là người đầu tiên)* · ⛔ **không** đường kích hoạt nào cho một lượt tra mà ⛔ không lấn sang hợp đồng vùng chọn của 1.18 *(Quyết định #1)* · **12,569 ms** pha một trên ba tệp thật là món nợ NFR1 mà `deferred-work.md` giao đích danh story này *(Quyết định #4)* · **bốn mâu thuẫn tài liệu** *(Concordance · dải tab · trích dẫn FR41 sai · **vạch trái treo ở cấp nguồn hay cấp nghĩa**)*. |

**Epic:** 1 — Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì
**Story key:** `1-17-panel-lookup-ban-ghi-co-cau-truc`
**Covers:** **FR28** *(bản ghi có cấu trúc: **nguồn · từ loại · nghĩa · ví dụ[] · trích dẫn[] · ghi chú**, ⛔ không phải một đoạn văn)* · **FR32** *(nguồn bất đồng ⇒ **hiển thị đồng thời**, ⛔ không hợp nhất)*
**Nhận nợ hiển thị từ 1.13:** **FR29** *(nhiều từ loại ⇒ nhiều mục riêng)* · **FR30** *(ví dụ gắn theo TỪ LOẠI; trích dẫn là trường RIÊNG có xuất xứ)* · **FR31** *(mọi định nghĩa hiển thị nguồn — ⛔ không ngoại lệ, ⛔ không chế độ ẩn nguồn)* · **FR34/FR35** *(mục tiếng Anh có từ loại + nghĩa tiếng Việt; nhãn từ loại **ngoại ngữ** phải **đánh dấu rõ**)*
**Governed by:** **AD-19** *(⛔ **không tồn tại** bước hợp nhất nguồn từ điển — ở **bất kỳ tầng nào**, kể cả tầng trình bày)* · **AD-21 / NFR16** *(Rust ⛔ không bao giờ trả văn bản hiển thị)* · **AD-2** *(đúng ba cổng — ⛔ không cổng thứ tư)* · **AD-1** *(quy tắc nghiệp vụ ở Rust; webview render + giữ state UI)* · **AD-16** *(nội dung ngoài ⛔ không render thành HTML — ⛔ không `v-html`)* · **AD-34** *(mọi thao tác qua `CommandRegistry`; ⛔ cấm màu viết thẳng; sàn NFR17 là cấu trúc)* · **AD-44 ④** *(*"rỗng **có lý do**"* ⛔ không được trông giống *"⛔ không có kết quả"*)* · AD-10 · AD-25
**UX-DR phải tôn trọng:** **UX-DR12** *(phân vai hai họ chữ — mục từ và nghĩa là **nội dung**, họ `read`; nhãn nguồn là **bộ máy**, họ `ui`)* · **UX-DR10** *(giãn dòng 1.66 là sàn cứng cho chữ nội dung)* · **UX-DR5** *(`ornament` là màu của **nét**, ⛔ không của chữ)* · **UX-DR6** *(⛔ không `opacity` để làm mờ chữ)* · **UX-DR16** *(⛔ không elevation)* · UX-DR8/UX-DR17 *(hợp đồng tiêu điểm — **đã dựng ở 1.6/1.14, ⛔ không đụng**)* · UX-DR27 *(⛔ không trạng thái rỗng câm)*
**Ràng buộc xuôi dòng phải để lại chỗ đứng:** **FR21 / Story 1.18** *(Auto-Lookup + **hợp đồng vùng chọn dùng chung cho mọi panel văn bản** + hiệu ứng 90 ms + vạch tiến trình 250 ms)* · **FR37, FR38 / Story 1.19** *(bật/tắt từng nguồn + ghi công)* · **FR41 / Story 1.20** *(lịch sử tra cứu + ghim — **tab thứ ba** của panel này)* · **FR64 / Story 7.7** *(Concordance — **đường sang nó ⛔ CHƯA tồn tại**, và AC của epic cấm trỏ tới)* · **FR50 / Story 3.4** *(đánh dấu thuật ngữ Glossary)* · **FR113 / Story 3.7** *(đề xuất bản dịch bằng âm Hán Việt)* · **Story 4.12** *(Tra cứu rút về thanh trạng thái ở màn hình hẹp)* · **Story 10.4** *(màn hình Attribution)*
**NFR:** **NFR1** *(tra cứu < 100 ms đầu-cuối — story này là chỗ đầu tiên đo được **đầu-cuối**)* · NFR13 *(ngoại tuyến)* · NFR14 *(hai nền tảng)* · NFR15 *(⛔ 0 phụ thuộc mới)* · NFR16 *(chuỗi ở `vi.json`)* · NFR17 *(bàn phím)*
**Ngày tạo:** 2026-08-06

---

## 🔴 ĐỌC TRƯỚC TIÊN — SÁU VIỆC STORY NÀY ⛔ KHÔNG LÀM

### ① ⛔ KHÔNG dựng Auto-Lookup, ⛔ không dựng hợp đồng vùng chọn dùng chung

`epics.md` Story 1.18 nói nguyên văn: cơ chế Auto-Lookup *"gắn vào một **hợp đồng vùng chọn
dùng chung cho mọi panel văn bản**"*, và Panel AI Translation + Editor *"nhận được cùng hành
vi khi chúng có nội dung ở các epic sau, **⛔ không cần cài lại**"*. Đó là một trừu tượng
phải thiết kế **một lần cho bốn panel**.

⛔ **Và ⛔ không dựng bốn thứ đi kèm nó**: hiệu ứng 90 ms `opacity` 0.4→1 · huỷ hiệu ứng khi
tra chồng · cuộn về đầu tức thì · **vạch tiến trình mảnh khi vượt 250 ms**. Cả bốn nằm ở AC
của **Story 1.18**, ⛔ không phải story này. ⚠️ Đừng nhầm vạch tiến trình 250 ms của 1.18 với
**AC7 của story này** *(⛔ không spinner, ⛔ không trạng thái "đang tải")* — chúng ⛔ không
mâu thuẫn: 1.17 cấm spinner **vĩnh viễn**, 1.18 thêm một vạch **⛔ không phải spinner**.

⇒ Story này cần **một đường kích hoạt tối thiểu** để chính nó nghiệm thu được, và hình dạng
của đường đó là **Quyết định #1** — nó phải **⛔ không** phải là hợp đồng vùng chọn.

### ② ⛔ KHÔNG dựng bật/tắt nguồn — FR37/FR38 là Story 1.19

`mockups/lookup-real-density.html:121` **có** vẽ chip `en.wiktionary · tắt` với viền đứt. Đó
là hình ảnh của **sản phẩm hoàn chỉnh**. Story này hiện **mọi nguồn có kết quả**, ⛔ không
một công tắc nào, ⛔ không một trạng thái `off` nào.

🔴 **Nhưng ràng buộc phải để lại:** thanh nhịp *(Quyết định #6)* phải là một danh sách **dẫn
xuất từ `groups`**, ⛔ không phải một danh sách nguồn viết cứng — 1.19 chỉ được phép **thêm
một cờ** vào từng chip, ⛔ không phải dựng lại cả thanh.

### ③ ⛔ KHÔNG dựng lịch sử và ghim — FR41 là Story 1.20

Mockup vẽ `📌 Ghim ⌘D` ở góc phải đầu mục và một dải tab `Từ điển | Concordance | F3`. **Cả
ba ⛔ không thuộc story này**: ghim và lịch sử là **1.20** *(tab thứ ba của chính panel này)*,
Concordance là **7.7**.

🔴 **Và AC của epic cấm đích danh:** *"⛔ **không trỏ tới bất kỳ năng lực nào chưa tồn tại**
— đường sang Concordance được bổ sung ở Story 7.7."* ⇒ trạng thái *không tìm thấy* **⛔ không
được** nhắc chữ *Concordance*, dù `EXPERIENCE.md:389` có nói tới nó — xem §Mâu thuẫn tài liệu #1.

### ④ ⛔ KHÔNG cài lại việc đọc từ điển — mọi đường đi qua cổng `DictionarySource`

Đường tra cứu **đã tồn tại đầy đủ** ở Rust từ Story 1.11/1.11b/1.13: `pick_route` ·
`pick_branch` · `lookup_grouped` *(pha một)* · `DictionarySource::senses` *(pha hai)*. Story
này **gọi chúng**, ⛔ không viết lại một câu SQL nào, ⛔ không gõ tên `rusqlite` ngoài
`core/store/**` *(`store_boundary.rs::only_core_store_may_name_rusqlite` cưỡng chế bằng máy)*.

⛔ **Và ⛔ không thêm cổng thứ tư** (AD-2). Thứ story này thêm là **một vỏ IPC** — đúng tiền
lệ `commands::dict::wire::read_han_viet` của Story 1.16, ghi ở doc-comment đầu tệp đó.

### ⑤ ⛔ KHÔNG hợp nhất, ⛔ không xếp hạng, ⛔ không chọn "câu trả lời đúng"

AD-19 là mệnh đề **của cả sản phẩm**, ⛔ không chỉ của tầng dữ liệu: *"Hai nguồn bất đồng về
cùng một đầu mục ⇒ **cả hai nhóm có mặt**, nghĩa giữ nguyên, ⛔ không nhóm nào bị chọn làm
*câu trả lời*."*

⛔ Không sắp xếp nguồn theo "độ tin cậy". ⛔ Không gộp hai nghĩa giống nhau của hai nguồn.
⛔ Không một dòng tóm tắt nào đứng **thay** cho các nguồn. Dòng dẫn của **AC5** nói *"hai
nguồn ghi khác nhau"* — nó ⛔ **không** nói nguồn nào đúng.

⚠️ `tests/dict_boundary.rs::no_function_merges_meanings_across_sources` là một cổng **đếm
tệp**. Nó ⛔ không nhìn thấy `src/**`. ⇒ ở tầng webview, mệnh đề này là **kỷ luật + một ca
test hành vi**, ⛔ không phải một cổng tự động — xem Bẫy 2.

### ⑥ ⛔ KHÔNG đụng ngưỡng màn hình hẹp, ⛔ không đụng thanh trạng thái

*"Tra cứu rút về thanh trạng thái, ⛔ không bao giờ mất hẳn"* (UX-DR15) là **Story 4.12** —
doc-comment đầu `LookupPanel.vue` đã ghi sẵn. Cây `src/**` hôm nay có **0** lời gọi
`matchMedia` và **0** lời gọi `window.innerWidth`; con số đó phải **giữ nguyên**.

---

## Story

As a người dịch,
I want kết quả tra cứu hiện thành **bản ghi có cấu trúc** chứ ⛔ không phải một đoạn văn,
So that mắt tôi nhặt được thứ cần trong một giây.

---

## Ranh giới phạm vi — ĐỌC TRƯỚC KHI GÕ DÒNG ĐẦU TIÊN

| Trong phạm vi | ⛔ Ngoài phạm vi (và ai sở hữu) |
|---|---|
| **Đường IPC tra cứu từ điển** — hai pha, `DictLayers` lấy từ state, hàm thuần + vỏ mỏng | Cổng thứ tư *(AD-2 — ⛔ cấm)* · viết lại truy vấn *(đã có ở 1.11/1.11b/1.13)* |
| Hình dạng bản ghi **trên dây** *(Quyết định #2)* | Chuỗi hiển thị từ Rust *(AD-21 — ⛔ cấm)* |
| Nội dung thân Panel Lookup: khối nguồn · nghĩa · ví dụ · trích dẫn · ghi chú · nhãn ngoại ngữ | Auto-Lookup + hợp đồng vùng chọn *(1.18)* · hiệu ứng 90 ms · vạch tiến trình 250 ms *(1.18)* |
| **Vùng đầu mục chiều cao cố định** + thanh nhịp *(Quyết định #6)* | Ghim `⌘D` · lịch sử · tab thứ ba *(1.20)* · tab Concordance *(7.7)* |
| Bốn trạng thái rỗng **phân biệt được** *(AC6)* | Bật/tắt nguồn · chip `tắt` *(1.19)* · màn hình Attribution *(10.4)* |
| Ba token `lookup-*` lần đầu có người tiêu thụ — đóng nửa Lookup của `deferred-work.md:129`/`:131` | Đổi bảng token màu · đổi cờ `wraps` của `ui-md` *(⛔ **quyết định của Ice** — `deferred-work.md:115`, lần thứ **TƯ**)* |
| Trạng thái panel sống sót qua đổi preset + **reset khi đổi Tác phẩm** *(khuôn `sourcePanelState.ts`)* | Ngưỡng màn hình hẹp *(4.12)* · preset người dùng đặt tên *(1.21)* |
| 🔴 **Đo NFR1 ĐẦU-CUỐI lần đầu tiên** + quyết định trần trang **từ số đo** *(Quyết định #4)* | Đánh dấu Glossary *(3.4)* · đề xuất âm Hán Việt *(3.7)* |

**⛔ KHÔNG ĐỤNG** *(ranh giới đã chốt sáu story liên tiếp)*: `tools/**` · `dict-manifest.toml`
· `src-tauri/capabilities/main.json` *(thêm permission ⇒ **phải là một AD mới trước đã**)* ·
`package.json` *(⛔ **0 phụ thuộc npm mới** — NFR15)* · `src-tauri/Cargo.toml` *(⛔ chốt lần
thứ **sáu**)* · `[profile.release]` · `_bmad-output/planning-artifacts/**` *(`epics.md` ·
`prd.md` · `DESIGN.md` · `EXPERIENCE.md` · `mockups/**` — **lệch thì GHI RA, ⛔ không sửa**;
tiền lệ quyết định #3 Story 1.3)* · `src/panels/SourceHanViet.vue` + `sourcePanelState.ts`
*(⛔ **không** refactor — 1.16 vừa đóng, mọi đường chung phải là **thêm**, ⛔ không **sửa**)*.

⚠️ **MỘT ngoại lệ tường minh, và ⛔ chỉ một:** khối `<style>` của `SourcePanel.vue` và
`SourceHanViet.vue` được đụng **đúng ba lớp** — `.load-error` · `.parallel-note` ·
`.hv-notice` — để áp **token thứ 17** *(Quyết định #7, Ice chốt 2026-08-06)*. ⛔ **Không**
một dòng `<script>` nào, ⛔ không đổi cấu trúc `<template>`.

**✅ ĐỤNG ĐÃ ĐƯỢC PHÉP** *(và ⛔ chỉ khi Ice chốt Quyết định tương ứng)*:
`src-tauri/src/core/dict/mod.rs` + `layer.rs` + `query.rs` *(⚠️ **chỉ** khi Quyết định #2/#4
đòi — derive `Serialize`, và **`LIMIT` chỉ khi số đo đòi**)* · `src-tauri/src/ports/dict_source.rs`
*(⚠️ **chỉ** nếu #4 = có `LIMIT`, và lúc đó là **đổi chữ ký**, ⛔ không thêm method)* ·
`src/panels/PanelFrame.vue` *(prop `show-status` **đã có** — chỉ dùng, ⛔ không sửa)* · các
hằng `*_FLOOR` của bốn script cổng *(AC13)*.

---

## 🔴 BẢY QUYẾT ĐỊNH — PHẢI CHỐT Ở TASK 0, TRƯỚC DÒNG MÃ ĐẦU TIÊN

> Khuôn Task 0 của Story 1.13/1.14/1.15/1.16. Mỗi mục có **mặc định đề xuất** kèm lý do; ⛔
> không mục nào được cài theo cảm tính. **#1, #2, #4 chặn thật** — chúng đổi hình dạng dữ
> liệu trên dây hoặc hình dạng chữ ký cổng, và sửa sau là mổ lại cả ba tầng.

### Quyết định #1 — **Đường kích hoạt một lượt tra, mà ⛔ không lấn sang 1.18**

Story này phải nghiệm thu được *"một kết quả tra cứu hiển thị"*, nhưng §⛔KHÔNG-LÀM ① cấm
dựng hợp đồng vùng chọn. Ba đường:

- **(a) — MẶC ĐỊNH ĐỀ XUẤT.** `lookupPanelState.ts` export **một** hàm vào duy nhất
  `runLookup(query: string)`. Một command mới `lookup.lookup_selection` *(có phím)* gọi nó,
  và **chỗ lấy vùng chọn là một dep tiêm vào** — `deps.currentSelection?: () => string`,
  cắm ở `src/main.ts`, cùng cửa mà `applyPreset`/`togglePanel`/`selectSourceTab` đã đi qua
  *(`src/commands/index.ts` §`CommandDeps`)*. Story 1.18 **thay đúng một dep đó** bằng hợp
  đồng vùng chọn thật và **⛔ không** phải chạm `runLookup`, ⛔ không phải chạm component.
  ⇒ Ranh giới nằm ở **một hàm**, ⛔ không rải ra.
- **(b)** ⛔ Không đường kích hoạt nào — panel chỉ hiện trạng thái *chưa tra gì*. ⛔ **Loại:**
  AC1–AC5 *(hình dạng bản ghi)* trở thành thứ ⛔ không nghiệm thu được bằng bất kỳ phép chạy
  nào, và một AC ⛔ không nghiệm thu được là một AC được đánh dấu đạt bằng lời hứa.
- **(c)** Một ô nhập truy vấn trong panel. ⛔ **Loại:** ⛔ không mockup nào có nó, ⛔ không FR
  nào đòi nó, và nó là một bề mặt phải gỡ đi ở 1.18 — tức việc dựng **và** việc gỡ, cả hai
  đều tính tiền.

⚠️ **(a) ⛔ không phải Auto-Lookup**: nó đòi **một thao tác tường minh** *(một phím)*, đúng
thứ FR21 tồn tại để **xoá bỏ**. Đó là lý do nó ⛔ không lấn phạm vi 1.18.

### Quyết định #2 — **Hình dạng bản ghi TRÊN DÂY** *(chặn thật)*

🔴 **Số đo:** ⛔ **không** một kiểu nào trong `core/dict/mod.rs` liên quan tra cứu derive
`Serialize` — `SourceInfo` · `EntryHit` · `SenseRecord` · `ExampleRecord` · `CitationRecord`
· `SourceGroup` · `GroupedLookup` · `QueryRoute` · `QueryBranch` · `SkippedLayer` ·
`SkipReason` đều là `#[derive(Debug, Clone, PartialEq, Eq)]` **và không hơn**. Ba kiểu Hán
Việt của Story 1.16 *(`CharacterReading` · `HanVietReading` · `HanVietLookup`)* **thì có** —
tiền lệ đã đứng sẵn.

- **(a) — MẶC ĐỊNH ĐỀ XUẤT.** Derive `serde::Serialize` thẳng trên các kiểu bản ghi đã
  `pub`, **⛔ không** `#[serde(rename_all)]` *(mọi trường đã `snake_case`, đúng như trên dây
  — AD-21, cùng luật `HanVietLookup`)*. Hai ngoại lệ **bắt buộc**:
  1. `QueryBranch`/`QueryRoute` đi ra dưới dạng **chuỗi định danh máy** *(`"exact_btree"` ·
     `"char_idx"` · `"fts_trigram"` · `"query_too_short"`)*, ⛔ không phải số thứ tự biến
     thể — một `usize` trên dây là thứ đảo nghĩa im lặng khi ai đó chèn một biến thể.
  2. 🔴 **`SkipReason` ⛔ KHÔNG đi nguyên vẹn.** Bốn biến thể của nó mang trường `detail:
     String` — **lỗi thô của SQLite**, và doc-comment tại chỗ ghi rõ *"⛔ Không đi lên giao
     diện"*. Đưa nó qua dây là vi phạm AD-21 ở đúng chỗ khó thấy nhất. ⇒ trên dây,
     `skipped` là **số lượng** cộng một **mã máy** cho mỗi mục *(`"open_failed"` ·
     `"schema_too_new"` · …)*, ⛔ không `path`, ⛔ không `detail`. Panel chỉ cần nói *"một
     phần từ điển ⛔ không trả lời"*; nó ⛔ không cần biết tệp nào hỏng thế nào.
- **(b)** Một tầng DTO riêng trong `commands/dict.rs`. ⛔ **Cân nhắc nhưng ⛔ không mặc
  định:** nó là **hai từ vựng cho một khái niệm** — đúng thứ doc-comment `ports/dict_source.rs`
  §"KIỂU BẢN GHI SỐNG Ở `core::dict`" cấm — và hai từ vựng **sẽ** trôi khỏi nhau.
  ⚠️ Nhưng ngoại lệ ở (a).2 **chính là** một mẩu DTO. Nếu Task 0 thấy số mẩu DTO vượt hai,
  chọn (b) cho **trọn** thay vì trộn hai đường.

⚠️ **`display_name` là DỮ LIỆU, ⛔ không phải chuỗi giao diện.** Nó đọc từ `dict_source` của
**chính tệp `.db`** *(`SourceInfo::display_name`)*, cùng hạng với `chapter.source_text` —
`vi.json` ⛔ không mang tên nguồn, và AD-21 ⛔ không bị vi phạm. 🔴 **Và đây là khác biệt so
với đường Hán Việt của 1.16**, nơi `sources_used` chỉ mang `code` thô: ở đây `SourceGroup`
đã cầm sẵn `SourceInfo` đầy đủ, nên **FR31 thoả bằng tên hiển thị thật**, ⛔ không phải bằng
`fx-hv`/`thieu-chuu`.

### Quyết định #3 — **`LookupMode` của một lượt tra: `Exact` hay `Substring`**

`LookupMode` là **tham số từ chỗ gọi** *(doc-comment `core/dict/mod.rs`: "⛔ không đoán từ
nội dung truy vấn")*, và story này **là** chỗ gọi đầu tiên.

- **(a) — MẶC ĐỊNH ĐỀ XUẤT: `Exact`.** Người dùng bôi đen **một cụm** ⇒ họ hỏi *"cụm này
  nghĩa gì"*, ⛔ không hỏi *"đầu mục nào chứa cụm này"*. Câu hỏi thứ hai là **Concordance
  (7.7)**, và nó có bề mặt riêng.
- **(b)** `Substring`. ⛔ **Loại làm mặc định:** nhánh `char_idx` một ký tự trả **3.177** đầu
  mục *(số đo thật, `dict-core.db`)* — một bức tường 3.000 mục cho một lượt bôi đen một chữ.
- ⛔ **⛔ KHÔNG fallback dây chuyền** *(thử `Exact`, rỗng thì thử `Substring`)*. Doc-comment
  `core/dict/mod.rs` cấm đích danh: nó làm mỗi lượt tra chạy hai-ba truy vấn *(⇒ số đo NFR1
  thành vô nghĩa)* và làm `LookupResult::branch` **nói dối** về đường đã đi.
- ⚠️ **Hệ quả phải nhìn thẳng:** với `Exact`, một cụm ⛔ không có trong từ điển trả **rỗng**.
  Đó ⛔ **không phải** một lỗ hổng — **AC6** dựng đúng trạng thái đó *(gợi ý tra **từng chữ**
  trong cụm vừa chọn)*, và `epics.md` viết nó ra trước khi có một dòng code nào.

### ✅ Quyết định #4 — ĐÃ CHỐT 2026-08-06 (Ice): **thêm `LIMIT` pha một**

🔴 `deferred-work.md` giao đích danh story này, **ba lần**, ở ba lượt review khác nhau:

| Dòng | Nội dung | Số đo |
|---|---|---|
| `:343` | *"NFR1: nhánh 2 một ký tự đã VƯỢT dải công bố của AD-26"* | trần 10 ms còn **27 %** dư địa *(một tệp)* |
| `:419` | *"NFR1 trên **ĐƯỜNG GOM**: nhánh 2 một ký tự VƯỢT trần — **12,569 ms** bản release"* | ba tệp `.db` **thật**, 200 lượt |
| `:446` | Pha hai: một trang 20 đầu mục hydrate hết **0,29–0,32 ms**; hydrate **cả** 3.385 đầu mục hết **13,015 ms** | |

Và `:449` khai hình dạng đường ra: *"giới hạn số hàng **pha một** *(phân trang + đếm)* là
thứ **duy nhất** chạm được vào 12,569 ms. Pha hai **⛔ không cần** làm gì."*

🔴 **CHỐT: có `LIMIT` ở pha một.** Ice đã chọn đúng câu này hai lần trước *(review 1.11 và
1.13)* theo hướng ngược lại — **chấp nhận nguyên trạng** — cả hai lần với lý do *"Panel
Lookup chưa tồn tại"*. Lý do đó hết hiệu lực ở story này, và lượt chốt 2026-08-06 lật nó.

**Hai mức, và story này làm CẢ HAI:**
1. **`LIMIT` ở pha một** — đổi chữ ký `DictionarySource::lookup`, đổi cả **sáu** hình dạng
   SQL trong `query.rs`, đổi `lookup`/`lookup_with_branch`/`lookup_grouped`, đổi mọi test
   hành vi đang gọi chúng. **Đây là phần đắt nhất của story.**
2. **Trần RENDER + HYDRATE ở webview** — hiện `N` đầu mục mỗi nguồn, phần còn lại là một
   dòng *"còn M nữa"* *(đúng `mockups/…:151` — `còn 6 nghĩa nữa ⌄`)*. Pha hai chỉ hydrate
   đúng phần đang hiện *(0,3 ms — số đo `:446`)*.

⚠️ **Số đo vẫn phải chạy** — nó ⛔ không còn quyết *có hay không*, nhưng nó quyết **cỡ trang
`N`** và nó là thứ duy nhất chứng minh `LIMIT` **thật sự** mua được thời gian *(xem hệ quả
① ngay dưới)*. AC11 giữ nguyên.

#### 🔴 BA HỆ QUẢ CỦA QUYẾT ĐỊNH NÀY — cả ba phát hiện khi đối chiếu với `query.rs` THẬT

**① `LIMIT` có thể ⛔ KHÔNG cắt được một mili-giây nào, và điều đó phải ĐO.**
Cả **sáu** câu SQL kết bằng **`ORDER BY e.id`** *(`query.rs:121, 167, 180, 207, 254, 281`)*.
`e.id` là `INTEGER PRIMARY KEY` *(= rowid)*, nên **nếu** kế hoạch truy vấn duyệt được theo
thứ tự rowid thì `LIMIT` dừng sớm và 12,569 ms giảm thật. **Nhưng** nhánh `char_idx` lọc
bằng `WHERE e.id IN (SELECT entry_id FROM char_idx WHERE ch = ?1)` **cộng** một `JOIN` sang
`dict_source` — kế hoạch rất có thể dựng một b-tree tạm để sắp, và **một phép sắp thì phải
đọc hết trước khi trả hàng đầu tiên**. Trong ca đó `LIMIT` cắt **băng thông**, ⛔ **không**
cắt **thời gian**.
⇒ Task 0 phải chạy `EXPLAIN QUERY PLAN` cho nhánh `char_idx` **trước** khi viết mã, và Task 8
phải đo **trước/sau**. 🔴 Nếu số đo nói `LIMIT` ⛔ không cắt thời gian, **⛔ đừng im lặng thi
hành** — ghi số ra và hỏi lại Ice *(đúng tiền lệ: quyết định kiến trúc ở dự án này đi theo
số đo, ⛔ không theo dự đoán)*.

**② 🔴 `LIMIT` cấp-TỆP có thể XOÁ SẠCH một nguồn, im lặng — FR31 vỡ, ⛔ không cổng nào đỏ.**
`dict-core.db` mang **sáu** nguồn trong **một** tệp *(doc-comment `DictionarySource::sources`)*.
`LIMIT N` chạy **trên tệp**, sắp theo `e.id`, ⛔ **không** biết gì về `source_id`. ⇒ một
nguồn mà mọi đầu mục khớp đều mang `id` lớn **⛔ không có một hàng nào** trong `N` hàng đầu —
nó biến mất khỏi `groups`, và `SourceGroup` *"⛔ không bao giờ rỗng"* nên ⛔ **không có chỗ
nào** ghi lại rằng nó từng tồn tại. Người dùng đọc *"4 nguồn"* trong khi sự thật là 5.
**Đây là đúng lớp lỗi mà AD-19/FR31 tồn tại để chặn, chỉ đến từ hướng ⛔ không ai canh.**
Ba đường ra, **chốt ở Task 0 sau khi đo**:
- **(a) — ĐỀ XUẤT: `LIMIT` theo TỪNG NGUỒN**, bằng `ROW_NUMBER() OVER (PARTITION BY
  e.source_id ORDER BY e.id) <= ?N`. Đúng ngữ nghĩa sản phẩm *(mỗi nguồn một trang)*, và
  trùng với mức 2 *(mỗi khối nguồn hiện `N` mục)*. ⚠️ Đổi lại: hàm cửa sổ **⛔ không dừng
  sớm được** ⇒ nó có thể ⛔ không mua được gì cho hệ quả ①. Đo rồi mới biết.
- **(b) `LIMIT` cấp-tệp + một cờ `truncated`** trên mỗi tệp, và panel nói *"danh sách nguồn
  ⛔ chưa đầy đủ"*. Rẻ, trung thực, nhưng làm thanh nhịp thành một **cận dưới**.
- **(c)** `LIMIT` chỉ áp cho nhánh `char_idx` *(nhánh duy nhất có số đo vượt trần)*, các
  nhánh khác giữ nguyên. Hẹp nhất, nhưng thêm một mệnh đề *"nhánh nào có trần"* phải nhớ.

**③ Thanh nhịp hứa SỐ CHÍNH XÁC, và `LIMIT` lấy mất chúng.**
Quyết định #6 *(và `mockups/…:116`)* hứa `5 nguồn · 22 nghĩa` cùng số nghĩa cho **từng**
nguồn. Sau `LIMIT`, con số đó là *"số đã lấy về"*, ⛔ không phải *"số có thật"*. Ba đường:
- **(a) — ĐỀ XUẤT:** một truy vấn `COUNT(*) … GROUP BY source_id` cho **mỗi tệp**, chạy cùng
  lượt với pha một. ⚠️ Nó **cũng** phải quét — tức nó có thể trả lại đúng chi phí mà `LIMIT`
  vừa cắt. **Đo cả hai cùng lúc ở Task 8**, ⛔ đừng cài rồi mới đo.
- **(b)** Lấy `N + 1` hàng: biết *"còn nữa"* mà ⛔ không tốn một truy vấn nào — nhưng chỉ nói
  được *"còn nữa"*, ⛔ không nói được *"còn 6"*. ⇒ thanh nhịp đổi sang `5 nguồn · 20+ nghĩa`.
- **(c)** Bỏ số đếm khỏi thanh nhịp. ⛔ **Loại:** *"biết hình dạng trước khi cuộn"* là toàn
  bộ lý do thanh nhịp tồn tại *(`mockups/…:272`)*.

#### ✅ NẾU ① ĐÚNG — Ice chốt 2026-08-06: **DỪNG và tìm phương án**

🔴 **⛔ Không tự đi tiếp với một `LIMIT` ⛔ không mua được gì.** Dev **dừng thi hành**, khảo
sát các đường dưới đây trên `EXPLAIN QUERY PLAN` thật, **ghi số**, đề xuất — rồi **hỏi Ice
trước khi cài**. Bốn ứng viên đã khảo sát sẵn ở lượt dựng story; ⛔ đừng nghĩ lại từ đầu.

🔴 **Một dữ kiện quyết định mà ⛔ chưa tài liệu nào ghi:** `char_idx` khai
**`PRIMARY KEY (ch, entry_id) WITHOUT ROWID`** *(`tools/dict-build/src/schema.rs:114-118`)*.
⇒ `SELECT entry_id FROM char_idx WHERE ch = ?1` **đã trả về `entry_id` sắp sẵn tăng dần**,
đọc thẳng theo tiền tố khoá chính — ⛔ **không** có phép sắp nào ở đó. **Đó là chỗ duy nhất
trong cả sáu câu SQL mà một `LIMIT` chắc chắn dừng sớm được.**

| | Phương án | Cắt được gì | Giá | Rủi ro |
|---|---|---|---|---|
| **C** | 🔴 **ĐỀ XUẤT** — `LIMIT` **vào trong subquery `char_idx`**: `… WHERE ch = ?1 ORDER BY entry_id LIMIT ?N` | Cắt **thật** — dừng sớm trên khoá chính; `JOIN dict_source` sau đó chỉ chạy `N` lần thay vì 3.177 | Nhỏ — chỉ nhánh `char_idx`, ⛔ không đụng năm câu còn lại | 🔴 **`AND e.lang = 'zh'` áp SAU khi cắt** ⇒ có thể trả ít hơn `N` *(hoặc 0)* dù còn hàng `zh` phía sau — **rỗng im lặng**, đúng lớp lỗi bị cấm. ⚠️ Đo được: lớp `en` chỉ đóng góp **9** cặp `char_idx` trên **1.341.179** *(0,00067 %)*, nên xác suất thực tế gần 0 — **nhưng "gần 0" ⛔ không phải "không"**. Guard: đẩy bộ lọc `lang` **vào** subquery, hoặc lấy `N + biên` rồi cắt sau |
| **B** | Bỏ `JOIN dict_source` khỏi pha một — `SELECT e.source_id` rồi ánh xạ `source_id → code` ở tầng gom qua `DictLayer::sources()` **đã cache sẵn** | Bỏ một `JOIN` trên 3.177 hàng; có thể mở đường cho kế hoạch chạy thẳng theo rowid | Vừa — đổi `COLUMNS`, đổi `run()`, ⛔ không đổi `EntryHit` *(vẫn `source_code: String`)* | ⚠️ Nhìn **giống** vi phạm *"khoá theo `code` chứ ⛔ không theo `id`"* nhưng **⛔ không phải**: ánh xạ nằm **trong phạm vi MỘT tệp**, và nguồn ánh xạ là bảng `dict_source` **của chính tệp đó**. 🔴 Mệnh đề bị cấm là khoá `id` **XUYÊN tệp**. ⇒ nếu chọn B, phải **viết mệnh đề đó ra** cạnh mã và thêm một ca test dựng hai tệp có `source_id = 1` trỏ hai nguồn khác nhau |
| **D** | Thêm chỉ mục phủ *(vd. `dict_entry(source_id, id)`)* | Nhiều | 🔴 **Rất đắt** — chạm `tools/dict-build/src/schema.rs` ⇒ **dựng lại mọi `.db`** ⇒ sai `sha256` trong `dict-manifest.toml` ⇒ **đo lại NFR6**. `senses.rs` ghi thẳng *"⛔ Đừng thêm chỉ mục"* | ⛔ **Ngoài phạm vi story này** — nó là một story tầng dữ liệu riêng, cùng hạng với `1-10c` |
| **A** | Bỏ `ORDER BY e.id` khi có `LIMIT` | Có thể nhiều | Nhỏ | ⛔ **Loại.** Thứ tự tất định là mệnh đề đã chốt ở `lookup_grouped` *("thứ tự lớp, rồi mã nguồn — ⛔ không phụ thuộc thứ tự hàng SQLite trả về")*. Bỏ nó là đổi kết quả theo cách ⛔ không tái lập được |

⚠️ **Và một khả năng phải xét TRƯỚC cả bốn:** nếu số đo **đầu-cuối** ở Task 8 cho ra
**< 100 ms** thì **NFR1 đã đạt** và ⛔ **không phương án nào cần chạy** — `LIMIT` vẫn giữ giá
trị của nó *(cắt băng thông IPC và số node DOM)*, và 12,569 ms backend chỉ là một dòng ghi
số. Đó chính là chữ *"nếu cần"* trong lượt chốt của Ice. **Đo trước, rồi mới hỏi có cần
không.**

⚠️ **Và luật SQL ⛔ không nhân nhượng:** `LIMIT` là **tham số ràng buộc** (`LIMIT ?N`), ⛔
**không** nội suy vào văn bản câu. `prepare_cached` khoá theo **văn bản câu**; một số nội
suy là một hình dạng SQL mới mỗi lần ⇒ cache trống trên đúng đường nóng mà nó tồn tại để
bảo vệ *(cùng luật `SENSE_BATCH` của `senses.rs`, luật 2)*. Và nhánh
`NoBranchQueryTooShort` ⛔ **không** chuẩn bị câu SQL nào — `LIMIT` ⛔ không áp cho nó, ⛔
đừng "đồng bộ cho đều".

⚠️ **Ngân sách phải nói cho đúng:** 12,569 ms là **phần backend**; trần 10 ms là một con số
**dẫn xuất** *(PRD dành ~99,95 ms cho vòng IPC + render, giả định `[A1]` — thứ **chưa ai
đo**)*. **Story này là chỗ `[A1]` được đo lần đầu.** Vượt trần backend ⛔ không đồng nghĩa
vượt NFR1; con số nghiệm thu thật của NFR1 chỉ có **sau story này**.

### Quyết định #5 — **Nhiều đầu mục cùng `headword` TRONG một nguồn**

🔴 `deferred-work.md:416` bàn giao đích danh: *"một nhóm nguồn có thể chứa **nhiều đầu mục
cùng `headword`**. Panel Lookup phải trình bày được ca đó *(gộp hiển thị, hoặc đánh số, hoặc
hiện liền nhau)* mà ⛔ **không** đổi dữ liệu."* Số hôm nay: **18** trong `dict-vietphrase.db`
*(46 trong nguồn thô)*.

- **(a) — MẶC ĐỊNH ĐỀ XUẤT: hiện LIỀN NHAU** trong cùng khối nguồn, mỗi đầu mục là một cụm
  nghĩa của chính nó, ⛔ không đánh số, ⛔ không gộp. Lý do: đây là trùng **TRONG** một
  nguồn *(⛔ không phải giữa các nguồn — AD-19 nói về ca kia)*, và một lượt "gộp hiển thị"
  ở đây là **cùng hình dạng tư duy** mà AD-19 cấm, chỉ nhỏ hơn.
- **(b)** Đánh số `打 (1)` · `打 (2)`. ⛔ Cân nhắc nếu (a) đọc ra khó hiểu trên dữ liệu thật —
  **nhưng chỉ sau khi nhìn** 18 hàng đó bằng mắt ở Task 0.

### Quyết định #6 — **Vùng đầu mục chiều cao CỐ ĐỊNH gồm những gì HÔM NAY**

`DESIGN.md §Motion` và AC cuối của epic đều nói: *"Vùng đầu mục Panel Lookup — cao **cố
định**. Đầu mục và thanh nhịp luôn ở cùng toạ độ; chỉ phần dưới thay đổi."* Mockup vẽ **năm**
thứ trong vùng đó; **ba** thuộc story sau.

| Thành phần trong mockup | 1.17? | Ghi chú |
|---|---|---|
| Đầu mục (`打`, token `lookup-headword`) | ✅ | |
| Thanh nhịp: `5 nguồn · 22 nghĩa` + một chip cho mỗi nguồn kèm số nghĩa | ✅ | **dẫn xuất từ `groups`**, ⛔ không danh sách viết cứng (§⛔KHÔNG-LÀM ②) |
| `📌 Ghim ⌘D` | ⛔ | Story **1.20** |
| Dải âm đọc `dǎ · đả — 21 nghĩa` | ⛔ | Đòi `dict_entry.reading` *(⛔ chưa qua dây)* **và** một luật lọc nghĩa theo âm *(⛔ không FR nào)* |
| Chip lọc từ loại `động từ 17` | ⛔ | ⛔ Không FR nào đòi. Nhặt lại khi có |

- **(a) — MẶC ĐỊNH ĐỀ XUẤT:** vùng đầu mục = **đầu mục + thanh nhịp**, chiều cao là một
  **hằng có tên** *(⛔ không một con số rải trong CSS)*, và nó **⛔ không đổi** giữa bốn
  trạng thái rỗng của AC6 — đó chính là mệnh đề *"chỉ phần dưới thay đổi"*.
- 🔴 **⚠️ Quyết định #4 vừa chạm vào đây.** Với `LIMIT` pha một, các con số của thanh nhịp
  ⛔ **không còn miễn phí** — xem **Quyết định #4 §hệ quả ③**. Ngữ nghĩa của thanh nhịp
  *(số thật · `20+` · hay bỏ số)* phải chốt **cùng lượt** với cỡ trang `N`, ⛔ không tách ra.
- ⚠️ **Bẫy:** chiều cao cố định + đầu mục dài *(một cụm 12 ký tự)* ⇒ chữ tràn. `lookup-headword`
  khai `wraps: false` *(24px, giãn dòng 1.3)*, tức bảng token **đã** hứa nó là một dòng.
  ⇒ đầu mục dài phải **cắt bằng `text-overflow`**, ⛔ không xuống dòng, ⛔ không co chữ.

### ✅ Quyết định #7 — ĐÃ CHỐT 2026-08-06 (Ice): **token thứ 17 `ui-md-wrap`**

`deferred-work.md:115` *(Story 1.4)* · `:129` · và mục cuối của §code review 1.16 đều nói
cùng một điều: **⛔ không một token `ui-*` nào khai `wraps: true`**, cả sáu đều ở giãn dòng
1.4–1.5 *(dưới sàn 1.66)*, và `check-tokens.mjs` **Kiểm E** chỉ áp sàn cho token khai
`wraps: true` ⇒ cổng **mù hoàn toàn**.

Story này thêm **ít nhất bốn** câu giao diện đầy đủ *(bốn trạng thái rỗng của AC6)*, tất cả
chắc chắn xuống dòng trong một panel hẹp.

🔴 **CHỐT: (b) — thêm token thứ 17 `ui-md-wrap`**, giãn dòng **1.66**, họ `ui`, qua sổ
`deviations` của `tokens.json` + `check-tokens.mjs`. Đúng tiền lệ `ui-md-strong` *(Story
1.14)* và `source-latin` *(Story 1.16)* — hai token đã đi qua đúng cửa này.

🔴 **Và nó phải áp cho CẢ BA CHỖ DÙNG ĐÃ CÓ, ⛔ không chỉ cho chuỗi mới của story này:**

| Chỗ | Tệp | Hôm nay |
|---|---|---|
| `.load-error` | `src/panels/SourcePanel.vue:199` | `ui-md` (1.5) |
| `.parallel-note` | `src/panels/SourcePanel.vue:208` | `ui-sm` (1.5) |
| `.hv-notice` | `src/panels/SourceHanViet.vue` | `ui-md` (1.5) |

⚠️ **Vì sao ⛔ không được bỏ ba chỗ đó lại:** mục `deferred-work.md:115` nói về một **lỗ hổng
của bảng token**, ⛔ không về một chỗ dùng. Thêm token mà để ba câu cũ tiếp tục chạy ở 1.5 là
**đếm món nợ lần thứ năm dưới một cái tên mới** — và cả ba đều là câu đầy đủ, chắc chắn
xuống dòng trong một panel hẹp, tức đúng ca mà sàn 1.66 tồn tại để giữ *(dấu `ườ` chạm dấu
`ộ`)*.

⚠️ **Đây là ngoại lệ tường minh với §Ranh giới "⛔ KHÔNG ĐỤNG `SourceHanViet.vue`"** — được
đụng **đúng khối `<style>`** của ba lớp trên, ⛔ **không** một dòng `<script>` nào, ⛔ không
refactor, ⛔ không đổi cấu trúc template.

⚠️ **`ui-sm` của `.parallel-note` cũng ⛔ không khai `wraps: true`.** Đổi nó sang `ui-md-wrap`
là **đổi cỡ chữ** (11,5px → 12px) — một thay đổi **nhìn thấy được**. ⇒ Task 7 phải chọn: đổi
sang token mới *(cỡ đổi)*, hay khai thêm một biến thể `ui-sm-wrap`. **Ghi ra lựa chọn**, ⛔
đừng đổi cỡ chữ trong im lặng.

⚠️ **Kiểm E của `check-tokens.mjs` chỉ áp `LINE_HEIGHT_FLOOR = 1.66` cho token khai
`wraps: true`.** ⇒ token mới **phải** khai `wraps: true` — nếu không, nó chỉ là một hàng
JSON nữa mà cổng vẫn mù, và cả lượt chốt này thành vô nghĩa. **Đó là ca đỏ-rồi-xanh bắt buộc
của AC13 cho token 17.**

---

## Acceptance Criteria

### AC1 — Mỗi nguồn là một khối RIÊNG, xếp chồng dọc, ⛔ không bao giờ gộp *(FR32; AD-19)*

**Given** một kết quả tra cứu có ≥ 2 nguồn
**When** hiển thị
**Then** mỗi nguồn là **một khối riêng xếp chồng dọc**, có **vạch trái 2px** và **thụt 13px**
*(⚠️ **cấp nào mang vạch** — khối nguồn hay từng nghĩa — là **mâu thuẫn tài liệu #4**; chốt
ở Task 0 và **ghi ra**, ⛔ không đoán)*
**And** thứ tự khối là thứ tự `GroupedLookup::groups` **nguyên vẹn** *(tất định: thứ tự lớp,
rồi `source.code` tăng dần — đã chốt ở `lookup_grouped`)*
**And** ⛔ **không** một hàm nào ở `src/**` gộp, xếp hạng, khử trùng, hay chọn một nguồn làm
"câu trả lời"
**And** hai nghĩa **giống hệt nhau về chữ** ở hai nguồn khác nhau vẫn hiện **hai lần**

### AC2 — Nhãn nguồn `ui-label` màu `primary`, dùng tên hiển thị THẬT *(FR31)*

**Given** một khối nguồn
**When** hiển thị
**Then** nhãn nguồn dùng token **`ui-label`** màu **`primary`**
**And** nội dung nhãn là **`SourceInfo::display_name`** — đọc từ `dict_source` của **chính
tệp `.db`** chứa đầu mục đó, ⛔ không từ một bảng tra `code → tên` dựng ở webview
**And** ⛔ **không tồn tại** một đường nào — prop, cờ, chế độ — làm nhãn nguồn biến mất

### AC3 — SÁU phần của FR28 đều render, và trích dẫn KHÁC ví dụ *(FR28, FR29, FR30)*

**Given** một nghĩa có đủ dữ liệu
**When** hiển thị
**Then** hiện đủ sáu phần: **nguồn · từ loại · nghĩa · ví dụ[] · trích dẫn[] · ghi chú**
**And** **mỗi `SenseRecord` là một mục riêng** *(FR29 — một từ nhiều từ loại ⇒ nhiều mục,
⛔ không nối `gloss` thành một chuỗi)*
**And** ví dụ và trích dẫn treo **theo từng nghĩa** *(`sense_id`)*, ⛔ không theo đầu mục
**And** **trích dẫn có vạch trái `primary`** để phân biệt với ví dụ *(⚠️ vạch của trích dẫn
là `primary`; vạch của **khối nguồn** ở AC1 ⛔ không phải `primary` — xem mockup `.sense`
dùng `--line-2`. Hai vạch, hai màu, ⛔ không nhầm)*
**And** từ loại: họ `read` **in nghiêng** màu `on-surface-variant`; nghĩa: token
**`lookup-gloss`**; ví dụ và trích dẫn: token **`lookup-example`**
**And** ⛔ trường vắng mặt (`pos = None`, `note = None`, `examples = []`) ⇒ **⛔ không render
một hàng rỗng nào**, ⛔ không dấu gạch giữ chỗ

### AC4 — Nhãn ngoại ngữ ĐÁNH DẤU RÕ, đọc TRƯỜNG chứ ⛔ không đoán *(FR35)*

**Given** một nghĩa có `pos_lang = Some("en")`
**When** hiển thị
**Then** nhãn từ loại được **đánh dấu rõ là nhãn ngoại ngữ** *(FR35 nguyên văn)*
**And** dấu hiệu đó đọc từ **trường `pos_lang`**, ⛔ **không** từ một bảng tra `"noun" ⇒
tiếng Anh` — một bảng như thế **sai im lặng** với mọi nhãn nó chưa gặp *(doc-comment
`SenseRecord::pos_lang` cấm đích danh)*
**And** cùng luật áp cho **`ExampleRecord::translation_lang`**
**And** `pos_lang = None` ⇒ ⛔ **không** dấu hiệu nào *(⛔ không mặc định "tiếng Việt")*

### AC5 — Dòng dẫn bất đồng đứng TRƯỚC khi liệt kê *(FR32)*

**Given** hai nguồn ghi **khác nhau** về cùng một mục từ
**When** hiển thị
**Then** một **dòng dẫn** nói rõ điều đó **TRƯỚC** khi liệt kê các khối nguồn
**And** dòng dẫn ⛔ **không** phán xét nguồn nào đúng, ⛔ không tóm tắt, ⛔ không đứng thay
cho các khối
**And** vị từ *"bất đồng"* là một **hàm thuần, xuất được, test được** — ⛔ không một biểu
thức chôn trong `<template>`
**And** ⛔ **chỉ một** nguồn có kết quả ⇒ ⛔ **không** dòng dẫn *(một nguồn ⛔ không bất đồng
với ai)*

### AC6 — BỐN trạng thái rỗng, và cả bốn PHÂN BIỆT ĐƯỢC

**Given** người dùng **chưa tra gì**
**Then** hiện chuỗi **dạy thao tác bôi đen** — chuỗi này **KHÁC** chuỗi không tìm thấy

**Given** một lượt tra đã chạy mà ⛔ không tìm thấy gì *(`groups` rỗng, `branch ≠ query_too_short`)*
**Then** hiện chuỗi **không tìm thấy** kèm gợi ý **tra từng chữ trong cụm vừa chọn**
**And** ⛔ **không trỏ tới bất kỳ năng lực nào chưa tồn tại** — ⛔ **không** nhắc chữ
*Concordance* *(đường sang nó là Story 7.7)*

**Given** `branch == query_too_short` *(chuỗi con tiếng Anh < 3 ký tự — AD-44 ④)*
**Then** hiện chuỗi **"truy vấn quá ngắn"**, ⛔ **không** phải "không tìm thấy"
**And** hai câu đó dẫn người dùng đi hai đường: một bảo *gõ thêm*, một bảo *từ này ⛔ không
có trong từ điển*

**Given** `skipped` khác rỗng *(một lớp ⛔ không nạp được, hoặc lượt tra trên nó hỏng)*
**Then** nói ra rằng **một phần từ điển ⛔ không trả lời**, kể cả khi có kết quả từ lớp khác
**And** ⛔ **không** hiện `path` hay `detail` của lỗi *(AD-21 — xem Quyết định #2)*

⚠️ **Và ca thứ năm ⛔ không phải một trạng thái rỗng:** `layers` rỗng *(⛔ không lớp nào
gắn — `src-tauri/resources/dict/` **rỗng trong git**, AD-25, và đó là **mặc định của mọi bản
dựng hôm nay**)*. Nó phải nói ra bằng chuỗi **riêng**, cùng luật `layers_loaded` mà Story
1.16 đã áp — ⛔ đừng để nó trông như "không tìm thấy".

### AC7 — ⛔ KHÔNG spinner, ⛔ không "đang tải"; vùng đầu mục chiều cao CỐ ĐỊNH

**Given** một lần tra đang chạy
**Then** ⛔ **không** spinner, ⛔ **không** trạng thái *"đang tải"* — `EXPERIENCE.md:117`:
*"spinner ở đây là **tiếng ồn**"*
**Given** vùng đầu mục
**When** kết quả đổi *(kể cả đổi giữa bốn trạng thái của AC6)*
**Then** chiều cao **⛔ không đổi một pixel** — đầu mục và thanh nhịp giữ nguyên toạ độ, chỉ
phần dưới thay đổi
**And** chiều cao đó là một **hằng có tên**, ⛔ không một con số rải trong CSS
**And** đầu mục dài **cắt bằng `text-overflow`**, ⛔ không xuống dòng *(`lookup-headword`
khai `wraps: false`)*

### AC8 — Đường IPC hai pha, và ⛔ KHÔNG một chuỗi hiển thị nào từ Rust *(AD-1, AD-21)*

**Given** webview cần một lượt tra
**Then** nó gọi **một** `#[tauri::command]` mới, vỏ mỏng, lấy `DictLayers` bằng `try_state`
*(⛔ không `state()` — cùng lý do `commands::chapter::wire`)*
**And** **hàm thuần** nhận `Option<&DictLayers>` và **đây là thứ test gọi** *(khuôn
`commands::dict::read_han_viet`, `commands::chapter::read_open_chapter`)*
**And** hai pha giữ nguyên: pha một `lookup_grouped`, pha hai `senses` **chỉ cho tập đầu mục
mà webview thật sự hiện** *(§HAI PHA của `core/dict/mod.rs`)*
**And** `entry_id` của pha hai đi **qua đúng lớp của nhóm** *(`DictLayers::layer(&group.layer)`)*
— ⛔ **không** trộn id giữa các tệp *(id chỉ duy nhất **trong một tệp `.db`**; trộn là đọc
nhầm nghĩa mà ⛔ **không lỗi nào được ném**)*
**And** ⛔ **không một chuỗi tiếng Việt có dấu nào** ở vị trí mã trong `src-tauri/**/*.rs`
*(Kiểm A của `check-i18n.mjs`)*; mọi câu hiển thị sống ở `vi.json`
**And** ⛔ **không** cổng thứ tư (AD-2), ⛔ không permission mới trong `capabilities/`

### AC9 — Bề mặt Lookup khai token CỦA CHÍNH NÓ

**Given** thân Panel Lookup
**Then** mọi bề mặt chữ khai token của chính nó: **`lookup-headword`** · **`lookup-gloss`** ·
**`lookup-example`** · **`ui-label`**
**And** ⛔ **không** bề mặt nội dung nào để kế thừa mặc định từ `body` *(`ui-md`, giãn dòng
**1.5** — dưới sàn 1.66; Kiểm E của `check-tokens.mjs` chỉ đọc `tokens.json` nên **hoàn toàn
mù** với việc component nào kế thừa gì — `deferred-work.md:129`)*
**And** `lookup-example` tiêu thụ **`font-synthesis: var(--synthesis-lookup-example)`** — nó
là **token thứ hai** của lời giải chữ Hán nghiêng giả *(`source-hanviet` đã đóng ở 1.16;
`deferred-work.md:133` gọi tên **cả hai**)*. ⚠️ Với từ điển Trung–Việt, ví dụ **chắc chắn**
có chữ Hán, ở 12,5px — cỡ mà nghiêng giả xấu nhất
**And** 🔴 **token thứ 17 `ui-md-wrap`** *(1.66, `wraps: true`)* có mặt trong `tokens.json` +
sổ `deviations`, **và** cả **ba** chỗ dùng cũ đã chuyển sang nó *(`.load-error` ·
`.parallel-note` · `.hv-notice`)* — chỉ thêm token mà để ba câu cũ ở 1.5 là **⛔ không đóng**
`deferred-work.md:115` *(Quyết định #7, Ice chốt)*
**And** ⛔ **không** một giá trị màu viết thẳng nào *(Kiểm B/B2)*; ⛔ không `opacity` để làm
mờ chữ *(Kiểm D, UX-DR6)*; ⛔ không `box-shadow` *(Kiểm F)*

### AC10 — State sống ngoài component, và ĐƯỢC reset khi Tác phẩm đổi

**Given** người dùng đổi preset bố cục *(`applyPreset()` → `api.clear()` → dựng lại **cả
bốn** panel)*
**Then** kết quả tra cứu đang hiện **⛔ không mất**, và ⛔ **không** một lượt IPC nào chạy lại
**And** state sống ở một module riêng *(`src/panels/lookupPanelState.ts`)*, ⛔ không trong
`ref` cục bộ của component — khuôn `sourcePanelState.ts`
**Given** Tác phẩm đang mở **bị thay** *(`modes/libraryImport.ts::finishSubmit`)*
**Then** state Lookup **bị vứt** cùng lượt với state Source
**And** ⛔ đây ⛔ **không** phải một lời gọi thứ hai rải ra — nó đi qua **đúng điểm nghẽn** mà
`resetSourcePanel()` đã đi qua *(lỗi này đã xảy ra thật một lần ở 1.16 và tốn một lượt code
review để bắt)*

### AC11 — 🔴 Đo NFR1 ĐẦU-CUỐI, và ghi SỐ chứ ⛔ không ghi lời hứa

**Given** đường tra cứu đã nối xong
**When** đo
**Then** ghi vào §Debug Log References một **bảng số** gồm ba cột: **pha một** *(Rust)* ·
**pha hai** *(Rust)* · **đầu-cuối** *(webview phát lệnh → bản ghi hiện trên màn hình)*
**And** đo trên **ít nhất ba hình dạng truy vấn**: một ký tự Hán *(nhánh `char_idx` — ca đắt
nhất, 3.177 đầu mục)* · một cụm 2–4 ký tự Hán · một từ tiếng Anh
**And** 🔴 **đo TRƯỚC và SAU khi thêm `LIMIT`** — `LIMIT` chỉ được coi là có tác dụng khi số
đo nói vậy *(§Quyết định #4 hệ quả ①: `ORDER BY e.id` + `JOIN` có thể buộc sắp toàn bộ, và
lúc đó `LIMIT` cắt băng thông chứ ⛔ không cắt thời gian)*
**And** `EXPLAIN QUERY PLAN` của nhánh `char_idx` ghi vào §Debug Log References **nguyên văn**
**And** 🔴 nếu số đo nói `LIMIT` ⛔ **không** cắt được thời gian ⇒ **DỪNG THI HÀNH** *(Ice chốt
2026-08-06)*, khảo sát **bốn phương án ứng viên** ở §Quyết định #4, **ghi số**, đề xuất, và
**hỏi Ice trước khi cài** — ⛔ **không** tự chọn, ⛔ **không** im lặng đi tiếp với một thay
đổi đắt tiền mà ⛔ không mua được gì
**And** ⚠️ xét điều kiện *"nếu cần"* **trước**: đầu-cuối **< 100 ms** ⇒ **NFR1 đã đạt** ⇒ ⛔
không phương án nào phải chạy, và 12,569 ms backend chỉ là một dòng ghi số
**And** cỡ trang `N`, ngữ nghĩa thanh nhịp *(§hệ quả ③)*, và đường xử lý §hệ quả ② đều ghi ra
kèm **con số làm cơ sở**
**And** ⚠️ nếu ⛔ không có tệp `.db` thật trong phiên *(`.gitignore: *.db` — chúng ⛔ không
có trong git)*, **nói thẳng điều đó** và ghi phép đo trên fixture kèm cảnh báo *"⛔ không
thay được số trên dữ liệu thật"* — ⛔ **đừng** đánh dấu đạt rồi im *(tiền lệ 1.16)*

### AC12 — 🔴 `LIMIT` ⛔ KHÔNG được làm một nguồn biến mất im lặng *(FR31; Quyết định #4 ②)*

**Given** một tệp `.db` mang **nhiều nguồn** *(`dict-core.db` mang **sáu**)*
**And** một truy vấn khớp nhiều đầu mục hơn cỡ trang `N`
**When** pha một chạy với `LIMIT`
**Then** **mọi nguồn có ít nhất một đầu mục khớp đều có mặt trong `groups`**
**Or** *(nếu chọn đường (b) của §hệ quả ②)* tệp mang một cờ `truncated` và panel nói ra rằng
**danh sách nguồn ⛔ chưa đầy đủ** — ⛔ không im
**And** 🔴 **ca test bắt buộc, và nó phải ĐỎ trên một cài đặt `LIMIT` cấp-tệp ngây thơ:** một
fixture **một tệp, hai nguồn**, nguồn thứ hai có **mọi** `dict_entry.id` **lớn hơn** mọi id
của nguồn thứ nhất, và số đầu mục của nguồn thứ nhất **≥ `N`**. Kỳ vọng: nguồn thứ hai
**vẫn có mặt**
**And** ⚠️ ca test đó ⛔ **không** dựng được bằng fixture ba-lớp-mỗi-lớp-một-nguồn đang có —
điều kiện tiên quyết của lỗi là **nhiều nguồn TRONG một tệp**. Dựng thiếu là một **test giả**
*(đúng lớp lỗi đã bị bắt ở code review 1.16)*
**And** thanh nhịp ⛔ **không bao giờ** khẳng định một con số nó ⛔ không biết — nếu số đếm là
cận dưới thì nó phải **đọc ra như một cận dưới** *(§hệ quả ③)*

### AC13 — Mọi cổng xanh, sàn nâng theo số THẬT, ranh giới ⛔ KHÔNG CHẠM giữ nguyên

**Then** cả **chín** lệnh DoD xanh *(xem §Testing standards)*
**And** mọi hằng `*_FLOOR` bị vượt được **nâng theo số thật** — ⛔ không để một sàn *"⛔ không
còn canh được gì"* *(tiền lệ 1.14 · AC11.1; và 1.16 đã **quên** đúng việc này rồi bị bắt ở
code review)*. Số hôm nay: `VUE_FLOOR` **10** vs 12 · `TS_FLOOR` **19** vs 23 · `COMMAND_FLOOR`
**13** vs 16 · `CLICK/DISPATCH_FLOOR` **6** vs 8 · `RS_FLOOR` **32** vs 39 · `FILE_FLOOR`
*(layout)* **28** vs 50 · `FILE_FLOOR` *(tokens)* **26**
**And** **đỏ-rồi-xanh** cho mọi cổng bị đụng: mỗi mệnh đề mới có **ít nhất một ca làm cổng
ĐỎ** cộng **một đối chứng âm**; con số ghi vào Completion Notes
**And** ⛔ `matchMedia` = **0**, `window.innerWidth` = **0** trong `src/**` *(giữ nguyên)*
**And** ⛔ 0 phụ thuộc mới, npm lẫn crate *(NFR15, `check-deps.mjs`)*

---

## Tasks / Subtasks

- [x] **Task 0 — Chốt bảy quyết định, và ĐO trước khi chốt #4** *(AC: tất cả)*
  - [x] Đọc trọn: `src-tauri/src/core/dict/mod.rs` · `ports/dict_source.rs` · `commands/dict.rs`
        · `commands/chapter.rs` · `src/panels/sourcePanelState.ts` · `src/panels/SourcePanel.vue`
        · `mockups/lookup-real-density.html`
  - [x] Xác nhận baseline: `git status` sạch, `cargo test` xanh, chín cổng DoD xanh **trước**
        khi gõ dòng đầu tiên *(một cổng đã đỏ từ trước là thứ phải biết ngay, ⛔ không phải
        sau bốn giờ)*
  - [x] 🔴 **Đo lại `:419`** — `bench_the_grouped_path_on_the_real_dictionaries` trên baseline
        hôm nay. Nếu ⛔ không có tệp `.db` thật ⇒ **ghi ra**, ⛔ không đoán
  - [x] 🔴 **`EXPLAIN QUERY PLAN` cho nhánh `char_idx`** *(và cho `fts_trigram`)* — **trước**
        khi viết một dòng `LIMIT` nào. Đây là thứ trả lời §Quyết định #4 hệ quả ①
  - [x] 🔴 Chốt đường xử lý **§hệ quả ②** *(nguồn biến mất)* và **§hệ quả ③** *(số đếm thanh
        nhịp)* — hai thứ này chốt **cùng lượt**, ⛔ không tách
  - [x] Nhìn **bằng mắt** 18 hàng trùng `headword` của `dict-vietphrase.db` *(Quyết định #5)*
  - [x] Chốt #1 · #2 · #3 · #5 · #6 *(#4 và #7 **Ice đã chốt** — đọc lại, ⛔ không mở lại)*
  - [x] Cỡ trang `N` của #4 chốt sau Task 8 *(nó cần số đo trước/sau)*
  - [x] Chốt **mâu thuẫn tài liệu #4** *(vạch trái treo ở cấp NGUỒN hay cấp NGHĨA)* — ghi ra
        kèm lý do; ⛔ không sửa `epics.md`/`DESIGN.md`/mockup
  - [x] Ghi cả bảy vào Change Log kèm lý do — ⛔ không mục nào "theo mặc định" mà ⛔ không nói
        đã đối chiếu gì

- [x] **Task 1b — 🔴 `LIMIT` pha một** *(AC: 11, 12; Quyết định #4 — ✅ Ice đã chốt)*
  - [x] Đổi chữ ký `DictionarySource::lookup` — thêm trần, **⛔ không** thêm method thứ sáu
  - [x] Sáu hình dạng SQL trong `query.rs` — `LIMIT ?N` **tham số ràng buộc**, ⛔ không nội suy
        *(ba nhánh không-verify dùng `LIMIT` ở SQL; ba nhánh cần `verify_substring` cắt ở
        Rust SAU verify — xem Bẫy 11 dưới)*
  - [x] `lookup` · `lookup_with_branch` · `lookup_grouped` truyền **cùng một** trần xuống
        **mọi** tệp *(cùng doctrine `route`/`branch`: một GIÁ TRỊ của cả lượt tra)*
  - [x] ⛔ **Không** áp cho `NoBranchQueryTooShort` — nhánh đó ⛔ không chuẩn bị câu SQL nào
  - [x] 🔴 **Trần áp SAU `verify_substring`** *(nhánh 2 ký tự)* và sau bước lọc của
        `fts_trigram` — cắt ứng viên trước khi xác minh cho ra **< `N`** mục và một dòng
        *"còn M nữa"* **sai** (Bẫy 11). Ca test phải dựng đúng tình huống `verify` loại bớt
        *(`the_limit_is_applied_after_verification_not_before` — đỏ-rồi-xanh đã kiểm chứng)*
  - [x] 🔴 Cài đường xử lý **§hệ quả ②** đã chốt ở Task 0, cộng **ca test AC12**
        *(`a_file_level_limit_flags_truncation_instead_of_silently_dropping_a_source` +
        biến thể `char_idx` — cả hai đỏ-rồi-xanh đã kiểm chứng)*
  - [x] Cập nhật **mọi** test hành vi đang gọi bốn hàm trên *(`dict_lookup.rs`,
        `dict_sources.rs`, `dict_boundary.rs`)* — ⚠️ đây là phần **đắt nhất** của story
        *(`dict_boundary.rs` ⛔ không cần đổi — nó đếm mã nguồn tĩnh, ⛔ không gọi các hàm này)*

- [x] **Task 1 — Hình dạng bản ghi trên dây** *(AC: 8; Quyết định #2)*
  - [x] Derive `Serialize` theo #2(a) — ⛔ không `rename_all` *(`SourceInfo`/`EntryHit`/
        `SenseRecord`/`ExampleRecord`/`CitationRecord`/`SourceGroup`/`GroupedLookup`)*
  - [x] `QueryBranch`/`QueryRoute` ra dây bằng **chuỗi định danh máy**
        *(`#[serde(rename = …)]` từng biến thể, ⛔ không `rename_all` — `"query_too_short"`
        không phải chuyển đổi cơ học từ `NoBranchQueryTooShort`)*
  - [x] 🔴 `skipped` **rút gọn**: số lượng + mã máy. ⛔ **Không** `path`, ⛔ **không** `detail`
        *(`SkipReason::wire_code()` + `#[serde(serialize_with = "serialize_skipped_as_wire_codes")]`
        trên `GroupedLookup::skipped` — kiểu Rust của trường không đổi, chỉ hình dạng dây đổi;
        `SkippedLayer`/`SkipReason` vẫn KHÔNG derive `Serialize`)*
  - [x] Ca test hành vi: một `SkipReason::OpenFailed { detail }` ⇒ chuỗi `detail` **⛔ không
        xuất hiện** trong JSON đi ra *(đây là ca làm cổng AD-21 ĐỎ nếu ai đó derive thẳng)*
        *(`skip_reason_detail_never_reaches_the_wire` ở `dict_sources.rs` — giá trị serialize
        đến từ `lookup_grouped` thật trên một tệp `garbage.db` hỏng thật, ⛔ không dựng tay)*

- [x] **Task 2 — `#[tauri::command]` tra cứu: hàm thuần + vỏ mỏng** *(AC: 8)*
  - [x] Hàm thuần trong `commands/dict.rs`, nhận `Option<&DictLayers>` *(khuôn `read_han_viet`)*
  - [x] Vỏ `wire` dùng `try_state` — ⛔ không `state()`, ⛔ không `.unwrap()`
  - [x] Đăng ký vào `generate_handler![…]` ở `lib.rs` *(hôm nay **6** command, sau story là 7+)*
        *(→ 7: `lookup_dictionary`)*
  - [x] ⛔ **Không nhánh lỗi cho ca "0 lớp"** — nó là trạng thái BÌNH THƯỜNG có tên (AD-25),
        cùng luật `read_han_viet`
  - [x] Test hành vi ở `src-tauri/tests/dict_*.rs` — **dùng lại fixture ba lớp của
        `dict_sources.rs`**, ⛔ đừng dựng bộ thứ hai

- [x] **Task 3 — Pha hai đi qua ĐÚNG lớp** *(AC: 8)*
  - [x] `DictLayers::layer(&group.layer)` → `senses(&entry_ids)`; `entry_ids` **của chính lớp đó**
  - [x] 🔴 Ca test **hai lớp**, cùng `entry_id` số học, nghĩa khác nhau ⇒ chứng minh ⛔ không
        trộn. *(Ca một lớp ⛔ không dựng được điều kiện tiên quyết của lỗi — đúng lớp "test
        giả" đã bị bắt ở 1.16)* *(`phase_two_never_mixes_entry_ids_across_two_layers_sharing_the_same_number`
        — dùng chính fixture `LAYERS` sẵn có: `base` và `hv-fixture` cùng `entry_id=1` cho `山`)*
  - [x] `senses(&[])` ⇒ ⛔ không chạm database *(cùng luật `han_viet(&[])`)* *(chứng minh gián
        tiếp qua `lookup_command_calls_senses_with_an_empty_batch_for_no_layer` — ⛔ nhóm nào
        khớp ⇒ `senses_by_layer` rỗng, ⛔ một khoá lớp nào có mặt)*

- [x] **Task 4 — `lookupPanelState.ts` + đường kích hoạt** *(AC: 6, 10; Quyết định #1)*
  - [x] Module-level state, khuôn `sourcePanelState.ts`: query hiện tại · kết quả · lỗi ·
        `pending` · `resolved`
  - [x] 🔴 **Bốn trạng thái rỗng là bốn vị từ RIÊNG**, ⛔ không một chuỗi `if/else` trong
        `<template>`. Cộng vị từ thứ năm cho ca "0 lớp" *(`neverLookedUp`/`notFound`/
        `queryTooShort`/`someLayerFailed` + `layersLoaded`; cộng `someLayerTruncated` cho
        AC12/hệ quả ②)*
  - [x] `runLookup(query)` — **một** đường vào duy nhất
  - [x] Command `lookup.lookup_selection` + `deps.currentSelection` tiêm ở `main.ts`
        *(`Mod+Alt+L`; `currentSelection` dùng `window.getSelection()` — thêm vào
        `ALLOWED_GLOBAL_MEMBERS` của `check-layout.mjs` Kiểm C kèm lý do AC)*
  - [x] `resetLookupPanel()` gọi từ `libraryImport.ts::finishSubmit` — **cùng điểm nghẽn**
  - [x] ⚠️ Tệp này ⛔ **không** được `import` vào `src/commands/index.ts` *(nó dùng `ref` của
        Vue; Kiểm C/D/E nạp `index.ts` bằng **Node thuần**)* — đi qua `CommandDeps`

- [x] **Task 5 — Vùng đầu mục chiều cao cố định + thanh nhịp** *(AC: 7; Quyết định #6)*
  - [x] Hằng có tên cho chiều cao; ⛔ không con số rải trong CSS *(`--lookup-head-height: 76px`
        + `overflow: hidden` trên `.lookup-head` — điều kiện CƠ HỌC "⛔ đổi một pixel" kể cả
        khi thanh nhịp tràn quá một dòng)*
  - [x] Thanh nhịp **dẫn xuất từ `groups`** — `N nguồn · M nghĩa` + một chip mỗi nguồn
        *(`computeSpine()` hàm thuần ở `lookupPanelState.ts`)*
  - [x] `text-overflow: ellipsis` cho đầu mục dài
  - [x] ⛔ Không ghim, ⛔ không tab Concordance, ⛔ không chip bật/tắt, ⛔ không lọc từ loại

- [x] **Task 6 — Khối nguồn và bản ghi** *(AC: 1, 2, 3, 4, 5)*
  - [x] Một khối cho mỗi `SourceGroup`, vạch trái 2px + thụt 13px, xếp chồng dọc *(ở CẤP
        NGHĨA theo mâu thuẫn tài liệu #4 chốt Task 0 — mockup thắng)*
  - [x] Nhãn nguồn `ui-label`/`primary` từ `display_name`
  - [x] Mỗi `SenseRecord` một mục; ví dụ nghiêng; **trích dẫn vạch trái `primary`**; ghi chú
  - [x] Nhãn ngoại ngữ đọc `pos_lang`/`translation_lang` — ⛔ không bảng tra
  - [x] Vị từ bất đồng là **hàm thuần export được** + dòng dẫn đứng **trước** danh sách
        *(`sourcesDisagree()` ở `lookupPanelState.ts`)*
  - [x] Nhiều đầu mục cùng `headword` trong một nguồn — theo Quyết định #5 *(danh sách
        `senses` truyền vào `LookupRecord.vue` đã PHẲNG theo entry_id → ord → sense_id,
        component ⛔ cần biết ranh giới đầu mục)*
  - [x] ⛔ **Không `v-html`** ở bất kỳ đâu *(AD-16 — nội dung từ điển là nội dung ngoài)*

- [x] **Task 7 — Token, token thứ 17, và cổng thị giác** *(AC: 9; Quyết định #7 — ✅ đã chốt)*
  - [x] Ba token `lookup-*` lần đầu có người tiêu thụ + `font-synthesis` cho `lookup-example`
  - [x] 🔴 **Token thứ 17 `ui-md-wrap`** — họ `ui`, giãn dòng **1.66**, **`wraps: true`**
        *(thiếu cờ này là cổng vẫn mù ⇒ cả lượt chốt thành vô nghĩa)*; vào `tokens.json`
        **và** sổ `deviations` của `check-tokens.mjs`, đúng khuôn `ui-md-strong`/`source-latin`
        *(`EXPECTED_COUNTS.typography` 16→17)*
  - [x] 🔴 **Áp cho cả ba chỗ dùng đã có** — `.load-error` · `.parallel-note`
        *(`SourcePanel.vue`)* · `.hv-notice` *(`SourceHanViet.vue`)*. ⚠️ **Chỉ khối `<style>`**,
        ⛔ không một dòng `<script>` nào
  - [x] ⚠️ `.parallel-note` đang ở `ui-sm` (11,5px) — chọn: đổi sang `ui-md-wrap` *(**cỡ chữ
        đổi**, nhìn thấy được)* hay khai thêm `ui-sm-wrap`. **Ghi ra lựa chọn** *(CHỐT: đổi
        sang `ui-md-wrap` — 11,5px→12px, dùng CHUNG một token cho cả ba chỗ, ⛔ thêm
        `ui-sm-wrap` riêng — xem `tokens.json` deviations)*
  - [x] Đối chiếu **từng bề mặt chữ** với token nó khai *(lượt rà mà `deferred-work.md:113`
        và `:129` gọi đích danh story này — mọi class CSS của `LookupPanel.vue`/
        `LookupRecord.vue` đều khai `font-family`/`font-size`/`line-height` qua `var(--...)`,
        ⛔ không một giá trị viết thẳng nào)*

- [x] **Task 8 — 🔴 ĐO NFR1 đầu-cuối, TRƯỚC và SAU `LIMIT`** *(AC: 11, 12)*
  - [x] Bảng ba cột × ba hình dạng truy vấn, **hai lượt**: trước `LIMIT` và sau `LIMIT`
        *(mở rộng `bench_the_grouped_path_on_the_real_dictionaries` — khối "SAU LIMIT" mới,
        qua đúng `commands::dict::lookup()`)*
  - [x] `EXPLAIN QUERY PLAN` ghi nguyên văn vào §Debug Log References *(đã ghi ở Task 0)*
  - [x] Chốt **cỡ trang `N`** từ số đo — **N = 20**, giữ nguyên giá trị đã dùng xuyên suốt
        (đo xác nhận: char_idx p95 20,836ms → 5,109ms; đường sản phẩm thật p95 6,535ms)
  - [x] ⚠️ **Xét điều kiện *"nếu cần"* TRƯỚC:** đầu-cuối < 100 ms ⇒ NFR1 đạt ⇒ ⛔ không
        phương án nào phải chạy *(ước tính đầu-cuối < 40ms — NFR1 ĐẠT, xem giới hạn phép đo)*
  - [x] 🔴 **Nếu `LIMIT` ⛔ không cắt được thời gian ⇒ DỪNG THI HÀNH** *(Ice chốt)* — **⛔ áp
        dụng**: `LIMIT` ĐÃ đo được là cắt thời gian thật (Task 0), nên ⛔ cần khảo sát bốn
        phương án / hỏi Ice. 🔴 Phát hiện thêm ở Task 8: nhánh `char_idx` (nhánh mà cả bốn
        phương án nhắm tới) **không** được đường sản phẩm thật của 1.17 chạm tới —
        `commands::dict::lookup()` cố định `Exact`, và `Exact` luôn đi `ExactBtree`. Đã ghi
        thẳng vào §Debug Log References, ⛔ giấu.
  - [x] Ghi giới hạn của phép đo *(engine nào, có tệp `.db` thật hay không)* — ⛔ không giấu
        *(backend: dữ liệu thật + hàm sản phẩm thật; webview: Chromium thật nhưng IPC giả
        lập tức thời, ⛔ có bản Tauri đã đóng gói để đo vòng IPC thật — ghi rõ trong §Debug
        Log References)*

- [x] **Task 9 — `vi.json` + chuỗi** *(AC: 6; NFR16)*
  - [x] Bốn+ chuỗi trạng thái rỗng, mỗi trạng thái **một chuỗi riêng** — ⛔ không dùng lại
        *(sáu khoá mới `panel.lookup.*` + tái dùng `panel.lookup.status` sẵn có cho ca
        "chưa tra gì" — đúng nghĩa gốc của khoá đó, ⛔ dùng lại chồng nghĩa)*
  - [x] ⛔ Chuỗi *không tìm thấy* ⛔ **không** nhắc *Concordance* *(đã đối chiếu nguyên văn)*
  - [x] Khoá chấm có tiền tố miền `panel.lookup.*` *(Kiểm B của `check-i18n.mjs`)*
  - [x] Giọng văn UX-DR47 *(Kiểm D)* — người dùng là dịch giả chuyên nghiệp: nêu **vì sao
        rỗng** và **làm gì tiếp**, ⛔ không xin lỗi, ⛔ không dấu chấm than *(Kiểm D xanh —
        70 chuỗi, không "chúng tôi"/"bạn")*

- [x] **Task 10 — Cổng và sàn** *(AC: 13)*
  - [x] Chín lệnh DoD xanh
  - [x] Nâng **mọi** `*_FLOOR` bị vượt theo số thật — ⛔ không bỏ qua như 1.16 đã bỏ qua
        *(`check-commands.mjs`: VUE 10→11, TS 19→20, COMMAND 13→14, DISPATCH 6→10;
        `check-i18n.mjs`: VUE 10→11; `check-layout.mjs`: FILE 28→30; `check-tokens.mjs`:
        FILE 26→32, COMPONENT 23→30 — sàn 1.14 đã tụt xuống ~62-65% số thật sau ba story)*
  - [x] Đỏ-rồi-xanh + đối chứng âm cho mỗi mệnh đề mới; ghi số *(kiểm chứng trực tiếp:
        `COMMAND_FLOOR=18` giả lập → cổng ĐỎ đúng thông báo "17 command (sàn 18)"; khôi phục
        `14` → xanh lại. Cộng đỏ-rồi-xanh đã làm ở Task 1b/1 cho `truncated_layers`/Bẫy 11/
        wire-shape — xem §Debug Log References)*

- [x] **Task 11 — Bàn giao trung thực**
  - [x] Cập nhật `src/panels/README.md` hàng 1.17 → ✅ *(cộng đoạn "Chữ trong thân panel" mới
        cho Panel Lookup)*
  - [x] `deferred-work.md`: đóng những mục story này đóng thật; **ghi mới** những gì ⛔ chưa đo
        *(mười một mục đóng: `:453` `:129/:131` `:133` `:115` `:343` `:363` `:416` `:419/:449`
        `:504` — cộng mục `:317` (mục từ tiếng Anh) ghi rõ ⛔ đóng; mục mới "Deferred from:
        1-17" ghi năm phát hiện: `query_too_short` không thể thực thi qua đường sản phẩm
        hôm nay, vòng IPC thật chưa đo, `.parallel-note` đổi cỡ chữ, mục từ tiếng Anh nhắc
        lại, `senses()` trượt không có tín hiệu riêng)*
  - [x] ⛔ **Không** đánh dấu đạt cho bất cứ thứ gì ⛔ chưa chạy — vế thị giác hai nền tảng
        thật (WKWebView/WebView2) là **món nợ cũ**, ⛔ không phải món nợ story này đóng
        *(ghi rõ trong deferred-work.md: story đo bằng Rust thật + Playwright/Chromium thật,
        ⛔ WKWebView/WebView2/`tauri dev` thật, ⛔ tự nhận đã đóng món nợ đó)*

### Review Findings

*Code review 2026-08-06 — ba lớp (Blind Hunter · Edge Case Hunter · Acceptance Auditor), baseline `cb03974`. Cả 9 cổng DoD xanh và `cargo test` 217/217 xanh tại thời điểm review, nên mọi mục dưới đây là khuyết tật ngữ nghĩa mà ⛔ không cổng nào bắt được.*

*Sáu mục `[Decision]` đã được Ice chốt 2026-08-07 — hướng xử lý ghi ngay trong từng mục dưới dạng ✅ **CHỐT**.*

- [x] [Review][Patch] **Thanh nhịp khẳng định số tuyệt đối trong khi `LIMIT` có thể đã cắt — truy vấn `COUNT` đã CHỐT ở Task 0 ⛔ không tồn tại trong mã** — `grep COUNT src-tauri/src/core/dict/` cho **0 kết quả**; Quyết định #4 §hệ quả ③ chốt đường (a) *("một truy vấn `COUNT(*) … GROUP BY source_id` riêng, chạy CHỈ KHI `truncated = true`")* nhưng ⛔ không được cài và ⛔ không được ghi là đã đổi ở Completion Notes. `computeSpine()` đếm thẳng trên `groups`/`sensesByLayer` đã bị `LIMIT 20` cắt, `LookupPanel.vue:62` in ra như số tuyệt đối. Vi phạm AC12 mệnh đề cuối: *"thanh nhịp ⛔ không bao giờ khẳng định một con số nó ⛔ không biết"*. ✅ **CHỐT (a) — cài `COUNT` như đã chốt ở Task 0.** Truy vấn `COUNT(*) … GROUP BY source_id` chạy **CHỈ KHI** `truncated = true`, đúng đường (a) của §hệ quả ③; thanh nhịp nói đúng số thật thay vì một cận dưới được diễn đạt bằng chữ.
- [x] [Review][Patch] **`pos_lang = "vi"` bị đánh dấu là nhãn NGOẠI NGỮ** — `LookupRecord.vue:35` `v-if="sense.pos_lang !== null"` bật chip `.lookup-foreign-flag`; fixture `dict_sources.rs:283,294,368` khai `pos_lang: Some("vi")` ⇒ nhãn **tiếng Việt** hiện chip `VI` màu `primary`. FR35/AC4 chỉ đòi đánh dấu nhãn **ngoại ngữ**. Vế *"đọc TRƯỜNG chứ ⛔ không đoán"* thì đúng; vế *"ngoại ngữ"* thì sai. ✅ **CHỐT (a) — Rust trả cờ `is_foreign`.** Quy tắc *"ngôn ngữ nào là ngoại ngữ"* là quy tắc nghiệp vụ ⇒ sống ở Rust (AD-1), ⛔ ở webview. Chấp nhận việc đụng hình dạng bản ghi trên dây của Quyết định #2 — đây là **thêm** một trường, ⛔ đổi ngữ nghĩa trường đã có. Áp cho **cả** `SenseRecord::pos_lang` lẫn `ExampleRecord::translation_lang` (AC4 nói *"cùng luật"*).
- [x] [Review][Patch] **Vùng đầu mục ⛔ KHÔNG cố định — nó đi từ 0 → 76px khi rời trạng thái "chưa tra gì"** — `LookupPanel.vue:55` `<div v-if="!neverLookedUp">` bọc **cả** `.lookup-head` (`:109-113`, `height: var(--lookup-head-height)`). AC7 đòi *"chiều cao ⛔ không đổi một pixel — **kể cả đổi giữa bốn trạng thái của AC6**"*, mà "chưa tra gì" là trạng thái thứ nhất. Hằng có tên thì có; bất biến mà nó tồn tại để giữ thì ⛔ không. ✅ **CHỐT (a) — luôn render `.lookup-head`.** Khung 76px có mặt ở **cả bốn** trạng thái, đúng chữ AC7. ⚠️ Phải xử vế chồng lấn: `showFrameStatus` hôm nay bật câu mặc định của `PanelFrame` ở đúng trạng thái 1 — cần quyết chỗ đặt câu dạy-thao-tác sao cho ⛔ hiện hai câu cùng lúc và ⛔ phá bất biến chiều cao.
- [x] [Review][Patch] **`EntryHit.headword`/`headword_simp` ⛔ không bao giờ được render — đầu mục hiển thị là TRUY VẤN THÔ** — `LookupPanel.vue:59` in `{{ currentQuery }}`; `grep headword src/panels/` ⛔ không chỗ nào đọc `EntryHit.headword`. Hệ quả ①: khớp qua `headword_simp` *(giản thể)* hiện **sai chữ** so với đầu mục thật trong từ điển. Hệ quả ②: Quyết định #5(a) đòi *"mỗi đầu mục là một cụm nghĩa của CHÍNH NÓ, hiện liền nhau"*, nhưng `LookupRecord.vue:33` lặp trên một danh sách `senses` **phẳng** ⇒ 18 ca trùng `headword` của `dict-vietphrase.db` đọc ra thành một danh sách dài liên tục — gần với *"gộp hiển thị"* (đường story ⛔ chọn) hơn là *"hiện liền nhau"*. ✅ **CHỐT (a) — render `headword` thật + nhóm nghĩa theo `entry_id`.** Sửa **cả hai** vế: đầu mục lấy từ `EntryHit` *(đóng lỗi hiện-sai-chữ khi khớp qua `headword_simp`)*, và `LookupRecord` gom nghĩa thành **từng cụm một đầu mục** thay vì một danh sách phẳng — đúng Quyết định #5(a) *"mỗi đầu mục là một cụm nghĩa của chính nó, hiện liền nhau, ⛔ đánh số, ⛔ gộp"*. ⚠️ Đầu mục ở `.lookup-head` *(một dòng, `wraps: false`)* vs đầu mục **theo từng nhóm** là hai chỗ khác nhau — nhiều nguồn có thể cho nhiều `headword` khác nhau cho cùng một truy vấn.
- [x] [Review][Patch] **Kết luận "NFR1 ĐẠT" dùng p95 mà bỏ qua p99 của chính đường sản phẩm** — §Debug Log `:1219` ghi `exact_btree 山 … p99 **70.742ms** ← ca xấu nhất đường sản phẩm`, nhưng `:1226` kết luận *"p95 6.535 ms … ⇒ NFR1 ĐẠT"* và `:1264` ước tính *"đầu-cuối tổng < 40ms"*. p99 backend 70,7 ms + render p50 ≈ 31 ms ≈ **101 ms**, chạm/vượt trần NFR1 100 ms — và điều đó ⛔ không được nêu ở §giới hạn phép đo *(vốn rất trung thực về các vế khác)*. AC11 đòi *"ghi SỐ chứ ⛔ không ghi lời hứa"*: số có, kết luận là lời hứa. ✅ **CHỐT (b) — ĐO LẠI / tối ưu TRƯỚC khi đóng story.** ⛔ Đóng story trên một con số chưa hiểu. p99 gấp **~10,8×** p95 *(70,742 vs 6,535)* là hình dạng bất thường — nghi warm-up trang/IO nhiễu/lượt đo đầu, nhưng **nghi ⛔ phải là biết**. Việc phải làm: đo lại `exact_btree 山` với số vòng lớn hơn + bỏ vòng warm-up, xác định p99 thật; nếu 70,7 ms là thật thì tối ưu hoặc ghi thẳng NFR1 ⛔ đạt ở p99. Đúng nguyên tắc *"đo trước khi chốt kiến trúc"* mà chính story này áp cho Quyết định #4.
- [x] [Review][Patch] **Vị từ `sourcesDisagree` ⛔ không so sánh gì cả, nhưng chuỗi i18n phát biểu một SỰ KIỆN** — `lookupPanelState.ts:31` `return groups.length >= 2`; `vi.json:69` = *"Các nguồn ghi khác nhau ở đây"*. Hai nguồn ghi **giống hệt nhau** — đúng ca AC1 dựng riêng — vẫn kích hoạt câu này. AC5 mở bằng *"Given hai nguồn ghi **khác nhau**"*. Vế *"hàm thuần, xuất được, test được"* thoả về hình thức, ⛔ không về nội dung. ✅ **CHỐT (a) — đổi CHUỖI thành mệnh đề khả năng.** `vi.json` nói *"Các nguồn có thể ghi khác nhau — đối chiếu bên dưới"*. Vị từ giữ nguyên hình dạng rẻ và thuần, và chuỗi khớp đúng thứ nó **thật sự biết** *(có ≥ 2 nguồn)*. ⛔ Đẩy một phép so sánh nội dung vào webview — đó là hình dạng tư duy hợp nhất mà AD-19 cấm, chỉ nhỏ hơn.
- [x] [Review][Patch] **Lỗi IPC ⇒ thân panel TRẮNG CÂM; `lookupError` là một export ⛔ không ai tiêu thụ** [src/panels/lookupPanelState.ts:86 · src/panels/LookupPanel.vue:17-28] — 🔴 **nặng nhất.** `grep lookupError src/**` cho **0 chỗ import**. Khi `lookupDictionary()` trả lỗi: `neverLookedUp === false` ⇒ `PanelFrame` tắt câu mặc định; `lookupResolved === false` ⇒ cả khối kết quả lẫn **cả bốn** chuỗi rỗng đều ⛔ render. Người dùng nhận đúng một dòng đầu mục và ⛔ không một câu nào giải thích. `vi.json` cũng ⛔ không có khoá lỗi tra cứu nào. Vi phạm AC6 · AD-44 ④ · UX-DR27 — và là **tái phát nguyên văn** khuyết tật mà code review 1.16 đã bắt *(bằng chứng còn trong cây: `SourcePanel.vue:56`)*, đúng thứ §Trí tuệ từ story trước dán vào để ⛔ học lại bằng tiền.
- [x] [Review][Patch] **Kết quả cũ bị xoá trắng trong khoảng chờ, banner cũ thì ở lại — và chú thích tại chỗ nói NGƯỢC với mã** [src/panels/LookupPanel.vue:34-37,72-73 · src/panels/lookupPanelState.ts:130,136,139,142] — chú thích `:34-37` khẳng định *"panel giữ nguyên trạng thái TRƯỚC ĐÓ cho tới khi lượt tra mới trả lời"*, nhưng `groupedLookup`/`sensesByLayer` cổng qua `lookupResolved` (`false` ngay khi `pending`) ⇒ toàn bộ bản ghi biến mất. Cùng lúc `someLayerFailed`/`someLayerTruncated` đọc thẳng `response.value` **⛔ qua** `lookupResolved` ⇒ banner của truy vấn **CŨ** nổi trên vùng bản ghi trống, dưới đầu mục **MỚI**. Đúng lớp nháy mà AC7 tồn tại để chặn.
- [x] [Review][Patch] **`runLookup` ⛔ có số thứ tự lượt; `resetLookupPanel` ⛔ huỷ lượt đang bay** [src/panels/lookupPanelState.ts:152-162,172-177] — `query.value` đặt **trước** `await`, `response.value` đặt bởi *bất kỳ* lượt nào về trước. Hai lần `Mod+Alt+L` liên tiếp: nếu lượt A về sau lượt B ⇒ đầu mục B kèm bản ghi A, `pending` đã tắt nên `lookupResolved === true` — ⛔ không dấu hiệu nào báo sai, và trạng thái đó **vĩnh viễn**. Cùng lỗ: đổi Tác phẩm trong lúc một lượt đang bay ⇒ promise ghi **sau** `resetLookupPanel()` làm kết quả Tác phẩm A sống lại dưới Tác phẩm B — đúng thứ reset tồn tại để chặn (Bẫy 8). Story 1.18 (Auto-Lookup) sẽ biến lỗ này thành thường trực.
- [x] [Review][Patch] **`QUERY_LENGTH_CEILING` cắt truy vấn IM LẶNG ⇒ "Không tìm thấy trong từ điển" là một câu SAI** [src-tauri/src/commands/dict.rs:59,94] — bôi đen > 200 ký tự bị cắt còn 200 rồi tra `Exact` ⇒ chắc chắn 0 kết quả ⇒ panel hiện `panel.lookup.not_found`, trong khi hệ thống **chưa hề tra** thứ người dùng chọn. Story đã dựng đúng cơ chế cần thiết (`truncated`/`truncated_layers`) cho trần số HÀNG mà ⛔ áp cùng nguyên tắc cho trần độ DÀI. Sửa: thêm một cờ vào `LookupResponse` *(vỏ lệnh, ⛔ đụng hình dạng bản ghi của Quyết định #2)* và một chuỗi riêng.
- [x] [Review][Patch] **`limit as i64` biến một trần lớn thành `LIMIT 0` — mất sạch dữ liệu, im lặng, ở một hàm `pub`** [src-tauri/src/core/dict/query.rs:141,197,301] — `(limit as i64).saturating_add(1)`: với `limit = usize::MAX` *(thành ngữ tự nhiên nhất cho "không giới hạn")* kết quả là `-1 + 1 = 0` ⇒ SQL trả 0 hàng, và `cap()` (`:84-88`) báo `truncated = false` vì `0 > usize::MAX` sai. `saturating_add` đang bảo vệ **sai chỗ** — phép ép kiểu đã tràn trước đó. Bằng chứng gián tiếp: cả hai tệp test phải bịa `const UNLIMITED: usize = 10_000` thay vì dùng `usize::MAX`, tức *"không giới hạn"* ⛔ biểu đạt được trong API này và mọi số đo "trước `LIMIT`" thật ra chạy với `LIMIT 10001`. Cùng hàm: `limit == 0` cho `groups` rỗng **kèm** `truncated = true` ⇒ panel hiện đồng thời "không tìm thấy" và "danh sách chưa đầy đủ". Sửa: `i64::try_from(limit).unwrap_or(i64::MAX)` + chặn `limit == 0`.
- [x] [Review][Patch] **Hai sàn được AC13 gọi ĐÍCH DANH ⛔ không được nâng** [scripts/check-commands.mjs:224 · scripts/check-i18n.mjs:171] — AC13 liệt kê `CLICK/DISPATCH_FLOOR` **6** vs 8 và `RS_FLOOR` **32** vs 39, và đòi *"**mọi** hằng `*_FLOOR` bị vượt được nâng theo số thật"*. Cả hai bị đánh dấu *"(không đổi ở Story 1.17)"* thay vì được nâng: `CLICK_FLOOR = 6` với số thật 8 (75%), `RS_FLOOR = 32` với số thật 40 (80%) — dưới doctrine ~81–85% mà chính các sàn khác trong cùng lượt tuân theo. Đúng cách 1.16 để lọt và bị bắt.
- [x] [Review][Patch] **Sổ sách `DISPATCH_FLOOR` ghi SAI xuất xứ — story này ⛔ thêm một `dispatch()` nào** [scripts/check-commands.mjs:200-201,225] — chú thích khai *"12 lời gọi `dispatch()` (trước story: … 8)"*. Đếm thật bằng chính `DISPATCH_CALL_RE` của cổng: `.vue` **13 → 13**, `.ts` **4 → 4** — **delta = 0**. Số "8" là ghi chép cũ chưa cập nhật từ 1.16, và Task 10/AC13 vừa chép lại nó thành một mệnh đề nhân quả sai. Sàn nâng lên 10 thì vô hại, nhưng một con số bịa trong đúng tệp mà cả kiến trúc dựa vào để tin các con số là chính thứ rot mà AC13 tồn tại để chặn.
- [x] [Review][Patch] **Thiếu ca đỏ-rồi-xanh BẮT BUỘC cho token thứ 17** [scripts/check-tokens.mjs · src/tokens/tokens.json] — Quyết định #7 kết đoạn gọi đích danh: *"token mới **phải** khai `wraps: true` — nếu không, nó chỉ là một hàng JSON nữa mà cổng vẫn mù… **Đó là ca đỏ-rồi-xanh bắt buộc của AC13 cho token 17**"*. Completion Notes chỉ ghi **một** kiểm chứng frontend (`COMMAND_FLOOR = 18` giả lập → đỏ → khôi phục). ⛔ Không ca nào hạ `ui-md-wrap.lineHeight` xuống 1.5 hoặc bỏ `wraps: true` để chứng minh Kiểm E **thật sự cắn** trên token mới — tức mệnh đề đắt nhất của story *(đóng `deferred-work.md:115` sau **bốn** lần bị gọi tên)* ⛔ chưa được chứng minh là có lưới.
- [x] [Review][Patch] **`LOOKUP_PAGE_LIMIT` ⛔ được ghim bởi một ca test nào — có thể tụt xuống 1 mà cả bộ vẫn xanh** [src-tauri/src/commands/dict.rs:51 · src-tauri/tests/dict_sources.rs] — hai ca AC12 gọi `lookup_grouped(..., 3)` với trần truyền **tay**, ⛔ đi qua `commands::dict::lookup`. Các ca đi qua command dùng fixture ≤ 1–2 đầu mục mỗi lớp, nên đặt `LOOKUP_PAGE_LIMIT = 1` vẫn xanh toàn bộ. Hằng mang **cả chính sách sản phẩm của Quyết định #4** lại là thứ duy nhất trong story ⛔ có lưới hồi quy.
- [x] [Review][Patch] **Thanh nhịp render `"7 · 22"` — mất chữ "nguồn"/"nghĩa"** [src/panels/LookupPanel.vue:62 · src/i18n/vi.json] — mockup `lookup-real-density.html:116` là `5 nguồn · 22 nghĩa`; Completion Notes tự xác nhận kết quả nghiệm thu bằng mắt là `"2 · 3 CVDICT 2 THIỀU CHỬU 1"` — một dãy số ⛔ đọc được, trong khi lý do tồn tại của thanh nhịp là *"biết hình dạng trước khi cuộn"*. `vi.json` ⛔ thêm khoá đơn vị nào ⇒ vế này ⛔ chưa làm chứ ⛔ không phải làm khác.
- [x] [Review][Patch] **Vết phê duyệt và File List lệch sổ** [Change Log · §Project Structure Notes] — Completion Notes viện dẫn *"**Ice xác nhận (b)**"* cho việc AC12 chấp nhận nguồn thứ hai vắng mặt, nhưng Change Log *(nơi mọi lượt chốt của Ice đều có một hàng ngày tháng)* ⛔ có hàng nào ghi lượt đó. Đường (b) **được** nhánh `Or` của AC12 cho phép nên đây ⛔ phải vi phạm — nhưng vết phê duyệt cần có mặt trước khi story rời `review`. Cùng hạng: `src-tauri/tests/ipc_contract.rs` được §Project Structure Notes ghi `UPDATE` mà ⛔ được đụng.
- [x] [Review][Defer] **Chip thanh nhịp bị `overflow: hidden` cắt CÂM khi nhiều nguồn** [src/panels/LookupPanel.vue:109-113,131-137] — deferred, đánh đổi có chủ đích đã ghi trong chú thích tại chỗ *(giữ bất biến AC7 quan trọng hơn)*; đo thật `山` cho **7–8 nhóm** ⇒ chip tràn quá hai dòng bị cắt, ⛔ dấu hiệu, ⛔ cuộn.
- [x] [Review][Defer] **`layers_loaded = false` khi MỌI tệp `.db` hỏng ⇒ hiện "chưa gắn lớp nào" (sai)** [src-tauri/src/core/dict/layer.rs:460-493 · mod.rs] — deferred, ca hiếm; `DictLayers::new` đẩy lớp mở hỏng vào `skipped` chứ ⛔ vào `layers`, nên `layers().is_empty()` đúng về cơ học mà sai về chẩn đoán. Banner `someLayerFailed` vẫn hiện song song nên người dùng ⛔ bị bỏ câm.
- [x] [Review][Defer] **Pha hai trượt ⇒ khối nguồn hiện tên với 0 nghĩa, ⛔ phân biệt được với đầu mục chỉ có âm đọc** [src-tauri/src/commands/dict.rs:109-118] — deferred, `else { continue }` + `senses(...).unwrap_or_default()` nuốt lỗi hydrate sau khi pha một đã thành công; đã được ghi là món nợ ở deferred-work *("`senses()` trượt không có tín hiệu riêng")*.
- [x] [Review][Defer] **`window.getSelection()` mù với `<input>`/`<textarea>`; vùng chọn rỗng = im lặng tuyệt đối** [src/main.ts:182 · src/commands/index.ts:475-477] — deferred, Story 1.18 sở hữu hợp đồng vùng chọn thật; dep hôm nay là dep TỐI THIỂU theo đúng Quyết định #1a.
- [x] [Review][Defer] **Nhánh `Substring`/`fts_trigram` nạp TOÀN BỘ hàng khớp vào RAM trước khi cắt** [src-tauri/src/core/dict/query.rs:203-221,238-254,321-333] — deferred, latent: đường sản phẩm 1.17 là `Exact`-only nên ⛔ chạm tới; trở thành thật khi 1.18/7.7 bật `Substring`. Trần an toàn ở SQL (vd `limit * 50`) vẫn giữ được thứ tự "verify rồi mới `cap`".
- [x] [Review][Defer] **Chuỗi `query_too_short` chỉ dẫn một thao tác ⛔ tồn tại trong panel** [src/i18n/vi.json:65] — deferred, `"gõ thêm ít nhất ba ký tự"` nhưng Panel Lookup ⛔ có ô nhập nào; story tự khai nhánh này ⛔ thực thi được qua đường sản phẩm `Exact`-only hôm nay. Story 1.18/7.7 sẽ kế thừa nguyên văn.

---

## Dev Notes

### Trạng thái repo hôm nay — SỐ, ⛔ không phải mô tả

| | Số thật (2026-08-06, `cb03974`) |
|---|---|
| `#[tauri::command]` đã đăng ký | **6** — `bootstrap_config` · `put_config` · `create_work_from_text` · `create_work_from_file` · `read_open_chapter` · `read_han_viet` |
| Đường IPC **tra cứu từ điển** | 🔴 **⛔ KHÔNG CÓ** — story này dựng nó |
| Người tiêu thụ `DictLayers` từ state | 🔴 **0** — `app.manage(layers)` ở `lib.rs:253`, ⛔ chưa ai lấy ra *(`deferred-work.md:453`)* |
| Kiểu bản ghi tra cứu derive `Serialize` | 🔴 **0/11** — xem Quyết định #2 |
| Method trên `DictionarySource` | **5** — `layer()` · `sources()` · `lookup()` · `senses()` · `han_viet()` |
| Người tiêu thụ token `lookup-headword`/`-gloss`/`-example` | 🔴 **0** — story này là **người đầu tiên** |
| Thân Panel Lookup | **trống** — `LookupPanel.vue` là **18 dòng**, render đúng một `PanelFrame` |
| Token typography | **16** *(14 của `DESIGN.md` + `ui-md-strong` + `source-latin`, cả hai qua sổ `deviations`)* |
| Khoá `vi.json` | **63** |
| Command trong `CommandRegistry` | **16** *(3 mode · 2 preset · 4 toggle · 2 focus · 2 library · 3 source)* |
| Tệp `.vue` / `.ts` / `src/**` / `.rs` | **12** / **23** / **50** / **39** |
| `matchMedia` / `window.innerWidth` trong `src/**` | **0** / **0** — phải giữ nguyên |
| Tệp `.db` từ điển trong git | **0** *(`.gitignore: *.db`, AD-25)* — mọi bản dựng hôm nay lên với **0 lớp** |

### API thật — chép từ MÃ, ⛔ không từ trí nhớ

```rust
// core/dict/mod.rs — PHA MỘT
pub fn lookup_grouped(layers: &DictLayers, query: &str, mode: LookupMode) -> GroupedLookup;
pub struct GroupedLookup { pub route: QueryRoute, pub branch: QueryBranch,
                           pub groups: Vec<SourceGroup>, pub skipped: Vec<SkippedLayer> }
pub struct SourceGroup   { pub layer: String, pub source: SourceInfo, pub entries: Vec<EntryHit> }
pub struct SourceInfo    { pub code: String, pub display_name: String }
pub struct EntryHit      { pub entry_id: i64, pub source_code: String, pub lang: String,
                           pub headword: String, pub headword_simp: Option<String> }
pub enum QueryBranch { ExactBtree, CharIdx, FtsTrigram, NoBranchQueryTooShort }
pub enum QueryRoute  { Zh, En }
pub enum LookupMode  { Exact, Substring }

// core/dict/mod.rs — PHA HAI (qua cổng)
pub struct SenseRecord { pub entry_id: i64, pub sense_id: i64, pub pos: Option<String>,
                         pub pos_lang: Option<String>, pub gloss: String,
                         pub note: Option<String>, pub ord: i64,
                         pub examples: Vec<ExampleRecord>, pub citations: Vec<CitationRecord> }
pub struct ExampleRecord  { pub text: String, pub translation: Option<String>,
                            pub translation_lang: Option<String>, pub ord: i64 }
pub struct CitationRecord { pub text: String, pub work: Option<String>,
                            pub author: Option<String>, pub ord: i64 }

// core/dict/layer.rs
impl DictLayers { pub fn layers(&self) -> &[DictLayer];
                  pub fn skipped(&self) -> &[SkippedLayer];
                  pub fn layer(&self, layer: &str) -> Option<&DictLayer>;   // ← đường PHA HAI
                  pub fn empty() -> DictLayers; }
// ⚠️ `DictLayer::source(code)` là `pub(super)` — ⛔ KHÔNG gọi được từ `commands/`.
//    ⛔ Không cần: `SourceGroup` đã cầm sẵn `SourceInfo` đầy đủ.

// ports/dict_source.rs
fn senses(&self, entry_ids: &[i64]) -> Result<Vec<SenseRecord>, StoreError>;

// lib.rs
app.manage(layers);                          // :253  — DictLayers, có thể RỖNG
app.try_state::<DictLayers>()                // khuôn đọc, xem commands/dict.rs
```

### Doctrine đã chốt ở 1.11/1.13/1.16 mà story này **thừa kế nguyên**

- **Vị từ điều phối chạy ĐÚNG MỘT LẦN cho cả lượt tra**, ở tầng gom. `lookup_grouped` đã làm
  đúng — ⛔ **đừng** gọi `pick_route`/`pick_branch` lại ở `commands/` hay ở webview.
- **Khoá theo `source.code` (chuỗi), ⛔ không `source_id` (số)** — mỗi tệp `.db` có bảng
  `dict_source` **riêng**, `id = 1` tồn tại ở **cả ba** tệp và trỏ ba nguồn khác nhau.
- **⛔ Không sổ đăng ký "tệp nào chứa gì"** *(AD-44 ① vá A2)*.
- **Hàm thuần là đường sản phẩm; vỏ `#[tauri::command]` là thứ bỏ đi được trong test.**
- **Rỗng im lặng bị cấm; rỗng CÓ LÝ DO thì không** *(AD-44 ④)* — và story này là chỗ mệnh đề
  đó cuối cùng có một màn hình để nói ra.

### ⚠️ MƯỜI MỘT CÁI BẪY — chín trong mười một cho ra một lượt CI **XANH** với kết quả **VÔ NGHĨA**

1. 🔴 **Derive `Serialize` thẳng lên `SkipReason`** ⇒ lỗi thô SQLite đi lên giao diện. Vi
   phạm AD-21 ở đúng chỗ ⛔ không cổng nào nhìn *(`check-i18n.mjs` Kiểm A quét **chuỗi
   trong mã**, ⛔ không quét **dữ liệu chạy qua dây**)*.
2. **Gộp/khử trùng ở tầng webview** ⇒ AD-19 vỡ. `no_function_merges_meanings_across_sources`
   **đếm tệp Rust**, ⛔ hoàn toàn mù với `src/**`. Một `new Set(...)` trên `gloss` là đủ.
3. 🔴 **Trộn `entry_id` giữa các lớp ở pha hai** ⇒ đọc nhầm nghĩa, **⛔ không lỗi nào được
   ném**. Ca test một-lớp ⛔ không dựng được điều kiện tiên quyết — phải **hai** lớp, cùng
   `entry_id` số học, nghĩa khác nhau.
4. **Hydrate pha hai cho TOÀN BỘ `groups`** ⇒ 13,015 ms *(số đo `:446`)* thay vì 0,3 ms cho
   một trang. ⛔ Không cổng nào đỏ; nó chỉ chậm.
5. **`branch == query_too_short` hiện thành "không tìm thấy"** ⇒ người dùng đi tìm một từ
   đang có trong từ điển. Đúng lớp lỗi AD-44 ④ ra đời để chặn.
6. **`skipped` bị bỏ qua** ⇒ *"một phần từ điển ⛔ không trả lời"* trông y hệt *"⛔ không có
   kết quả"*. `SkippedLayer` tồn tại **chỉ** để phân biệt hai câu đó.
7. **Bề mặt chữ quên khai token của chính nó** ⇒ chạy ở `ui-md` giãn dòng **1.5**, dưới sàn
   1.66, dấu `ườ` chạm dấu `ộ` — Kiểm E **chỉ đọc `tokens.json`**, hoàn toàn mù.
8. **State module-level ⛔ không có đường reset** ⇒ Tác phẩm B hiện kết quả của A. **Lỗi này
   đã xảy ra THẬT ở 1.16** và tốn một lượt code review để bắt — ⛔ đừng học lại bằng tiền.
9. 🔴 **`LIMIT` cấp-tệp làm một nguồn biến mất** ⇒ FR31 vỡ, ⛔ không lỗi, ⛔ không cổng nào
   đỏ, và **fixture ba-lớp-mỗi-lớp-một-nguồn hiện có ⛔ không dựng được điều kiện tiên
   quyết** — phải là **nhiều nguồn TRONG một tệp**. Xem AC12.
10. 🔴 **Tin rằng `LIMIT` đã cắt được thời gian mà ⛔ không đo.** Sáu câu SQL kết bằng
    `ORDER BY e.id`; một kế hoạch phải sắp toàn bộ trước thì `LIMIT` ⛔ **không** cắt một
    mili-giây nào — nhưng test vẫn xanh, kết quả vẫn đúng, và story vẫn *"đã thêm `LIMIT`"*.
    ⇒ đây là bẫy **đắt nhất** của story: một thay đổi mổ vào chữ ký cổng, đổi sáu câu SQL và
    mọi test hành vi, để mua **0 ms**. `EXPLAIN QUERY PLAN` ở Task 0 tồn tại đúng vì nó.
11. 🔴 **Đặt `LIMIT` TRƯỚC `verify_substring` ở nhánh 2 ký tự** ⇒ *"còn M nữa"* **nói dối**,
    và trang hiện ra **ít hơn `N`** mục mà ⛔ không ai giải thích được vì sao.
    `query.rs::char_idx` chạy `INTERSECT` rồi **lọc lại bằng `verify_substring`** trên kết
    quả *(một tập ứng viên, ⛔ không phải một tập đã đúng)*. `LIMIT N` ở SQL cắt **ứng
    viên**, rồi `verify` loại tiếp ⇒ còn lại **< N**. ⚠️ Cùng bẫy áp cho `fts_trigram`
    *(FTS5 trigram cũng trả ứng viên)*. ⇒ trần phải áp **sau** bước xác minh, hoặc lấy
    `N + biên` rồi cắt ở Rust — **và ca test phải dựng đúng tình huống `verify` loại bớt**,
    ⛔ không chỉ ca *"khớp sạch"*.

### 🔴 BỐN mâu thuẫn tài liệu đã phát hiện — ⛔ dev KHÔNG sửa tài liệu, chỉ NÓI RA

1. **`EXPERIENCE.md:389` bảo trỏ sang Concordance; `epics.md` §Story 1.17 cấm đích danh.**
   *"Không tìm thấy thì gợi ý tra từng chữ **và trỏ sang Concordance**"* vs *"⛔ **không trỏ
   tới bất kỳ năng lực nào chưa tồn tại** — đường sang Concordance được bổ sung ở Story 7.7."*
   ⇒ **`epics.md` thắng** *(nó là tài liệu nghiệm thu của story)*. Ghi ra, ⛔ không sửa.
2. **Mockup `lookup-real-density.html` vẽ tab `Từ điển | Concordance` ở thanh tiêu đề panel**
   — nhưng Story 1.14 §Quyết định #4A đã **gỡ `<header>`** của `PanelFrame`, và 1.16 §Quyết
   định #5 đặt dải tab ở **đầu THÂN panel**. ⇒ theo tiền lệ 1.16. Cùng hạng với mâu thuẫn #3
   mà 1.16 đã ghi.
3. **`QueryBranch::NoBranchQueryTooShort` viện dẫn "FR41, Story 1.17"** trong doc-comment
   *(`core/dict/mod.rs:192`)*, nhưng **FR41 là lịch sử + ghim (Story 1.20)**. Mệnh đề *"truy
   vấn quá ngắn"* thuộc **AD-44 ④**, ⛔ không thuộc FR41. Nội dung đúng, **trích dẫn sai** —
   ghi ra.
4. 🔴 **Vạch trái 2px + thụt 13px treo ở CẤP NÀO — hai tài liệu nói hai cấp.** `epics.md`
   §Story 1.17 và `DESIGN.md:360` đọc ra là **khối NGUỒN** *("mỗi nguồn là một khối riêng
   xếp chồng dọc, **có vạch trái 2px và thụt 13px**")*; mockup thì đặt
   `border-left: 2px + padding-left: 13px` lên **`.sense`** — **từng NGHĨA** — còn khối nguồn
   `.src` chỉ có một `border-bottom` dưới nhãn *(`lookup-real-density.html:64-68`)*.
   ⚠️ Khác biệt **nhìn thấy được**: một nguồn 9 nghĩa cho ra **một** vạch hay **chín** vạch.
   ⇒ Chốt ở Task 0 theo **mockup** *(nó là bản vẽ ở mật độ thật, và chín vạch chính là thứ
   làm mắt nhặt được ranh giới nghĩa)* **hoặc** theo `epics.md`; ⛔ dù chọn bên nào cũng
   **ghi ra**, ⛔ không sửa tài liệu.

### Bàn giao — CHÍN mục `deferred-work.md` gọi đích danh Story 1.17

| Dòng | Mục | Story này làm gì |
|---|---|---|
| `:453` | `app.manage(DictLayers)` ⛔ chưa có người tiêu thụ — *"đó là **Story 1.17**"* | ✅ **Đóng** (AC8) |
| `:129`, `:131` | Bề mặt ĐỌC phải khai `read-*`/`lookup-*` của chính nó | ✅ **Đóng nửa Lookup** (AC9). Nửa Editor ở Epic 2 |
| `:133` | `--synthesis-*` — `lookup-example` là token **thứ hai** | ✅ **Đóng** (AC9) |
| `:113` | Rà lại **từng cờ `wraps`** khi 1.14/1.17 dựng panel | ✅ Rà ở Task 7; kết quả ghi ra |
| `:343`, `:419`, `:449` | NFR1 — trần trang là quyết định sản phẩm của **1.17** | 🔴 **Ice chốt: có `LIMIT` pha một** (Quyết định #4). Cỡ trang từ SỐ ĐO (AC11) |
| `:115` | Cờ `wraps` của `ui-md` — bị gọi tên **bốn** lần từ Story 1.4 | 🔴 **Ice chốt: token thứ 17 `ui-md-wrap`** ⇒ ✅ **ĐÓNG** (Quyết định #7, AC9) |
| `:416` | 18 đầu mục trùng `headword` trong VietPhrase — *"bàn giao trình bày cho 1.17"* | ✅ Quyết định #5 |
| `:363` | ⛔ Không giới hạn độ dài truy vấn — *"validate thuộc tầng IPC/UI của 1.13/**1.17**"* | ✅ Task 2 — một sàn trên **có tên**, ⛔ không một `panic` |
| `:504` | `applyPreset()` mất trạng thái panel — *"nhặt lại ở 1.16 / **1.17** / Epic 2"* | ✅ (AC10) |
| `:317` | 🟡 *"Panel Lookup có hình dạng hiển thị cho mục từ **TIẾNG ANH** chưa"* — chủ sở hữu **Sally (UX)** | ⚠️ **⛔ KHÔNG ĐÓNG.** Ice chốt **tạm dùng mặc định** của story; chữ ký UX vẫn thiếu. Task 11 ghi lại nguyên trạng |

### 🧠 Trí tuệ từ story trước — thứ đắt tiền, ⛔ đừng học lại bằng tiền

- **Story 1.16 · code review:** một hàm `export` mà ⛔ **không một chỗ nào `import`** là một
  lỗi **im lặng hoàn toàn** — `sourceChapterError`/`sourceHanVietError` được export, ⛔ không
  ai tiêu thụ, nên `err.project.no_work_open` *(có `MessageKey` riêng và **ba** test Rust)*
  ⛔ không bao giờ tới được màn hình. ⇒ ở story này: **mỗi** vị từ trạng thái rỗng phải có
  một chỗ tiêu thụ **nhìn thấy được**, và Task 10 phải kiểm điều đó bằng mắt.
- **Story 1.16 · code review:** *"ba trạng thái sập thành một ở đường lỗi và đường chờ"* —
  một mặc định `?? true` biến *"chưa tra được"* thành *"đã tra mà không có"*. ⇒ ở đây có
  **bốn** trạng thái rỗng cộng ca "0 lớp"; ⛔ **đừng** để một `??` nào phán quyết thay.
- **Story 1.16 · code review:** một test *"chạy đúng"* mà ⛔ không dựng được điều kiện tiên
  quyết của lỗi là một **test giả**. ⇒ Bẫy 3.
- **Story 1.16:** một `invoke()` trượt bằng thứ ⛔ không phải `IpcError` **khi có cầu IPC** là
  một **lỗi THẬT**, ⛔ không phải *"chạy ngoài Tauri"*. Dùng **lại** khuôn `src/config/dict.ts`
  *(`isIpcError` + `hasIpcBridge`)*, ⛔ đừng viết khuôn thứ hai.
- **Story 1.14:** `dockview-vue` mount **mọi** component với **đúng một** prop tên `params`
  *(`src/layout/panelProps.ts`)*. ⛔ Đừng khai prop khác cho `LookupPanel`.
- **Story 1.14:** trạng thái tiêu điểm đọc từ **DOM thật**, ⛔ không từ `activePanel`. ⛔
  **Đừng đụng `PanelFrame` ở vế này.**
- **Story 1.13:** *"⛔ không một câu SQL nào được chuẩn bị"* cho trạng thái ⛔ không hỗ trợ —
  trạng thái phải **phân biệt được** với *"đã chạy mà ⛔ không tìm thấy"*.

### Testing standards

Bộ DoD **chín lệnh** *(khuôn 1.14/1.15/1.16)* — **mã thoát là phán quyết**, ⛔ không phải đầu ra:

```
cargo test --manifest-path src-tauri/Cargo.toml
npm run build            npm run check:tokens     npm run check:i18n
npm run check:commands   npm run check:layout     npm run check:deps
npm run check:dict-manifest                       npm run check:scope
```

- **⛔ Không có bộ chạy test frontend, và ⛔ không được thêm** *(NFR15, quyết định của Ice đã
  chốt ở 1.5 và giữ qua **năm** story)*. ⇒ vế DOM nghiệm thu bằng **bảng chạy tay có số**,
  ghi vào §Debug Log References — **⛔ không** bằng văn xuôi.
- **Test Rust là test HÀNH VI qua biên**, đặt ở `src-tauri/tests/**`. Đường tra cứu thuộc họ
  `dict_*` — **dùng lại fixture ba lớp của `dict_sources.rs`/`dict_lookup.rs`**, ⛔ đừng dựng
  bộ fixture thứ hai.
- 🔴 **Test FR36 phải XOÁ TỆP THẬT rồi chạy lại** *(`epics.md:818`)*. ⚠️ Và **đóng lớp trước
  khi xoá** — một tệp `.db` còn mở lúc xoá làm test **đỏ trên Windows** (NFR14); đúng ca đã
  bị bắt ở code review 1.16.
- **Đỏ-rồi-xanh cho mọi cổng bị đụng**: mỗi mệnh đề mới phải có **ít nhất một ca làm cổng
  ĐỎ** cộng **một đối chứng âm**. Con số ghi vào Completion Notes.

### Project Structure Notes

```
src-tauri/src/
  commands/dict.rs        UPDATE  hàm thuần tra cứu + `mod wire` — khuôn `read_han_viet`
  lib.rs                  UPDATE  generate_handler![…] — ⛔ KHÔNG đụng capabilities/
  core/dict/mod.rs        UPDATE  derive Serialize + trần đi xuống mọi tệp (#4 ĐÃ CHỐT)
  core/dict/layer.rs      UPDATE  derive Serialize cho SkippedLayer/SkipReason-wire + impl trần
  core/dict/query.rs      UPDATE  🔴 SÁU hình dạng SQL — `LIMIT ?N` tham số RÀNG BUỘC
  ports/dict_source.rs    UPDATE  🔴 ĐỔI CHỮ KÝ `lookup` — ⛔ KHÔNG thêm method thứ sáu
  tests/dict_sources.rs, tests/dict_lookup.rs, tests/dict_boundary.rs, tests/ipc_contract.rs  UPDATE

src/
  panels/LookupPanel.vue        UPDATE  ⛔ thay doc-comment "khung, ⛔ không phải nội dung"
  panels/LookupRecord.vue       NEW     khối một nguồn — ⚠️ khai token của CHÍNH NÓ
  panels/lookupPanelState.ts    NEW     ⚠️ NGOÀI component (AC10); ⛔ KHÔNG import vào commands/
  panels/README.md              UPDATE  hàng 1.17 → ✅
  config/dict.ts                UPDATE  adapter IPC thứ hai — khuôn `readHanViet`
  commands/index.ts             UPDATE  một command mới + dep `currentSelection`
  main.ts                       UPDATE  tiêm dep
  modes/libraryImport.ts        UPDATE  gọi `resetLookupPanel()` cùng `resetSourcePanel()`
  panels/SourcePanel.vue        UPDATE  ⚠️ CHỈ khối <style> — ba lớp của token 17 (#7)
  panels/SourceHanViet.vue      UPDATE  ⚠️ CHỈ khối <style> — `.hv-notice` (#7)
  i18n/vi.json                  UPDATE  ≥ 5 khoá `panel.lookup.*`
  tokens/tokens.json            UPDATE  🔴 token thứ 17 `ui-md-wrap` + deviations (#7 ĐÃ CHỐT)
scripts/check-*.mjs             UPDATE  các hằng *_FLOOR + sổ deviations của check-tokens.mjs
```

⚠️ **`src/commands/**` ⛔ không được `import` Vue** — `scripts/check-commands.mjs` nạp thư mục
đó bằng **Node thuần**. Hướng phụ thuộc một chiều: `panels/` → `commands/`.
⚠️ **`owner` và `status-key` viết LITERAL ở chỗ gọi** — Kiểm E của `check:commands` đọc
**tĩnh** hai thuộc tính đó. Một biểu thức bị đếm rồi **bỏ qua** = mất lưới.

### 📌 Bối cảnh git

`cb03974` *(1.10c/1.16 — nguồn mới + test từ điển)* · `564be15` *(1.15 — Library import form)*
· `c3efb20` *(1.14 — bốn panel + `dockview`; cùng lượt mang `core/dict/{layer,mod,senses}.rs`
và `ports/dict_source.rs` của 1.13)* · `7e38de8` *(test hành vi `core::matching`)*.

**Đọc gì trước khi gõ:** `src-tauri/src/core/dict/mod.rs` *(trọn — 861 dòng, doctrine hai pha)*
· `src-tauri/src/ports/dict_source.rs` *(trọn — doctrine cổng)* · `src-tauri/src/commands/dict.rs`
+ `commands/chapter.rs` *(khuôn hàm-thuần-vỏ-mỏng)* · `src/panels/sourcePanelState.ts` *(trọn
— khuôn state module-level, và **năm** lỗi mà code review 1.16 đã vá ở đó)* ·
`src/panels/SourcePanel.vue` · `src/config/dict.ts` *(khuôn adapter IPC)* ·
`mockups/lookup-real-density.html` *(mật độ THẬT — ⛔ đừng dựng theo mock 3-bản-ghi cũ)*.

### 🌐 Phiên bản đang ghim — ⛔ KHÔNG đổi một dòng nào

`tauri 2.11.5` · `tauri-runtime 2.11.3` · `rusqlite` bundled · `dockview-vue` · Rust toolchain
`1.97.1`. **⛔ 0 phụ thuộc mới, cả npm lẫn crate** — mọi phụ thuộc mới phải qua rà NFR15 và
vào bảng Stack **trước**, và `check-deps.mjs` có danh sách cấm cùng ngưỡng sàn.

### References

- `epics.md:1706-1750` — Story 1.17, tám mệnh đề AC · `:1752+` — Story 1.18 *(ranh giới ngay
  dưới)* · `:1807+` 1.19 · `:1845+` 1.20 · `:1530+` 1.13 *(tầng dữ liệu đã giao)*
- `epics.md:132` FR28 · `:134` FR29 · `:136` FR30 · `:138` FR31 · `:140` FR32 · `:144` FR34 ·
  `:146` FR35 · `:818` *(nghiệm thu FR36 bằng test thật)*
- `ARCHITECTURE-SPINE.md:290` AD-19 · `:302` AD-21 · `:328` AD-25 · `:406` AD-34 · `:571`
  AD-44 · `:654` *(bảng Hard Rules — §Tra cứu)*
- `DESIGN.md:269-271` — `lookup-headword` · `lookup-gloss` · `lookup-example` · `:340`
  *(vùng đầu mục cao cố định)* · `:360` *(§Components — **Bản ghi từ điển**, nguyên văn của
  AC1/AC2/AC3)* · §Shapes *(vạch dọc là hình dạng chủ đạo)*
- `EXPERIENCE.md:117` *(⛔ không trạng thái "đang tải")* · `:272` *(cảnh dùng thật — ba nguồn,
  một dòng dẫn báo bất đồng)* · `:389` *(§Trạng thái rỗng — ⚠️ mâu thuẫn #1)*
- `mockups/lookup-real-density.html` — **tham chiếu thị giác chính**; `:68` `.sense` *(vạch
  trái 2px + thụt 13px)* · `:72` `.cite` *(vạch trái `primary`)* · `:65` `.srcname`
  *(`ui-label`/`primary`)* · `:115-122` *(thanh nhịp)* · `:151` *("còn 6 nghĩa nữa")*
- `deferred-work.md:113` · `:129-133` · `:317` · `:343` · `:363` · `:416` · `:419-449` ·
  `:453` · `:504`
- `src-tauri/src/core/dict/mod.rs:52-78` *(§PHẠM VI + §HAI PHA — vì sao `LIMIT` thuộc story
  này)* · `:186-207` *(`NoBranchQueryTooShort` — nguyên văn cho AC6)* · `:394-459` *(ba kiểu
  bản ghi của FR28/FR29/FR30)* · `:480-528` *(`SourceGroup`/`GroupedLookup`)*

---

## Câu hỏi cho Ice

### ✅ Ba câu đã trả lời 2026-08-06

1. ✅ **Mục từ TIẾNG ANH — TẠM DÙNG MẶC ĐỊNH.** `deferred-work.md:317` mở từ 2026-08-05, chủ
   sở hữu là **Sally (`bmad-ux`)**. `EXPERIENCE.md`/`DESIGN.md`/mockup đều dựng quanh mục từ
   **tiếng Trung**; mục tiếng Anh có hình dạng khác *(từ loại + nghĩa tiếng Việt + ví dụ, ⛔
   không Hán Việt, ⛔ không đầu mục chữ Hán 24px)*.
   ⇒ **Dùng:** **cùng một cấu trúc khối**, chỉ khác token đầu mục *(`lookup-headword` họ
   `read`, ⛔ không phải chữ Hán 34px của mockup)*.
   🔴 **Và mục `:317` ⛔ KHÔNG ĐÓNG.** Đây là một lựa chọn **tạm** của tầng story, ⛔ không
   phải một chữ ký UX — đúng thứ mục đó cảnh báo: *"tự chế ở tầng story là **đúng cách một
   bất nhất giao diện ra đời**"*. Task 11 phải ghi lại nguyên trạng đó, ⛔ không tick xong.
2. ✅ **Quyết định #7 = (b)** — thêm **token thứ 17 `ui-md-wrap`**. Xem §Quyết định #7.
3. ✅ **Quyết định #4 = có `LIMIT` pha một.** Xem §Quyết định #4 — và **ba hệ quả** mà lượt
   đối chiếu với `query.rs` thật vừa phát hiện.

### ✅ Câu thứ tư — cũng đã trả lời 2026-08-06

4. ✅ **Nếu `LIMIT` ⛔ không cắt được thời gian trên nhánh `char_idx` ⇒ DỪNG và tìm phương
   án.** Ice chốt **(b)**: ⛔ **không** đi tiếp với một `LIMIT` ⛔ không mua được gì chỉ vì
   nó *"đã được chốt"*.
   ⇒ **Bốn phương án ứng viên đã khảo sát sẵn** ở §Quyết định #4 — **C** *(đẩy `LIMIT` vào
   subquery `char_idx`, nơi `PRIMARY KEY (ch, entry_id) WITHOUT ROWID` cho phép dừng sớm
   thật)* là đề xuất; **B** *(bỏ `JOIN dict_source`)* là đường phụ; **D** *(chỉ mục mới)*
   ngoài phạm vi; **A** *(bỏ `ORDER BY`)* đã loại.
   🔴 **Quy trình bắt buộc:** dừng thi hành → khảo sát trên `EXPLAIN QUERY PLAN` thật → ghi
   số → đề xuất → **hỏi Ice trước khi cài**. ⛔ Dev **không** tự chọn giữa bốn phương án.
   ⚠️ **Và xét điều kiện *"nếu cần"* TRƯỚC:** đầu-cuối < 100 ms ⇒ NFR1 đã đạt ⇒ ⛔ không
   phương án nào phải chạy.

---

## Dev Agent Record

### Agent Model Used

Claude Sonnet 5 (`claude-sonnet-5`), qua skill `bmad-dev-story`.

### Debug Log References

**Task 0 — bench lại `:419` (`bench_the_grouped_path_on_the_real_dictionaries`, `--release`, `AURA_DICT_BENCH_DIR=tools/dict-build/out`), 4 lớp thật (`base` 7 nguồn, `thieu-chuu` 1, `tran-van-chanh` 1, `vietphrase` 1):**

```
── PHA MỘT — `lookup_grouped` trên 4 lớp ──
nhánh              truy vấn   nhóm   hàng      p50       p95       p99
zh-1-btree         山            7     10   0.258ms   0.347ms   0.400ms
zh-2-charidx-1     山            8   6565   9.560ms  12.949ms  16.676ms
zh-2-charidx-2     中國           5    354   2.612ms   3.451ms   4.062ms
zh-3-trigram       中國人         4     35   0.633ms   0.764ms   0.859ms
en-1-btree-lower   running       1      1   0.215ms   0.303ms   0.445ms
en-1-btree-upper   Running       1      1   0.218ms   0.299ms   0.375ms
en-2-trigram       dic           1    572   1.678ms   1.873ms   2.002ms

── PHA HAI — hydrate toàn bộ nhóm xấu nhất ──
zh-2-charidx-1/tất cả (vietphrase, 3.385 đầu mục)  p95 17.066ms  p99 20.281ms
```
Chậm nhất: `zh-2-charidx-1` (nhánh `char_idx` 1 ký tự) — vượt trần 10 ms. Khớp `:419` ban đầu (12,569 ms trên 3 tệp); số đổi nhẹ vì tập lớp thật hôm nay là 4 tệp (`vietphrase` tách lớp riêng từ Story 1.10c/1.16).

**`EXPLAIN QUERY PLAN` nguyên văn (`sqlite3`, không có Rust prepare_cached liên quan tới kế hoạch):**

```
-- dict-vietphrase.db, char_idx 1 ký tự ('山'), KHÔNG LIMIT và CÓ LIMIT 20 — HAI kế hoạch GIỐNG NHAU:
QUERY PLAN
|--SEARCH e USING INTEGER PRIMARY KEY (rowid=?)
|--LIST SUBQUERY 1
|  `--SEARCH char_idx USING PRIMARY KEY (ch=?)
`--SCAN s USING COVERING INDEX sqlite_autoindex_dict_source_1
-- ⛔ KHÔNG có "USE TEMP B-TREE FOR ORDER BY" — driving loop từ char_idx (PK (ch, entry_id)
-- WITHOUT ROWID) đã sắp theo entry_id tăng dần, khớp ORDER BY e.id.

-- dict-vietphrase.db, char_idx 2 ký tự INTERSECT ('中', '國'), KHÔNG LIMIT và CÓ LIMIT 20 — GIỐNG NHAU:
QUERY PLAN
|--SEARCH e USING INTEGER PRIMARY KEY (rowid=?)
|--LIST SUBQUERY 2
|  `--COMPOUND QUERY
|     |--LEFT-MOST SUBQUERY
|     |  `--SEARCH char_idx USING PRIMARY KEY (ch=?)
|     `--INTERSECT USING TEMP B-TREE
|        `--SEARCH char_idx USING PRIMARY KEY (ch=?)
`--SCAN s USING COVERING INDEX sqlite_autoindex_dict_source_1
-- TEMP B-TREE ở đây là CHO PHÉP TÍNH INTERSECT (bắt buộc), ⛔ không phải cho ORDER BY ngoài
-- — kết quả INTERSECT đã ra theo entry_id tăng dần, không cần sort riêng ở outer query.

-- dict-core.db, fts_trigram ('"dic"', lang='en') — CÓ sort riêng:
QUERY PLAN
|--SCAN f VIRTUAL TABLE INDEX 0:M1
|--SEARCH e USING INTEGER PRIMARY KEY (rowid=?)
|--SEARCH s USING INTEGER PRIMARY KEY (rowid=?)
`--USE TEMP B-TREE FOR ORDER BY
-- nhánh này CÓ phải vật liệu hoá + sort trước khi LIMIT cắt được — nhưng nhánh 3 vốn dưới
-- trần (0,6–2,0 ms mọi ca đo), không quan trọng cho NFR1.

-- dict-core.db, exact btree zh ('山') — CÓ sort riêng (MULTI-INDEX OR cần hợp nhất hai scan):
QUERY PLAN
|--MULTI-INDEX OR
|  |--INDEX 1
|  |  `--SEARCH e USING INDEX idx_entry_headword (headword=?)
|  `--INDEX 2
|     `--SEARCH e USING INDEX idx_entry_headword_simp (headword_simp=?)
|--SEARCH s USING INTEGER PRIMARY KEY (rowid=?)
`--USE TEMP B-TREE FOR ORDER BY
-- nhánh 1 luôn rất nhanh (0,2-0,3ms), không quan trọng cho NFR1.
```

**Đo tay `sqlite3 .timer on` (xác nhận kế hoạch có thật sự đổi thời gian không, không chỉ đổi shape):**

| Ca | KHÔNG `LIMIT` | `LIMIT 20` | Tỉ lệ |
|---|---|---|---|
| char_idx 1 ký tự, `山`, vietphrase (3.385 hàng) | 9–12 ms | ~1 ms | **~10×** |
| char_idx 2 ký tự INTERSECT, `一人`, dict-core (75 hàng) | ~13 ms | ~5 ms | **~2,6×** |
| char_idx 2 ký tự INTERSECT, `中國`, vietphrase (18 hàng, dưới N) | ~9 ms | ~9 ms | không đổi *(không đủ hàng để LIMIT cắt gì)* |
| fts_trigram, `"dic"`, dict-core (572 hàng) | 4 ms | 3 ms | không đáng kể — khớp EXPLAIN (có sort riêng) |
| `ROW_NUMBER() OVER (PARTITION BY source_id)`, `山`, vietphrase | — | **15 ms** | **CHẬM HƠN** cả bản không `LIMIT` — hai bước `USE TEMP B-TREE FOR ORDER BY`, không dừng sớm |

**Hệ quả ② — tái lập lỗi "nguồn biến mất" thật trên `dict-core.db` (7 nguồn):** `LIMIT 20` cấp-tệp cho `一` (rất phổ biến) trả **20/20 hàng đều thuộc `cvdict`**; bốn nguồn khác — `en-wiktionary` (1.590 hàng), `cc-cedict` (1.186), `viwiktionary` (11), `unihan` (1) — biến mất hoàn toàn khỏi 20 hàng đầu theo `e.id`. Xác nhận đúng lớp lỗi AC12 mô tả.

**Task 8 — bench release đầy đủ LẦN THỨ HAI (`bench_the_grouped_path_on_the_real_dictionaries`, mở rộng thêm khối "SAU LIMIT"), trên cùng bốn lớp thật:**

```
── PHA MỘT (KHÔNG LIMIT) — nhánh xấu nhất giữ nguyên ──
zh-2-charidx-1  山   8 nhóm  6565 hàng   p50 10.223ms  p95 20.836ms  p99 27.114ms
  (pha hai hydrate TOÀN BỘ vietphrase, 3385 đầu mục: p95 18.524ms — cộng dồn ~39ms/lượt)

── SAU LIMIT (Quyết định #4) — commands::dict::lookup(), LOOKUP_PAGE_LIMIT = 20 ──
nhánh         truy vấn   nhóm  hàng    p50      p95       p99
exact_btree   山            7    10   2.423ms   6.535ms   70.742ms  ← ca xấu nhất đường sản phẩm
exact_btree   中國           5     5   0.765ms   0.984ms    1.230ms
exact_btree   打            7    11   1.546ms   2.103ms    2.485ms
exact_btree   running       1     1   0.563ms   0.803ms    1.203ms
exact_btree   Running       1     1   0.646ms   0.980ms    1.120ms
char_idx-1(sub, LIMIT=20, chỉ pha một)  山   —   2.193ms   5.109ms   26.394ms

Chậm nhất SAU LIMIT: lookup("山") — p95 6.535 ms (trần đầu-cuối NFR1: 100ms) ⇒ NFR1 ĐẠT
JSON LookupResponse lớn nhất đo được: 35.084 byte
```

🔴 **ĐO LẠI 2026-08-07 (code review, Ice chốt "đo lại trước khi đóng") — p99 70,742 ms ⛔ TÁI LẬP.**

Lượt review bắt đúng một chỗ: bảng trên ghi `p99 70.742ms` kèm chữ *"ca xấu nhất đường sản
phẩm"*, nhưng kết luận *"NFR1 ĐẠT"* chỉ dùng p95. p99 backend 70,7 ms + render p50 ~31 ms
≈ **101 ms** — chạm/vượt trần NFR1 100 ms. Ice chốt: ⛔ đóng story trên một con số chưa
hiểu (*"nghi ⛔ phải là biết"*).

**Đo lại ba lượt độc lập**, cùng máy, cùng bốn tệp `.db` thật, cùng `--release`, cùng
`WARMUP = 10` / `RUNS = 200`:

| Lượt | `exact_btree 山` p50 | p95 | **p99** | `exact_btree 打` p99 |
|---|---|---|---|---|
| Gốc (2026-08-06) | 2,423ms | 6,535ms | **70,742ms** | 2,485ms |
| Lại #1 | 0,293ms | 0,468ms | **0,566ms** | 3,788ms |
| Lại #2 | 0,920ms | 1,048ms | **1,136ms** | 4,039ms |
| Lại #3 | 1,335ms | 1,580ms | **1,793ms** | 2,695ms |

⇒ **70,742 ms là NHIỄU của lượt đo gốc, ⛔ một thuộc tính của mã.** p99 đường sản phẩm ổn
định ở **0,57–4,04 ms** qua ba lượt — cách trần 100 ms hai bậc độ lớn, và ⛔ còn khoảng
cách p95↔p99 bất thường nào (gốc: gấp **10,8×**; đo lại: gấp **1,1–2,1×**, đúng hình dạng
mong đợi). Nguyên nhân khả dĩ nhất của lượt gốc: page cache lạnh / lượt đo đầu tiên trên
tệp `.db` chưa được OS ánh xạ — đúng thứ `WARMUP` tồn tại để loại mà 10 vòng ⛔ đủ cho một
tệp 592.538 đầu mục đọc lần đầu.

**⇒ NFR1 ĐẠT, và lần này kết luận đứng trên p99 chứ ⛔ chỉ p95.** Ước tính đầu-cuối xấu
nhất: 4,04 ms (backend p99) + ~31 ms (render, Playwright/Chromium thật) ≈ **35 ms**, trần
100 ms.

⚠️ **Giới hạn phép đo ⛔ đổi:** vòng IPC Tauri thật (WKWebView/WebView2) vẫn **chưa** được
đo — xem §giới hạn bên dưới. Ba lượt trên chỉ khoá lại vế **backend**.

⚠️ **Một ngoại lai KHÁC vẫn còn, và nó ⛔ nằm trên đường sản phẩm:** `en-2-trigram/tất cả`
(pha hai hydrate **toàn bộ** 572 đầu mục) đo được p99 **57,771 ms** ở lượt #1. Nhánh này
⛔ được `commands::dict::lookup()` chạm tới (nó hydrate **một trang**, ⛔ toàn bộ — đúng
Bẫy 4), nên nó ⛔ ảnh hưởng NFR1 hôm nay. Ghi ra vì Story 1.18/7.7 sẽ mở đường `Substring`
và lúc đó con số này thành thật.

🔴 **PHÁT HIỆN QUAN TRỌNG — nhánh `char_idx` (nhánh "đắt nhất" mà cả Quyết định #4 xoay quanh) KHÔNG được đường sản phẩm thật của Story 1.17 chạm tới.** `commands::dict::lookup()` cố định `LookupMode::Exact` (Quyết định #3), và `pick_branch` cho `Exact` **luôn luôn** trả `ExactBtree` bất kể độ dài truy vấn — `char_idx`/`fts_trigram` chỉ được chọn khi `mode = Substring`, thứ **không tồn tại** trong bất kỳ lời gọi nào của 1.17. ⇒ Trần NFR1 mà Panel Lookup **tự nó** phải đạt hôm nay được quyết định hoàn toàn bởi nhánh `ExactBtree` (luôn rẻ, p95 đo được 6,535 ms — dưới xa trần 100 ms) — **không phải** bởi nhánh `char_idx` đắt đỏ mà Task 0/Quyết định #4 dành phần lớn công sức để xử lý.

⚠️ **Điều này KHÔNG làm Quyết định #4/`LIMIT` trở thành công sức thừa** — ba lý do:
1. `LIMIT` (đường (b), hệ quả ②) vẫn là hàng rào BẮT BUỘC cho FR31 ngay trên nhánh `ExactBtree`: một headword khớp hàng nghìn đầu mục trong một tệp nhiều nguồn (như `一` trên `dict-core.db`) vẫn cần trần + cờ `truncated` — AC12 vẫn áp dụng nguyên vẹn cho `ExactBtree`, đã đo và test (Task 1b).
2. `char_idx`/`fts_trigram` sẽ được CHẠM tới ngay khi Story 1.18 (Auto-Lookup) hoặc 7.7 (Concordance) gọi `LookupMode::Substring` — hạ tầng `LIMIT` đã sẵn sàng, đã đo, đã test cho ngày đó.
3. Số đo char_idx "sau LIMIT" (`LIMIT=20`, gọi trực tiếp `lookup_grouped` vì đường Substring ⛔ qua command hôm nay) vẫn cho kết quả tốt: p95 **5,109 ms** so với **20,836 ms** không `LIMIT` — cơ chế hoạt động đúng như thiết kế, sẵn sàng cho chỗ gọi tương lai.

**Cỡ trang `N` CHỐT: 20** — giữ nguyên giá trị đã dùng xuyên suốt Task 1b/2/8, đúng thứ tự "chốt sau Task 8" mà story yêu cầu. Ghi vào doc-comment `LOOKUP_PAGE_LIMIT` (`commands/dict.rs`).

**Đo webview (Playwright headless Chromium, mock `window.__TAURI_INTERNALS__.invoke` trả lời tức thời):** thời gian từ dispatch phím `Mod+Alt+L` tới khi `.lookup-source-name` xuất hiện trong DOM — 15 lượt, p50 ≈ 31 ms, khoảng 11,6–111,2 ms (nhiễu cao, đặc trưng của môi trường headless test + chi phí dispatch sự kiện của Playwright, ⛔ không đại diện chính xác một cú gõ phím thật). Đây là ước tính CHO PHẦN RENDER, ⛔ phải số đo "đầu-cuối" hoàn chỉnh.

⚠️ **GIỚI HẠN CỦA PHÉP ĐO — nói thẳng, không giấu (AC11):**
- Backend (Rust): đo trên **dữ liệu thật** (`tools/dict-build/out/*.db`, `--release`), qua **đúng hàm sản phẩm** `commands::dict::lookup()` — số liệu đáng tin.
- Webview render: đo qua **Chromium thật** (Playwright) nhưng với IPC **giả lập tức thời** (`invoke` mock trả lời ngay, ⛔ có độ trễ round-trip thật của Tauri) và trong môi trường headless CI-like — số liệu chỉ mang tính ƯỚC LƯỢNG cho phần render, ⛔ phải phép đo webview thật trên bản build Tauri đã đóng gói.
- ⛔ **Không có** một bản Tauri đã biên dịch + WKWebView/WebView2 thật đang chạy trong phiên này để đo vòng IPC THẬT (serialize Rust → deserialize JS → Vue reactivity → paint) — đây là món nợ cũ (thị giác hai nền tảng thật), Task 11 ghi lại nguyên trạng, ⛔ đánh dấu đạt cho vế đó.
- **Kết luận có cơ sở nhưng KHÔNG phải một phép đo đầu-cuối hoàn chỉnh 100%:** backend p95 6,535 ms + JSON 35 KB (serialize nhanh, ⛔ đo riêng được vì `to_string` trong Rust chạy < 1ms cho kích cỡ này) + render ước lượng ~30 ms ⇒ tổng ước tính **dưới 40 ms**, cách xa trần 100 ms của NFR1 với biên độ lớn — nhưng vòng IPC Tauri thật (serialize/deserialize qua cầu JS) chưa được đo trực tiếp.

### Completion Notes List

**Tổng kết bàn giao (Task 11).** Story hoàn thành cả mười hai task (0, 1b, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11 — mười ba nhãn nhưng Task 5/6/7 và Task 2/3 thực thi gộp lượt do chung tệp). Cả 80 checkbox Tasks/Subtasks đã `[x]`. Chín cổng DoD xanh (`cargo test`, `npm run build`, `check:tokens`, `check:i18n`, `check:commands`, `check:layout`, `check:deps`, `check:dict-manifest`, `check:scope`) — xác nhận lần cuối cùng lượt với toàn bộ thay đổi. Kiểm bằng browser thật (Playwright, vì Chrome extension không kết nối được trong phiên) xác nhận bốn trạng thái AC6 + bản ghi có cấu trúc (POS/gloss/example/citation/note/nhãn ngoại ngữ) hiển thị đúng.

**Việc KHÔNG được đánh dấu đạt (trung thực theo AC11/Task 11):**
- Vòng IPC Tauri thật (WKWebView/WebView2, `tauri dev`/bản đóng gói) — chưa đo. Số NFR1 dựa trên Rust thật (đáng tin) + webview render qua Chromium với IPC giả lập (ước tính).
- `QueryBranch::query_too_short` — bề mặt UI đúng nhưng không thể thực thi qua đường sản phẩm Exact-only hôm nay; chủ Story 1.18/7.7.
- Mục từ tiếng Anh — hình dạng tạm, chữ ký UX chính thức (Sally) vẫn thiếu.
- Nghiệm thu hai nền tảng thật (macOS WKWebView / Windows WebView2) cho toàn bộ bề mặt mới — món nợ cũ kế thừa từ 1.6/1.14/1.16, không phải món nợ 1.17 đóng.

**Quyết định kiến trúc đáng chú ý nhất của story:** đo thật (Task 0 + Task 8) lật ngược giả định ban đầu — nhánh `char_idx` (trọng tâm ban đầu của Quyết định #4) hoá ra không được đường sản phẩm Exact-only của chính 1.17 chạm tới; `ExactBtree` (luôn nhanh) mới là nhánh quyết định NFR1 hôm nay. `LIMIT` vẫn đúng đắn và cần thiết (FR31/AC12 trên `ExactBtree`, sẵn sàng cho `Substring` ở 1.18/7.7) nhưng lý do tồn tại của nó đã dịch chuyển so với lúc story được dựng — ghi thẳng thay vì im lặng, đúng nguyên tắc "đo trước khi chốt kiến trúc" của dự án.

- Task 0 hoàn thành — bảy quyết định đã chốt kèm số đo, xem Change Log 2026-08-06 (dòng đầu). Tóm tắt: #1=(a), #2=(a), #3=(a) Exact, #4=đã chốt (LIMIT pha một) + hệ quả ①=`LIMIT` đơn giản đã đủ (không cần Phương án C, không cần hỏi Ice) + hệ quả ②=đổi sang đường (b) `LIMIT` cấp-tệp + cờ `truncated` (đường (a) `ROW_NUMBER` bị loại vì đo chậm hơn không `LIMIT`; **Ice xác nhận (b)** khi hỏi lại vì chữ AC12 "kỳ vọng nguồn thứ hai vẫn có mặt" mâu thuẫn với Given/Then cho phép đường (b)) + hệ quả ③=đường (a) `COUNT` riêng khi `truncated`, #5=(a) hiện liền nhau, #6=(a), #7=đã chốt (token `ui-md-wrap`). Mâu thuẫn tài liệu #4 chốt theo mockup (vạch ở cấp NGHĨA).
- Task 1 hoàn thành — hình dạng bản ghi trên dây. `SourceInfo`/`EntryHit`/`SenseRecord`/`ExampleRecord`/`CitationRecord`/`SourceGroup`/`GroupedLookup` derive `serde::Serialize` thẳng, không `rename_all`. `QueryBranch`/`QueryRoute` dùng `#[serde(rename = …)]` per-variant (không phải blanket rename_all) ra chuỗi máy (`"exact_btree"`, `"char_idx"`, `"fts_trigram"`, `"query_too_short"`, `"zh"`, `"en"`). `SkipReason`/`SkippedLayer` VẪN không derive `Serialize` (đúng chốt #2.2) — thêm `SkipReason::wire_code()` (mười mã máy) và một hàm `serialize_skipped_as_wire_codes` gắn qua `#[serde(serialize_with = …)]` lên `GroupedLookup::skipped`, biến `Vec<SkippedLayer>` (kiểu Rust không đổi) thành mảng chuỗi mã máy trên dây — số lượng đọc qua độ dài mảng. Test `skip_reason_detail_never_reaches_the_wire` dùng `lookup_grouped` thật trên tệp `garbage.db` hỏng thật (không dựng `GroupedLookup` tay, đúng nguyên tắc `ipc_contract.rs`), xác nhận lỗi thô SQLite và đường dẫn tệp không lộ ra JSON, đồng thời xác nhận `branch`/`route` ra chuỗi máy. `cargo test` xanh (41 dict_sources, +1).
- Task 1b hoàn thành — `LIMIT` pha một. Đổi chữ ký `DictionarySource::lookup`/`DictLayer::lookup`/`lookup`/`lookup_with_branch`/`lookup_grouped` thêm tham số `limit: usize`. Sáu hình dạng SQL của `query.rs`: `exact`/`exact_en`/`char_idx` 1-ký-tự dùng `LIMIT ?N` ở SQL (đo cắt được thời gian thật — xem §Debug Log); `char_idx` 2-ký-tự/`fts_trigram`/`fts_trigram_en` fetch không giới hạn rồi cắt ở Rust SAU `verify_substring` (tránh Bẫy 11 bằng kiến trúc, không phải bằng kỷ luật). Thêm `LookupResult::truncated: bool` + `GroupedLookup::truncated_layers: Vec<String>` cho hệ quả ②. Ba ca test mới ở `dict_sources.rs`, cả ba đã kiểm chứng đỏ-rồi-xanh bằng cách tạm mô phỏng cài đặt ngây thơ rồi khôi phục: `a_file_level_limit_flags_truncation_instead_of_silently_dropping_a_source` (nhánh `ExactBtree`), `a_file_level_limit_on_the_char_idx_branch_also_flags_truncation` (nhánh `CharIdx` 1 ký tự — nhánh đắt nhất), `the_limit_is_applied_after_verification_not_before` (Bẫy 11, nhánh `CharIdx` 2 ký tự với dương tính giả xen giữa mục thật). Cập nhật 19 lời gọi `lookup()` ở `dict_lookup.rs` + 20 lời gọi `lookup_grouped()` ở `dict_sources.rs` (hằng `UNLIMITED = 10_000` giữ nguyên hành vi cũ cho mọi ca không nhắm tới truncation). `dict_boundary.rs` không cần đổi (đếm mã nguồn tĩnh). Cỡ trang `N` thật (dùng ở `commands/dict.rs`, Task 2) vẫn chưa chốt — chờ Task 8. Toàn bộ `cargo test` xanh (155+3 = một số cụ thể ghi ở Task 10).

- Task 10 hoàn thành — cổng và sàn. Nâng sáu hằng `*_FLOOR` bị tụt so với số thật (đặc biệt `check-tokens.mjs` FILE/COMPONENT_FLOOR đã tụt xuống ~62-65% sau ba story liên tiếp không ai nâng — đúng lỗi Story 1.16 để lọt mà story này KHÔNG được lặp lại). Kiểm chứng trực tiếp cơ chế canh gác bằng cách đặt `COMMAND_FLOOR` cao hơn số thật một lượt (18 > 17 thật) → cổng đỏ đúng thông báo, khôi phục lại 14 → xanh. Toàn bộ 9 cổng DoD + `cargo test` xanh sau khi nâng sàn.
- Task 8 hoàn thành — đo NFR1 đầu-cuối trước/sau `LIMIT`. Mở rộng bench thật (`--release`, bốn lớp thật) thêm khối "SAU LIMIT" gọi đúng `commands::dict::lookup()`. Phát hiện quan trọng: nhánh `char_idx` (trọng tâm của Quyết định #4) **không** được đường sản phẩm thật của 1.17 chạm tới vì `Exact` luôn đi `ExactBtree` — trần NFR1 mà 1.17 tự nó phải đạt do `ExactBtree` quyết định (p95 6,535ms, xa trần 100ms). `LIMIT` vẫn giữ giá trị: (1) AC12/FR31 vẫn áp cho `ExactBtree` khi một headword khớp nhiều đầu mục trong tệp nhiều nguồn; (2) hạ tầng sẵn sàng cho `Substring` khi 1.18/7.7 cần. Cỡ trang N chốt = 20. Đo webview qua Playwright thật (Chromium thật, IPC giả lập tức thời) — p50 ≈ 31ms render. Giới hạn phép đo ghi thẳng: chưa có bản Tauri đóng gói để đo vòng IPC thật. Ước tính đầu-cuối tổng < 40ms ⇒ NFR1 ĐẠT.
- Task 5 + Task 6 + Task 7 hoàn thành cùng lượt (thực thi tự nhiên trộn vào nhau — ba task đều nằm trong `LookupPanel.vue`/`LookupRecord.vue`). Dựng `LookupRecord.vue` (khối một nguồn: nhãn `ui-label`, danh sách nghĩa PHẲNG — POS/gloss/examples/citations/note, nhãn ngoại ngữ đánh dấu bằng `.lookup-foreign-flag` màu `primary`) + `LookupPanel.vue` (vùng đầu mục cố định `--lookup-head-height: 76px` + `overflow: hidden`, thanh nhịp, hai banner `someLayerFailed`/`someLayerTruncated`, dòng dẫn bất đồng, bốn nhánh trạng thái rỗng loại trừ lẫn nhau). Token thứ 17 `ui-md-wrap` (12px/1.66/`wraps:true`) thêm vào `tokens.json` + `deviations` + `EXPECTED_COUNTS.typography` (16→17) của `check-tokens.mjs`, áp cho `.load-error`/`.parallel-note` (`SourcePanel.vue`) + `.hv-notice` (`SourceHanViet.vue`, chỉ khối `<style>`) + mọi bề mặt chữ mới của `LookupPanel.vue`/`LookupRecord.vue` — đóng `deferred-work.md:115` sau bốn lần bị gọi tên. `.parallel-note` đổi cỡ 11,5px→12px (chấp nhận, ghi ra). **Kiểm bằng browser thật** (Playwright headless Chromium, vì extension Chrome không kết nối được trong phiên này): dev server thật (`npm run dev`) + mock `window.__TAURI_INTERNALS__.invoke('lookup_dictionary')` trả dữ liệu dàn dựng (2 nguồn bất đồng, 1 skipped, 1 truncated, POS có `pos_lang`, ví dụ có bản dịch, trích dẫn có work/author, ghi chú) — kích hoạt qua đúng phím `Mod+Alt+L` thật (không gọi hàm nội bộ). Xác nhận bằng mắt: (1) trạng thái "chưa tra gì" hiện đúng status mặc định của `PanelFrame`; (2) trạng thái có kết quả hiện đúng headword, thanh nhịp "2 · 3 CVDICT 2 THIỀU CHỬU 1", hai banner, dòng dẫn bất đồng (màu `tm-text`/`tm-rule`), khối CVDICT (2 nghĩa, ví dụ nghiêng), khối THIỀU CHỬU (nhãn "noun EN" đánh dấu rõ, trích dẫn vạch trái `primary`, ghi chú) — chữ Hán render đúng, KHÔNG lỗi console ngoài các lỗi IPC-ngoài-Tauri đã biết; (3) trạng thái "không tìm thấy" và (4) trạng thái "0 lớp" hiện hai câu KHÁC NHAU, phân biệt được. Ảnh chụp màn hình lưu tạm ở `/tmp/{1..5}_*.png` (không commit — chỉ dùng để nghiệm thu phiên này).
- Task 4 hoàn thành — `lookupPanelState.ts` (module-level, khuôn `sourcePanelState.ts`) + đường kích hoạt. Năm vị từ trạng thái riêng: `neverLookedUp`/`notFound`/`queryTooShort`/`layersLoaded` (bốn nhánh loại trừ lẫn nhau, AC6) + `someLayerFailed`/`someLayerTruncated` (hai banner không loại trừ, đi cùng kết quả). `runLookup(query)` một đường vào, `resetLookupPanel()` cùng điểm nghẽn `resetSourcePanel()` ở `libraryImport.ts::finishSubmit`. Thêm `commands::dict::lookupDictionary` adapter IPC (`config/dict.ts`, khuôn `readHanViet`) + toàn bộ kiểu wire TS khớp Rust (`QueryRoute`/`QueryBranch`/`SourceInfo`/`EntryHit`/.../`GroupedLookup`/`LookupResponse`). Đăng ký command `lookup.lookup_selection` (phím `Mod+Alt+L`) ở `commands/index.ts` + hai dep mới `runLookup`/`currentSelection` tiêm ở `main.ts` (`currentSelection` dùng `window.getSelection()` trực tiếp — dep TỐI THIỂU theo đúng Quyết định #1a, Story 1.18 sẽ thay). `window.getSelection` phải thêm vào `ALLOWED_GLOBAL_MEMBERS` của `scripts/check-layout.mjs` Kiểm C (bị chặn ban đầu, sửa kèm dòng lý do AC — đỏ-rồi-xanh tự nhiên qua chính cổng DoD). Cả 9 cổng DoD xanh + `cargo test` xanh.
- 🔴 **Phát hiện khi bắt đầu Task 4**: `GroupedLookup` (Task 1) THIẾU đường phân biệt ca "0 lớp gắn" (AD-25) với ca "đã tra mà ⛔ không khớp" — cả hai đều cho `groups: []` VÀ `skipped: []` giống hệt nhau khi thư mục từ điển rỗng. AC6 đòi năm trạng thái phân biệt được. Sửa: thêm `GroupedLookup::layers_loaded: bool` (`!layers.layers().is_empty()`), đúng doctrine `HanVietLookup::layers_loaded` của Story 1.16 — quên áp lại cho `GroupedLookup` là chính lỗi mà "trí tuệ từ story trước" của story này cảnh báo (§Testing standards). Hai ca test mới xác nhận: `layers_loaded=false` khi 0 lớp, `layers_loaded=true` khi có lớp mà ⛔ khớp. `cargo test` xanh (48 dict_sources, +1; test cũ `a_missing_or_empty_directory_is_an_empty_layer_set_not_an_error` được bổ sung một assertion).
- Task 2 + Task 3 hoàn thành cùng lượt (thực thi tự nhiên trộn vào nhau — pha hai "đi đúng lớp" là một phần của việc viết hàm thuần `lookup`). `commands/dict.rs` thêm `pub fn lookup(layers: Option<&DictLayers>, query: &str) -> LookupResponse`: `LookupMode::Exact` cố định (Quyết định #3, ⛔ tham số `mode` trên chữ ký — quyết định sản phẩm, ⛔ chỗ gọi tự chọn), cỡ trang `LOOKUP_PAGE_LIMIT = 20` (tạm, chờ Task 8) sống Ở ĐÂY (không phải `core/dict/**`, đúng doctrine "LIMIT là chính sách sản phẩm" của `ports/dict_source.rs`), sàn `QUERY_LENGTH_CEILING = 200` đóng `deferred-work.md:363`. `LookupResponse { grouped: GroupedLookup, senses_by_layer: BTreeMap<String, Vec<SenseRecord>> }` — khoá theo LỚP (⛔ `entry_id` phẳng, tránh trộn nghĩa xuyên lớp). Vỏ `lookup_dictionary` dùng `try_state`, đăng ký vào `generate_handler!` (lib.rs) → 7 command. Sáu ca test mới ở `dict_sources.rs`: ca "0 lớp", ca hydrate đúng 3 lớp cùng khớp `山` (AD-19), ca chỉ hydrate đúng tập đã khớp (⛔ hydrate cả lớp), **ca Bẫy 3 bắt buộc** (`base`/`hv-fixture` cùng `entry_id=1` cho `山`, nghĩa khác nhau hoàn toàn — chứng minh ⛔ trộn), ca `senses(&[])` gián tiếp qua `senses_by_layer` rỗng, ca sàn độ dài truy vấn (201 ký tự, ký tự Hán ở vị trí 201 bị cắt mất ⇒ `route = En` chứ ⛔ `Zh` — chứng minh cắt xảy ra TRƯỚC `pick_route`). `cargo test` xanh (47 dict_sources, +6).

### File List

- `src-tauri/src/core/dict/mod.rs` — UPDATE: `LookupResult::truncated`, `GroupedLookup::truncated_layers`, chữ ký `lookup`/`lookup_with_branch`/`lookup_grouped` thêm `limit`; `Serialize` cho `EntryHit`/`SourceInfo`/`SenseRecord`/`ExampleRecord`/`CitationRecord`/`SourceGroup`/`GroupedLookup`/`QueryRoute`/`QueryBranch`; hàm `serialize_skipped_as_wire_codes`
- `src-tauri/src/core/dict/query.rs` — UPDATE: sáu hàm nhánh nhận `limit`, hàm `cap()` mới, `LIMIT ?N` ở ba nhánh không-verify
- `src-tauri/src/core/dict/layer.rs` — UPDATE: `DictLayer::lookup` truyền `limit` xuống `lookup_with_branch`; `SkipReason::wire_code()` mới
- `src-tauri/src/ports/dict_source.rs` — UPDATE: `DictionarySource::lookup` thêm tham số `limit`
- `src-tauri/tests/dict_lookup.rs` — UPDATE: hằng `UNLIMITED`, 19 lời gọi `lookup()` thêm tham số
- `src-tauri/tests/dict_sources.rs` — UPDATE: hằng `UNLIMITED`, 20 lời gọi `lookup_grouped()` thêm tham số, mười ca test mới (AC12 ×2, Bẫy 11 ×1, wire-shape ×1, `commands::dict::lookup` ×6) + fixture `LIMIT_ENTRIES`/`LIMIT_LAYER`/`TRAP11_ENTRIES`/`TRAP11_LAYER` + mở rộng bench khối "SAU LIMIT" (Task 8)
- `src-tauri/src/commands/dict.rs` — UPDATE: hàm thuần `lookup()` + `LookupResponse` + hằng `LOOKUP_PAGE_LIMIT`/`QUERY_LENGTH_CEILING` + vỏ `wire::lookup_dictionary`
- `src-tauri/src/lib.rs` — UPDATE: đăng ký `lookup_dictionary` vào `generate_handler!`
- `src/panels/lookupPanelState.ts` — NEW: state module-level + năm vị từ trạng thái + `runLookup`/`resetLookupPanel`
- `src/config/dict.ts` — UPDATE: kiểu wire `QueryRoute`/`QueryBranch`/`SourceInfo`/`EntryHit`/`ExampleRecord`/`CitationRecord`/`SenseRecord`/`SourceGroup`/`GroupedLookup`/`LookupResponse` + adapter `lookupDictionary`
- `src/commands/index.ts` — UPDATE: `CommandDeps.runLookup`/`currentSelection` + đăng ký command `lookup.lookup_selection`
- `src/main.ts` — UPDATE: import `runLookup`, tiêm `runLookup`/`currentSelection` vào `installCommands`
- `src/modes/libraryImport.ts` — UPDATE: `finishSubmit` gọi `resetLookupPanel()` cùng `resetSourcePanel()`
- `scripts/check-layout.mjs` — UPDATE: thêm `window.getSelection` vào `ALLOWED_GLOBAL_MEMBERS` kèm lý do AC
- `src/i18n/vi.json` — UPDATE: thêm khoá `command.lookup.lookup_selection` + bảy khoá `panel.lookup.*`
- `src/panels/LookupPanel.vue` — UPDATE: nội dung thật (đầu mục, thanh nhịp, banner, bốn trạng thái rỗng, danh sách `LookupRecord`)
- `src/panels/LookupRecord.vue` — NEW: khối một nguồn (nhãn, nghĩa, ví dụ, trích dẫn, ghi chú, nhãn ngoại ngữ)
- `src/tokens/tokens.json` — UPDATE: token thứ 17 `ui-md-wrap` + deviations
- `scripts/check-tokens.mjs` — UPDATE: `EXPECTED_COUNTS.typography` 16→17, ghi chú số deviation, `FILE_FLOOR` 26→32, `COMPONENT_FILE_FLOOR` 23→30
- `scripts/check-commands.mjs` — UPDATE: `VUE_FLOOR` 10→11, `TS_FLOOR` 19→20, `COMMAND_FLOOR` 13→14, `DISPATCH_FLOOR` 6→10
- `scripts/check-i18n.mjs` — UPDATE: `VUE_FLOOR` 10→11
- `scripts/check-layout.mjs` — UPDATE: `FILE_FLOOR` 28→30 (+ `window.getSelection` vào `ALLOWED_GLOBAL_MEMBERS`, đã ghi ở Task 4)
- `src/panels/README.md` — UPDATE: hàng 1.17 → ✅, đoạn "Chữ trong thân panel" cho Panel Lookup
- `_bmad-output/implementation-artifacts/deferred-work.md` — UPDATE: đóng mười một mục gọi tên Story 1.17 (`:453`/`:129`/`:131`/`:133`/`:115` cuối/`:343`/`:363`/`:416`/`:419`/`:449`/`:504`), ghi `:317` là tạm/⛔ đóng, thêm mục mới "Deferred from: 1-17-panel-lookup-ban-ghi-co-cau-truc" (năm phát hiện)
