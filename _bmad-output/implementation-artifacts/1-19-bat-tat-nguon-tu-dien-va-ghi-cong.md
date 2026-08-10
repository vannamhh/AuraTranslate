---
baseline_commit: 51132cb
---

# Story 1.19: Bật tắt nguồn từ điển và ghi công

Status: done

> 🔴 **CÂY LÀM VIỆC KHÔNG SẠCH LÚC TẠO STORY — bảy tệp của Story 1.18b chưa commit.**
> `git status` 2026-08-08: 5 tệp `M` + 2 tệp `??`, toàn bộ là bàn giao của **1.18b** *(đã
> `done` trong `sprint-status.yaml`)*. Trong đó `src/panels/sourcePanelState.ts` và
> `src/panels/README.md` là **hai tệp story này sẽ mổ**. ⇒ **Task 0 phải hỏi Ice** trước
> dòng mã đầu tiên; xem §Bối cảnh git. `baseline_commit` giữ nguyên `51132cb` theo luật
> workflow; baseline **thật** ghi vào Change Log sau khi Ice chốt.
>
> 🔴 **STORY NÀY CHIA MỘT MÀN HÌNH VỚI STORY 10.4, VÀ `epics.md` KHÔNG PHÂN RANH.** Ba
> trong sáu AC của 1.19 nói về **màn hình Attribution**, mà FR109/Story 10.4 cũng đặc tả
> **cùng màn hình đó** bằng bảy AC *(`epics.md:6174-6216`)*. Không story nào nói cái nào
> dựng trước. ⇒ **Quyết định #4 chia ranh giới đó, và Ice đã CHỐT (a) ngày 2026-08-08** — bảng
chia đôi ở §Quyết định #4 là **hợp đồng ràng buộc cả hai story**.
>
> 🔴 **MOCKUP `sources-attribution.html` ĐÃ LỖI THỜI SO VỚI DỮ LIỆU THẬT — đo được, không
> suy đoán.** Bảng Attribution trong mockup liệt **HVTĐTD** *(Ice chốt 2026-08-08: **không
> tìm được nguồn**)* và **Cổ hán văn** *(chưa dựng — `epics.md:336` NFR6)*, và **không** liệt
> **Trần Văn Chánh** *(đã dựng, `license_kind = "copyrighted"`, rủi ro pháp lý ghi thẳng
> trong `attribution`)*. ⇒ màn hình phải **dẫn xuất từ tệp có mặt**, và bảng của mockup là
> **bố cục**, không phải **dữ liệu**. Xem §Số đo mở màn.
>
> 🔴 **HVTĐTD KHÔNG CÒN LÀ MỘT NGUỒN ĐANG CHỜ DỰNG — Ice chốt 2026-08-08: KHÔNG TÌM ĐƯỢC
> NGUỒN DỮ LIỆU.** Hệ quả trực tiếp: AC gốc của epic *("Given lớp HVTĐTD… Then ghi rõ ©
> Đặng Thế Kiệt")* **không có dữ liệu nào để nghiệm thu, hôm nay lẫn về sau**. Story này
> giao **CHỖ GIỮ**: cơ chế phải biểu diễn được một giấy phép *"phép riêng tác giả cấp"* mà
> **không** ai phải sửa mã khi một nguồn loại đó xuất hiện — dù nguồn đó là HVTĐTD hay một
> nguồn **khác thay chỗ nó**. Xem **AC9 đã viết lại**.

## Change Log

| Ngày | Thay đổi |
|---|---|
| 2026-08-10 | **Code review xong — Status `review` → `done`.** Ba lớp chạy độc lập; **11** phát hiện đã vá *(4 `decision` Ice chốt + 7 `patch`)*, **2** `defer`, **0** loại là nhiễu. **Ba món `high`:** ① *lượt tra lại sau toggle đọc tập bị tắt CŨ* — `runLookup`/`refreshHanViet` phát **trước** `putConfig`, mà `commands/dict.rs` mở đầu cả hai command bằng `disabled_sources(store)` và trên dây **không có** tham số `disabled` nào ⇒ AC2 *"hiệu lực NGAY"* vỡ ở **cả hai** bề mặt; ② *cổng canh AC1 trên `src-tauri/src/**` thủng* — `SOURCE_CODES[9]` lỗi thời **thiếu `tran-van-chanh` và `en-wiktionary-vi`**, thừa mã ma `hvtdtd`; nay chỉ còn **MỘT** danh sách `REAL_SOURCE_CODES[10]` cho cả ba cổng, chứng minh bằng phép **cấy lỗi** *(trước: 14/14 xanh; sau: đỏ đúng `lib.rs`)*; ③ *lớp phủ khai `aria-modal` mà không bẫy tiêu điểm* — Tab rời hộp thoại vào nền bị che, và khi tiêu điểm ra ngoài thì `Escape` không còn tới được `@keydown.esc` trên `.attr-scrim`. **Bốn quyết định Ice chốt:** **D1** ghi `lang` vào `dict_source` *(số đo lật lựa chọn đầu: dẫn xuất lúc đọc tốn **~480 ms** mỗi lượt khởi động vì `dict_entry` không có index trên `source_id`)* ⇒ `SCHEMA_VERSION` **2→3**, giá đọc còn **~16 ms**, và **AC6 nay hỏi đúng theo ĐƯỜNG ĐANG TRA** — món nợ `viwiktionary-en` **đã đóng**, không còn là giới hạn; **D2** tab Hán Việt có câu riêng cho *"mọi nguồn đều tắt"*; **D3** bảng Attribution hiện **lỗi tải** thay vì nói sai *"chưa gắn lớp nào"* *(`dictSourcesError` trước đó là kênh chết, 0 nơi tiêu thụ)*; **D4** command toàn cục bị **nuốt** khi lớp phủ mở, qua `KeymapGate` tiêm từ `main.ts`. **Bốn món `medium` khác:** `aimedCode` nay đứng **trước** `focusedChipCode()` *(WKWebView không dời tiêu điểm khi bấm chuột ⇒ bản cũ toggle **sai nguồn**)* · `refreshHanViet` có số thứ tự lượt · lượt ghi `put_config` nối tiếp qua `writeQueue` · lượt lưu **trượt** nay **lùi** state để chip không nói dối. **Số thật:** `cargo test` **250 → 251**; `npm run build` exit 0; **8/8** cổng PASS; bốn tệp `.db` dựng lại và **ba lượt dựng độc lập cho cùng bốn `sha256`** *(tái lập được đã kiểm THẬT ở v3, gồm cả tính tất định của `GROUP_CONCAT`)*; `dict-manifest.toml` cập nhật cả bốn hash. **Hai lượt đếm sai trong tài liệu đã sửa:** 7→**8** món nợ, 21→**25** khoá `vi.json`. 🔴 **Chưa chạy trên app thật:** vế DOM của lượt vá này *(bẫy tiêu điểm, cửa nuốt hợp âm, hai câu `vi.json` mới)* nghiệm thu bằng **đọc mã** và cổng tĩnh, **không** bằng một lượt render — cùng lỗ hổng mà món nợ *"vế DOM chưa có bộ chạy test"* đã khai, nay **rộng thêm** vì lượt vá thêm mã DOM mới. |
| 2026-08-10 | **Story cài xong — Status → `review`.** **Cây bẩn:** Ice chốt **commit riêng** lượt 1.18b trước *(`030890f`, tiền lệ của chính 1.18b: "diff của story đọc được một mình, và `git revert` lật được story mà không lật lượt vá")*. ⇒ **baseline THẬT của story này là `030890f`**; `baseline_commit: 51132cb` trong frontmatter **giữ nguyên** theo luật workflow. **Bốn quyết định còn lại — Ice chốt theo mặc định đề xuất:** **#1(a)** khoá thứ tư của `ScopeKind::AppConfig`, tầng Global, giá trị là tập **BỊ TẮT**; **#2(a)** bộ lọc ở **Rust**, tầng gom, `disabled` là **tham số** từ chỗ gọi; **#3(a)** **CÓ** áp cho tab Hán Việt; **#5(a)** kiểu `SourceAttribution` **riêng** + command **riêng**, `SourceInfo` giữ nguyên hai trường *(lý do đo được: `license_text` 43.304 ký tự × 7 nguồn ≈ **215 KB** trên đúng đường nóng NFR1)*. **Ba chỗ lệch so với đặc tả, ghi ra chứ không giấu:** ① **BA** command tĩnh chứ không hai — `attribution.close` là command thứ ba vì **Kiểm A của `check:commands`** đòi mọi `@click` là đúng một `dispatch('<id>')`, nên một **nút đóng** *(AC11 cần cho người dùng chuột)* không tồn tại được nếu không có command của nó; ba lý do của §KHÔNG-LÀM ⑤ vẫn đứng nguyên *(id tĩnh · đếm được · gán lại được ở 1.21)*. ② **`⌥1…6` cho từng nguồn — BÁC**, lệch so với `mockups/sources-attribution.html:140`, ba lý do đo được. ③ **Dải chip nằm NGOÀI `.lookup-head`** — Bẫy 4 đòi *"nói ra và đo"*: đầu mục 31px + `margin` 7px + thanh nhịp 15px + `padding` ⇒ **76px đã đầy**, nên chip là một hàng **riêng** đứng **trên** vùng đầu mục *(đúng thứ tự mockup vẽ)*; `--lookup-head-height` giữ nguyên giá trị **và** vai trò. **Đo AC12:** p95 **1,9 / 2,2 / 2,4 ms** cho ba cấu hình *(lượt 2)*, đều dưới trần 100 ms; tỉ lệ chạm trần `LIMIT` **1,8 % → 1,8 % → 4,8 %** — **không đổi** khi tắt một nguồn *(đúng như #2a tiên liệu)*, và mức tăng ở 9/10 là do **đường lui `Substring`** chạy nhiều hơn chứ **không** do bộ lọc. **`cargo test` 232 → 250**; `npm run build` exit 0; **7/7** cổng PASS; **9** sàn `*_FLOOR` đã nâng kèm số thật; **7** phép đỏ-rồi-xanh *(3 trên đường Rust, 4 trên cổng)*. **8 món nợ mở ở `deferred-work.md` §1-19** *(bản đầu khai 7, đếm sai; lượt code review 2026-08-10 đếm lại bằng `grep` ra 8)*. 🔴 **AC9 là một CHỖ GIỮ đã nghiệm thu bằng fixture, KHÔNG một tính năng đã chạy trên dữ liệu thật** — 0 nguồn thật mang `author-grant`, và HVTĐTD sẽ không tới. |
| 2026-08-08 | **Task 0 — hai quyết định đã chốt.** **#4 (a)** — Ice đồng ý đề xuất: story này giao **đường dữ liệu + bề mặt Attribution tối thiểu** đủ ba AC của epic; **Story 10.4** giao phần hướng-bản-phát-hành *(nút mở văn bản giấy phép · sao chép bản ghi công · hai cột biên tập **Vai trò**/**Ghi chú xuất xứ** · hai thẻ pháp lý · rà bàn phím đầy đủ)*. Bảng chia đôi ở §Quyết định #4 là hợp đồng ràng buộc cả hai story. **AC9 — Ice chốt: HVTĐTD KHÔNG TÌM ĐƯỢC NGUỒN ⇒ làm CHỖ GIỮ, có thể thay bằng nguồn khác về sau.** AC9 viết lại: mệnh đề nghiệm thu chuyển từ *"hiện đúng câu cho HVTĐTD"* sang *"**cơ chế** biểu diễn được một `license_kind` chưa gặp, và thêm một nguồn loại đó **không sửa một dòng mã**"* — đúng thứ AD-10 đã đòi bằng chữ *("mô hình hoá trường này thành enum các giấy phép mở sẽ khiến nó bị gán nhãn sai ngay trên màn hình Attribution")*. ⚠️ **Ba chỗ xuôi dòng thừa hưởng, ghi ra chứ không sửa file quy hoạch** *(tiền lệ 1.10c)*: `epics.md:1839-1841` **và** `:6202-6204` *(Story 10.4)* đều mang AC HVTĐTD nguyên văn ⇒ **cùng số phận**; `deferred-work.md:292` *(nghĩa vụ thông báo tác giả)* mất điều kiện kích hoạt; `mockups/sources-attribution.html` vẽ HVTĐTD như một hàng **có thật** — nay là hàng thứ ba của mockup đã lệch dữ liệu. |
| 2026-08-08 | Tạo story. Baseline `51132cb` **+ 7 tệp chưa commit của 1.18b**. Phân tích: `epics.md:1807-1843` §Story 1.19 + `:6174-6216` §Story 10.4 *(cùng màn hình)* + `:148-158` FR36–FR41 + `:296` FR103 + `:310-316` FR109/FR112 · `ARCHITECTURE-SPINE.md` *(AD-1 · AD-2 · AD-10 · AD-18 · AD-19 · AD-21 · AD-25 · AD-34 · AD-44)* · `EXPERIENCE.md:338` + `mockups/sources-attribution.html` *(**đặc tả thị giác nguyên văn của story này** — ba bề mặt)* · `DESIGN.md` §Typography *(`ui-label`)* · story `1-10`, `1-10b`, `1-13`, `1-16`, `1-17`, `1-18`, `1-18b` · **toàn bộ `deferred-work.md`** · mã thật `src-tauri/src/core/dict/**` · `src-tauri/src/ports/dict_source.rs` · `src-tauri/src/core/scope/**` · `src-tauri/src/commands/**` · `src/panels/**` · `src/config/**` · `scripts/*.mjs`. **Đo trên bốn tệp `.db` THẬT** *(`tools/dict-build/out/`, `sqlite3`)* — xem §Số đo mở màn. **Phát hiện:** ① `SourceInfo` chỉ mang `code`+`display_name`; **bốn** trường giấy phép chưa ai đọc *(`core/dict/mod.rs:421-423` giao đích danh story này)* — nhưng `license_text` **43.304 ký tự** và `SourceInfo` nằm trong **mọi** `SourceGroup` của **mọi** lượt tra ⇒ nhồi vào đó là đổ ~200 KB lên đúng đường nóng NFR1 *(Quyết định #2)*; ② `dict_source` có **sáu** trường giấy phép chứ không bốn *(`license_text`, `source_version` bị các story trước bỏ quên khi đếm)*; ③ **không** màn hình Cài đặt/Giới thiệu nào tồn tại — ba chế độ của AD-24 đã đầy *(Quyết định #4b)*; ④ chỉ **MỘT** nguồn phục vụ đường tiếng Anh *(`viwiktionary-en`)* ⇒ tắt nó là tắt **cả** đường `en` *(Bẫy 5)*; ⑤ `BootstrapConfig` đóng băng danh sách tên trường ở `tests/ipc_contract.rs` — thêm trường thứ sáu phải sửa **cùng lượt** *(Bẫy 1)*. |

**Epic:** 1 — Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì
**Story key:** `1-19-bat-tat-nguon-tu-dien-va-ghi-cong`
**Covers:** **FR37** *(bật/tắt từng nguồn trong Panel Lookup)* · **FR38** *(ghi công đầy đủ từng nguồn)*
**Chạm ranh giới:** **FR36** *(gỡ lớp không làm hỏng tra cứu — story này **không được** làm nó vỡ)* · **FR109** *(màn hình Attribution — **chia với Story 10.4**, xem Quyết định #4)* · **FR112** *(chính sách gỡ bỏ — story này chỉ **nói ra**, không thực thi)*
**Governed by:** **AD-1** *(quy tắc nghiệp vụ ở Rust; webview render + state UI)* · **AD-2** *(đúng ba cổng — story này **không** mở cổng thứ tư)* · **AD-10** *(một tệp `.db` = một lớp; trường giấy phép **không** là enum các giấy phép mở)* · **AD-18** *(bảng ngữ nghĩa hai tầng — thêm một `ScopeKind` là một quyết định, xem Quyết định #1)* · **AD-19** *(không hợp nhất nguồn)* · **AD-21** *(chuỗi ở `vi.json`; dây mang **dữ liệu**, không mang câu)* · **AD-25** *(0 lớp là trạng thái BÌNH THƯỜNG có tên)* · **AD-34** *(mọi thao tác qua `CommandRegistry`; handler chuột chỉ `dispatch`)* · **AD-44 ①** *(vị từ điều phối chạy **TRÊN** adapter; **không** sổ đăng ký "tệp nào chứa gì")*
**UX phải tôn trọng:** **UX-DR5** *(`ornament` là màu của **nét**)* · **UX-DR6** *(không `opacity` làm mờ chữ ở trạng thái nghỉ — một chip **tắt** phải phân biệt bằng thứ khác)* · **UX-DR8/UX-DR17** *(hợp đồng tiêu điểm)* · **UX-DR27** *(trạng thái có tên, không im lặng)* · `DESIGN.md` §Do's *(`primary` dành cho **đúng ba** việc, trong đó có **nhãn nguồn từ điển**)* · `DESIGN.md` §Giãn dòng *(sàn **1.66** cho họ `read`)*
**Ràng buộc xuôi dòng phải để lại chỗ đứng:** **Story 1.20** *(lịch sử + ghim — **tab thứ ba** của Panel Lookup; story này **không** được lấy mất chỗ đó)* · **Story 1.21** *(gán lại phím — mọi command story này đăng ký phải gán lại được)* · **Story 10.4** *(màn hình Attribution bản phát hành)* · **Story 10.5** *(giấy phép trong bản phát hành)* · **Story 3.4** *(Glossary highlight — cùng bề mặt Panel Source)* · **Story 3.7 / FR113** *(đề xuất bản dịch bằng âm Hán Việt — **đọc qua cùng cổng**, nên bộ lọc nguồn chạm nó)* · **Story 7.7** *(Concordance)* · **Story 4.12** *(màn hình hẹp)*
**NFR:** **NFR1** *(p95 < 100 ms đầu-cuối — bộ lọc nằm trên đúng đường nóng đó, **đo lại**)* · **NFR13** *(ngoại tuyến)* · **NFR14** *(hai nền tảng)* · **NFR15** *(**0** phụ thuộc mới)* · **NFR16** *(chuỗi ở `vi.json`)* · **NFR17** *(bàn phím)*
**Ngày tạo:** 2026-08-08

---

## Bối cảnh git — ĐỌC TRƯỚC KHI GÕ DÒNG ĐẦU TIÊN

```
 M _bmad-output/implementation-artifacts/deferred-work.md
 M _bmad-output/implementation-artifacts/sprint-status.yaml
 M src/panels/README.md              ← story này cũng sửa
 M src/panels/SourceHanViet.vue
 M src/panels/sourcePanelState.ts    ← story này có thể chạm (đường âm Hán Việt)
?? _bmad-output/implementation-artifacts/1-18b-tach-tu-tieng-trung-tab-han-viet.md
?? src/panels/wordBoundary.ts
```

Toàn bộ là bàn giao của **Story 1.18b**, đã `done`. **Tiền lệ đã có:** ở chính 1.18b, Ice
chốt **commit riêng lượt vá trước**, rồi mới bắt đầu story — lý do: *"diff của story đọc
được một mình, và `git revert` lật được story mà không lật lượt vá"*.

⇒ **Task 0 hỏi Ice**, không tự commit. Nếu Ice chốt commit riêng: ghi baseline **thật** vào
Change Log, **giữ nguyên** `baseline_commit: 51132cb` trong frontmatter.

---

## Số đo mở màn — BỐN TỆP `.db` THẬT, đo 2026-08-08

`sqlite3 tools/dict-build/out/*.db` — **10 nguồn** trên **4 tệp**. Đây là dữ liệu story
này phải dựng màn hình từ đó, **không** phải bảng của mockup.

| tệp *(lớp)* | `code` | `display_name` | `license_kind` | `license_id` | `len(license_text)` | đầu mục · `lang` |
|---|---|---|---|---|---|---|
| `dict-core.db` *(`base`)* | `cc-cedict` | CC-CEDICT | `open` | `CC-BY-SA-4.0` | 20.134 | 124.758 · `zh` |
| | `cvdict` | CVDICT | `open` | `CC-BY-SA-4.0` | 20.134 | 122.596 · `zh` |
| | `en-wiktionary` | Wiktionary tiếng Anh (mục tiếng Trung) | `open` | `CC-BY-SA-4.0` | 43.304 | 174.677 · `zh` |
| | `en-wiktionary-vi` | Wiktionary tiếng Anh (mục tiếng Việt…) | `open` | `CC-BY-SA-4.0` | 43.304 | 2.232 · `zh` |
| | `unihan` | Unihan | `open` | `Unicode-3.0` | 1.994 | 49.870 · `zh` |
| | `viwiktionary` | Wiktionary tiếng Việt | `open` | `CC-BY-SA-4.0` | 43.304 | 1.598 · `zh` |
| | `viwiktionary-en` | Wiktionary tiếng Việt (mục tiếng Anh) | `open` | `CC-BY-SA-4.0` | 43.304 | **119.039 · `en`** |
| `dict-thieu-chuu.db` | `thieu-chuu` | Thiều Chửu — Hán Việt Tự Điển | `public-domain` | `CC0-1.0` | 9.412 | — |
| `dict-tran-van-chanh.db` | `tran-van-chanh` | Trần Văn Chánh — Từ điển Hán Việt | **`copyrighted`** | **NULL** | 9.933 | — |
| `dict-vietphrase.db` | `vietphrase` | VietPhrase | **`unknown`** | **NULL** | 1.786 | — |

**Năm dữ kiện rút ra, mỗi cái đổi một quyết định:**

1. **`license_kind` đã có BỐN giá trị khác nhau** — `open` · `public-domain` · `copyrighted` ·
   `unknown` — và hai trong bốn **không có `license_id`**. Đúng thứ AD-10 tiên liệu: mô hình
   hoá thành enum các giấy phép mở dán nhãn sai `tran-van-chanh` và `vietphrase`. Màn hình
   **không được** suy `license_id` từ `license_kind`, và **không được** hiện chuỗi rỗng khi
   `license_id` là `NULL` — nó phải đọc `license_kind` ra một câu *(bảng ánh xạ ở `vi.json`,
   có nhánh mặc định cho một `license_kind` chưa gặp)*.
2. **`license_text` tới 43.304 ký tự cho MỘT nguồn.** Bảy nguồn của `dict-core.db` cộng lại
   ~215 KB. ⇒ nó **không được** đi kèm mỗi lượt tra *(Quyết định #2)*.
3. **`attribution` của `tran-van-chanh` mang một CẢNH BÁO PHÁP LÝ, không một lời cảm ơn:**
   *"CÒN TRONG BẢN QUYỀN, tác giả còn sống… điều đó KHÔNG xoá bản quyền tác phẩm gốc."*
   ⇒ trường này phải hiện **nguyên văn, đầy đủ**, không cắt bằng `text-overflow: ellipsis`.
4. **Đúng MỘT nguồn phục vụ đường tiếng Anh** (`viwiktionary-en`). Tắt nó ⇒ **mọi** lượt tra
   `QueryRoute::En` trả rỗng. Đó là hành vi **đúng theo FR37**, nhưng nó phải đọc ra bằng
   một câu **khác** *"không tìm thấy"* — xem AC6 và Bẫy 5.
5. **KHÔNG nguồn nào mang `license_kind = "author-grant"`, và sẽ không có.** HVTĐTD và Cổ
   hán văn đứng trong mockup lẫn trong AC của epic; `epics.md:336` ghi *"HVTĐTD và Cổ hán
   văn **chưa dựng**"*, và **Ice chốt 2026-08-08 rằng HVTĐTD không tìm được nguồn dữ liệu**.
   ⇒ AC9 **không** đo được trên dữ liệu thật, và mệnh đề nghiệm thu của nó chuyển từ *"nguồn
   X hiện đúng câu Y"* sang *"**cơ chế** biểu diễn được, và thêm một nguồn loại đó không sửa
   một dòng mã"*. Đó là mệnh đề **đúng hơn** cho một chỗ giữ: nó vẫn đứng khi nguồn thay chỗ
   HVTĐTD mang một `license_kind` mà hôm nay chưa ai biết tên.

---

## 🔴 ĐỌC TRƯỚC TIÊN — SÁU VIỆC STORY NÀY KHÔNG LÀM

### ① KHÔNG dựng "gỡ lớp" cho người dùng — **tắt** và **gỡ** là hai việc khác nhau

Mockup nói thẳng *(`sources-attribution.html:180-181`)*:

> *"**Tắt** chỉ giấu nguồn khỏi kết quả — dữ liệu vẫn nằm trong bản cài. **Gỡ** là xoá file
> dữ liệu khỏi bản phát hành, việc của **người đóng gói** chứ không phải của người dùng
> (FR112)."*

⇒ **không** nút xoá tệp, **không** đường ghi vào `resources/dict/`, **không** cơ chế tải
thêm *(NFR6: "không có cơ chế tải thêm sau khi cài")*. Một nguồn **bị tắt** vẫn có mặt đầy
đủ trên màn hình Attribution — xem **AC10**, và đó là mệnh đề dễ cài sai nhất của story.

### ② KHÔNG mở cổng thứ tư, KHÔNG dựng sổ đăng ký

AD-2 khai **đúng ba** cổng. Bộ lọc nguồn là một **tham số** đi xuống, không một cổng mới.
Và AD-44 ① vá A2 cấm *"sổ đăng ký tệp `.db` nào chứa gì"* — danh sách nguồn **luôn** dẫn
xuất từ `DictLayers` đang gắn, **không bao giờ** từ một hằng trong mã hay một bảng trong
`global.db`. Tập **bị tắt** lưu xuống đĩa là một tập **`code`**, không một bản sao danh sách
nguồn *(xem Bẫy 3)*.

### ③ KHÔNG đụng `runLookup` · `sequence` · `resetLookupPanel` · hợp đồng vùng chọn

`lookupPanelState.ts::runLookup` là **một** điểm nghẽn duy nhất và Story 1.20 *(lịch sử)*
dựng lên đúng nó *(dải chip của story này ở `sources-attribution.html:132-140`)*. Story này
**thêm một lượt tra lại** khi bộ lọc đổi — nó đi **qua** hàm đó,
không **quanh** nó. `selectionContract.ts` · `wordBoundary.ts` · `SourceHanViet.vue`:
**không một dòng đổi** trừ khi Quyết định #3 chốt (b).

### ④ KHÔNG lấy tab thứ ba của Panel Lookup

`epics.md:1874` giao **tab thứ ba** cho Story 1.20 *(lịch sử + ghim)*. Danh sách nguồn của
story này là một **dải chip trong vùng đầu mục** *(mockup `:132-141`)* cộng một bề mặt
Attribution riêng — **không** một tab.

### ⑤ KHÔNG dựng phím tắt cho từng nguồn

Mockup vẽ `⌥1…6` cạnh dải chip *(`sources-attribution.html:140`)*. **Bác**, ba lý do đo được:
- danh sách nguồn **dẫn xuất lúc chạy** *(0 tới 10 nguồn tuỳ tệp có mặt)*, còn `CommandRegistry`
  là một **danh sách tĩnh** mà `check-commands.mjs` đếm bằng máy (`COMMAND_FLOOR`) — một
  command sinh động phá chính cơ chế cưỡng chế của AD-34;
- `Mod+Alt+1`/`Mod+Alt+2` **đã thuộc** preset bố cục *(`commands/index.ts:327`)*;
- FR22/Story 1.21 đòi **mọi** command gán lại được, mà một id không tồn tại lúc dựng màn
  hình phím thì không gán được.

⇒ đăng ký **đúng hai** command tĩnh: bật/tắt nguồn **đang có tiêu điểm**, và mở Attribution.
Ghi lệch so với mockup vào Change Log kèm lý do.

### ⑥ KHÔNG dựng bộ chạy test frontend

NFR15 + Ice chốt ở 1.5, giữ qua chín story. Vế DOM nghiệm thu bằng **bàn đo chạy tay** như
1.18b, và giới hạn đó ghi lại *(nó đã là một mục trong `deferred-work.md`)*.

---

## Story

As a người dịch,
I want tắt một nguồn từ điển tôi không tin và luôn thấy được ghi công đầy đủ,
So that tôi kiểm soát được thứ mình đang đọc.

---

## Ranh giới phạm vi

| Trong phạm vi | Ngoài phạm vi (và ai sở hữu) |
|---|---|
| Đọc **sáu** trường giấy phép của `dict_source` từ **chính tệp** | Thêm/đổi một cột nào của `dict_source` *(1.9/1.10 — lược đồ đã đóng)* |
| Bật/tắt **từng nguồn**, lưu ở **tầng Global** | Bật/tắt theo **Tác phẩm** *(Quyết định #1 — bác, xem lý do)* |
| Bộ lọc áp ở **Rust**, tầng gom, **tham số từ chỗ gọi** | Lọc ở webview sau khi đã nhận kết quả *(Quyết định #2 — bác)* |
| Dải chip nguồn trong vùng đầu mục Panel Lookup | Tab thứ ba *(1.20)* · thanh trạng thái màn hình hẹp *(4.12)* |
| Bề mặt Attribution **tối thiểu đủ ba AC của epic** | Nút *"Mở văn bản giấy phép"* · *"Sao chép bản ghi công"* · cột **Vai trò/Ghi chú xuất xứ** *(**Story 10.4**, Quyết định #4)* |
| Trạng thái **"mọi nguồn đều tắt"** có tên riêng | Gộp nó vào `not_found`/`no_layers` *(cấm — AD-44 ④)* |
| **Đo lại** NFR1 với bộ lọc bật | Tin số đo p95 6,535 ms của 1.17 *(cấm — bộ lọc đổi câu SQL)* |
| **0** phụ thuộc mới · **0** cột mới · **0** cổng mới | `LICENSE`/`NOTICE` của bản phát hành *(10.5)* · gỡ tệp `.db` thật *(10.1/10.4)* |

---

## 🔴 NĂM QUYẾT ĐỊNH — CHỐT Ở TASK 0, TRƯỚC DÒNG MÃ ĐẦU TIÊN

> Mỗi quyết định có **mặc định đề xuất kèm lý do**. Chốt theo mặc định ⇒ một dòng Change Log.
> Chốt ngược ⇒ ghi **lý do**, không chỉ ghi lựa chọn.

### 🔴 Quyết định #1 — Trạng thái bật/tắt sống ở đâu? *(CHẶN THẬT)*

**(a) — MẶC ĐỊNH ĐỀ XUẤT: một khoá mới của `ScopeKind::AppConfig`, tầng Global.**

`kinds.rs` khai `AppConfig => "app_config" : GlobalOnly` và mô tả nó là *"Lựa chọn ứng dụng:
theme, chế độ cuối cùng"*. Story 1.14 đã thêm khoá thứ ba `workspace_layout` vào đúng cửa đó
*(`scope/store.rs:70`)* — tiền lệ có sẵn, rẻ, và không đụng bảng AD-18.

**Vì sao Global chứ không hai tầng:** FR103 liệt kê tầng Tác phẩm gồm *"Glossary riêng,
prompt riêng, TM riêng, ngôn ngữ nguồn"* — **không** có nguồn từ điển. Và `mockups/settings.html:246`
đã phân xử đúng lớp câu hỏi này cho phím tắt: *"một thao tác không nên đổi phím theo từng
Tác phẩm"*. Một người dịch không tin VietPhrase thì không tin nó ở **mọi** Tác phẩm.

**Hình dạng giá trị:** một chuỗi **các `code` BỊ TẮT**, phân tách bằng `,` — **không** phải
danh sách được bật.
🔴 Lý do là một bất biến: mặc định **mọi nguồn đều bật**, nên một nguồn **mới** *(một tệp
`.db` được thêm ở bản sau)* phải tự động bật. Lưu tập **được bật** làm nguồn mới im lặng
**tắt** ngay khi nó xuất hiện — một lớp dữ liệu có mặt trong bản cài mà không ai thấy, đúng
lớp lỗi *"rỗng im lặng"* mà AD-44 ④ cấm.

**(b) — một `ScopeKind` thứ mười.** **Bác** *(trừ khi Ice muốn tầng Tác phẩm)*: AC4 của
Story 1.8 đòi mỗi loại mới khai ngữ nghĩa tường minh và ký tay; một hàng mới chỉ để mang một
chuỗi phẳng là dựng một bảng cho một giá trị.

**(c) — `localStorage` ở webview.** **Cấm.** AD-1 + FR103: cấu hình sống trong `global.db`.

### 🔴 Quyết định #2 — Bộ lọc áp ở đâu? *(CHẶN THẬT — quyết định đắt nhất của story)*

**(a) — MẶC ĐỊNH ĐỀ XUẤT: ở RUST, tầng gom, `disabled` là THAM SỐ từ chỗ gọi.**

Cùng doctrine `route` / `branch` / `limit`: `commands/dict.rs` *(Panel Lookup)* quyết chính
sách, `core/dict` nhận giá trị và truyền **cùng một** giá trị xuống **mọi** tệp.

**Bốn lý do, cả bốn cưỡng chế được:**

1. **Trần `LIMIT = 20` chạy TRƯỚC.** Lọc ở webview nghĩa là một nguồn bị tắt vẫn **ăn chỗ**
   trong 20 hàng của pha một, rồi bị vứt đi — nên một nguồn **đang bật** biến mất khỏi màn
   hình chỉ vì một nguồn **đã tắt** có `entry_id` nhỏ hơn. Đó là **vỡ AC3** *("các nguồn còn
   lại không đổi")* theo cách không ai nhìn ra, và nó chỉ xảy ra trên truy vấn đông kết quả.
2. **`count_by_source` và `hidden_sources` đếm trên SQL.** Lọc ở webview để lại một thanh
   nhịp nói *"7 nguồn"* trong khi màn hình hiện 4 — AC12 của Story 1.17 cấm đích danh:
   *"thanh nhịp không bao giờ khẳng định một con số nó không biết"*.
3. **Đường âm Hán Việt không đi qua `groups`.** `lookup_han_viet` chọn **một** âm theo thứ
   tự ưu tiên lớp; một bộ lọc ở webview **không với tới** nó *(xem Quyết định #3)*.
4. **AD-1.** *"Toàn bộ quy tắc nghiệp vụ sống trong Rust."*

**Chỗ áp cụ thể:** trong `lookup_grouped`, **sau** khi có `layer.source(code)` và **trước**
khi dựng nhóm — tức lọc theo `dict_source.code`, không theo tệp. ⚠️ **Và `count_by_source`
phải nhận cùng tập** *(Bẫy 2)*.

**(b) — lọc trong câu SQL** (`AND s.code NOT IN (…)`). **Bác ở story này**: nó đụng năm câu
truy vấn của `query.rs` *(`exact` · `exact_en` · `char_idx` · `fts_trigram` · `fts_trigram_en`)*
cộng `count_by_source`, mỗi câu một `IN` sinh động — sáu bề mặt để sai thay vì một. Nó **có**
một ưu điểm thật *(trần `LIMIT` áp sau khi lọc ⇒ trang đầy hơn)*; ghi lại thành món nợ nếu
số đo AC12 cho thấy tỉ lệ cắt tăng đáng kể.

**(c) — lọc ở webview.** **Bác** — bốn lý do trên.

### 🔴 Quyết định #3 — Bộ lọc có áp cho tab **Hán Việt** không?

**(a) — MẶC ĐỊNH ĐỀ XUẤT: CÓ, và `lookup_han_viet` nhận cùng tham số.**

FR37 nói *"kết quả từ nguồn đó **không xuất hiện**"*. Âm Hán Việt **là** một kết quả tra cứu
mang `source_code` *(`HanVietReading::source_code`, FR31)*, và `HanVietLookup::sources_used`
**hiện tên nguồn lên màn hình** *(`panel.source.han_viet_sources_prefix`)*. Để nó ngoài bộ
lọc là để một nguồn *"đã tắt"* vẫn viết chữ lên tab Hán Việt — một câu tự mâu thuẫn ngay
trên màn hình.

⚠️ **Hệ quả bắt buộc, đo được:** `priority_order()` đẩy lớp NỀN xuống cuối, nên tắt các lớp
gỡ rời **đổi âm hiển thị** chứ không chỉ giấu bớt — `西` có thể đi từ âm của `tran-van-chanh`
về âm của `unihan`/`en-wiktionary-vi`. Đó là hành vi **đúng** *(cùng cơ chế mà FR36 dựa vào
khi một lớp bị gỡ)*, nhưng nó phải **đo và ghi ra**, không để người đọc phát hiện sau.

⚠️ **Hệ quả thứ hai:** Panel Source phải **tra lại** khi bộ lọc đổi *(cùng cửa
`sourcePanelState.ts` đang có)*, không giữ âm cũ trên màn hình.

**(b) — KHÔNG áp, chỉ áp cho Panel Lookup.** Chọn (b) thì phải **viết ra** trên màn hình
rằng tab Hán Việt không theo bộ lọc — im lặng là lựa chọn duy nhất **bị cấm**.

### ✅ Quyết định #4 — Màn hình Attribution: 1.19 dựng tới đâu, 10.4 còn gì? — **CHỐT (a)**

> **Ice chốt 2026-08-08: đồng ý đề xuất.** Bảng dưới đây là **hợp đồng ràng buộc cả hai
> story** — 10.4 không được coi bề mặt của 1.19 là bản nháp để dựng lại, và 1.19 không được
> lấn sang cột phải.

**(a) — ĐÃ CHỐT: 1.19 dựng ĐƯỜNG DỮ LIỆU + bề mặt TỐI THIỂU đủ ba AC của epic;
10.4 dựng phần hướng-bản-phát-hành.**

| 1.19 giao | 10.4 giao |
|---|---|
| Command `list_dict_sources` — mọi nguồn của **mọi tệp có mặt**, kèm sáu trường | Nút *"Mở văn bản giấy phép"* *(`license_text` đầy đủ)* |
| Bảng: tên · giấy phép · lớp *(nền/gỡ rời)* · ghi công | Nút *"Sao chép bản ghi công"* |
| `license_kind` đọc ra câu, có nhánh mặc định | Cột **Vai trò** và **Ghi chú xuất xứ** *(nội dung biên tập, không có trong `.db`)* |
| Nguồn **bị tắt** vẫn liệt kê đầy đủ (AC10) | Hai thẻ *"Giấy phép của phần mềm"* / *"Nếu bạn là chủ sở hữu"* |
| Xoá một `.db` ⇒ ghi công biến mất (AC8) | Rà bàn phím đầy đủ + nghĩa vụ thông báo tác giả HVTĐTD *(`deferred-work.md:292`)* |

**Đặt màn hình ở đâu — (a) một lớp phủ mở bằng command, KHÔNG một chế độ thứ tư.** AD-24
khai **ba** chế độ ngang hàng và `MODE_IDS` là một hằng có ba phần tử; thêm chế độ thứ tư là
một quyết định kiến trúc *(và `Mod+4` là phím của Story 8.11)*. Lớp phủ dựng trong `App.vue`,
mở bằng command `attribution.open`, đóng bằng `Escape`, trả tiêu điểm về chỗ cũ *(UX-DR17)*.

**(b) — hoãn toàn bộ Attribution sang 10.4.** **Bác**: ba AC của epic thuộc **story này**,
và một story giao một nửa AC của chính nó là một story chưa xong.

### 🔴 Quyết định #5 — `SourceInfo` có mang thêm trường giấy phép không?

**(a) — MẶC ĐỊNH ĐỀ XUẤT: KHÔNG. Một kiểu RIÊNG, một command RIÊNG, đọc MỘT LẦN.**

`SourceInfo` nằm trong **mọi** `SourceGroup` của **mọi** lượt tra — tức trên đúng đường nóng
NFR1. Số đo ở §Số đo mở màn: `license_text` một mình là **43.304 ký tự**; bảy nguồn của
`dict-core.db` cộng lại ~215 KB. Nhồi vào `SourceInfo` là đổ ngần đó qua IPC **mỗi lần bôi
đen** — Auto-Lookup chạm nó hàng trăm lần mỗi Chương.

⇒ `SourceInfo` **giữ nguyên hai trường**. Thêm `SourceAttribution` *(sáu trường + `layer` +
`is_base`)*, đọc bởi một command **riêng** mà màn hình Attribution gọi **một lần khi mở**.

⚠️ **`license_text` trên dây:** mặc định **không gửi** — gửi độ dài, để 10.4 sở hữu đường
mở văn bản đầy đủ. Chốt ngược thì phải ghi số byte thật đã đo.

**Đọc ở đâu:** `DictLayer::open` hiện chỉ `SELECT code, display_name`. Thêm một truy vấn
**thứ hai lúc mở** *(một lần cho mỗi tệp, cả đời tiến trình)* hay đọc **lúc gọi command**?
🔴 **Đề xuất: lúc gọi command** — `license_text` ~215 KB giữ thường trực trong RAM cho một
màn hình hiếm mở là một cái giá không ai xin.

**(b) — nới `SourceInfo`.** **Bác** — lý do đo được ở trên.

---

## Acceptance Criteria

> Quy ước: **Given/When/Then**. Mọi AC mang chữ *"đo"* đòi **số THẬT trên tệp `.db` thật**,
> không suy luận, và mọi phép đo có **đối chứng âm**.

### AC1 — Danh sách nguồn dựng từ tệp CÓ MẶT, không từ một danh sách viết cứng

**Given** thư mục từ điển có `n` tệp `.db` hợp lệ
**When** Panel Lookup dựng dải chip nguồn
**Then** dải chip chứa **đúng** tập `dict_source.code` của các tệp đó, thứ tự tất định
**And** **0** tên nguồn nào xuất hiện dưới dạng chuỗi viết cứng trong `src/**` hoặc `src-tauri/src/**`
*(nghiệm thu bằng `grep` cho mười `code` thật; số phải là **0** ở vị trí mã)*
**And** thêm một tệp `.db` mới ⇒ chip mới xuất hiện, **không sửa một dòng mã**

### AC2 — Bật/tắt từng nguồn, hiệu lực NGAY

**Given** một dải chip nguồn và một kết quả tra cứu đang hiện
**When** người dùng tắt một nguồn
**Then** kết quả tra cứu hiện lại **không có** nguồn đó, **không cần** bôi đen lại
**And** chip đó phân biệt được bằng mắt ở trạng thái tắt **không dùng `opacity` làm mờ chữ**
*(UX-DR6)*
**And** bật lại ⇒ nguồn trở lại, kết quả giống hệt trước khi tắt *(đối chứng âm bắt buộc)*

### AC3 — Nguồn bị tắt biến mất, các nguồn còn lại KHÔNG đổi

**Given** một truy vấn cho `k` nhóm nguồn
**When** tắt một nguồn rồi tra lại **cùng** truy vấn đó
**Then** còn `k−1` nhóm, và **từng nhóm còn lại giống hệt** bản trước — cùng đầu mục, cùng
nghĩa, cùng thứ tự
**And** thanh nhịp đọc `k−1`, **không** `k`
**And** `hidden_sources` và `total_entries` tính trên **tập đã lọc**
**And** 🔴 **ca ĐẶC BIỆT phải có test:** một truy vấn **chạm trần `LIMIT = 20`** — tắt một
nguồn ở đó phải làm các nguồn còn lại **nhiều kết quả hơn hoặc bằng**, **không bao giờ ít
hơn** *(đây là ca mà một bộ lọc ở webview vỡ; xem Quyết định #2 lý do 1)*

### AC4 — Bộ lọc là THAM SỐ từ chỗ gọi, một giá trị cho cả lượt tra

**Given** `lookup_grouped` / `lookup_han_viet`
**When** chạy
**Then** tập nguồn bị tắt **nhận từ chỗ gọi**, **không** đọc `Store` bên trong `core/dict/**`
**And** cùng **một** giá trị đi xuống **mọi** tệp *(cùng doctrine `route`/`branch`/`limit`)*
**And** `core/dict/**` **không** gõ tên một `code` nguồn cụ thể nào
**And** `tests/dict_boundary.rs` canh mệnh đề đó bằng máy

### AC5 — Lựa chọn sống sót qua khởi động lại

**Given** người dùng tắt hai nguồn
**When** đóng rồi mở lại ứng dụng
**Then** đúng hai nguồn đó vẫn tắt
**And** giá trị lưu ở **tầng Global** (`ScopeKind::AppConfig`, Quyết định #1)
**And** giá trị lưu là tập **BỊ TẮT** — một nguồn **mới** xuất hiện ở bản sau **mặc định bật**
*(đối chứng âm: thêm một tệp `.db` sau khi đã lưu ⇒ nguồn mới **bật**)*
**And** một `code` đã lưu mà tệp của nó **không còn** ⇒ bỏ qua im lặng, **không** lỗi, **không**
để lại một chip mồ côi

### AC6 — "Mọi nguồn đều tắt" là một trạng thái CÓ TÊN

**Given** người dùng tắt **mọi** nguồn *(hoặc mọi nguồn của đường đang tra — xem Bẫy 5)*
**When** tra cứu
**Then** Panel Lookup nói ra bằng một chuỗi **RIÊNG**, khác cả bốn chuỗi đã có
(`not_found` · `query_too_short` · `no_layers` · `lookup_failed`)
**And** câu đó chỉ đường về dải chip *(bật lại một nguồn)*, **không** nói *"không tìm thấy"*
**And** chuỗi sống ở `vi.json` *(NFR16)*

### AC7 — Màn hình Attribution liệt kê MỌI nguồn có mặt

**Given** màn hình Attribution
**When** mở
**Then** liệt kê **mọi** nguồn của **mọi** tệp `.db` đang gắn — hôm nay là **10 nguồn / 4 tệp**
**And** mỗi hàng mang: tên hiển thị · giấy phép · lớp *(nền hoặc gỡ rời)* · ghi công **nguyên
văn, không cắt**
**And** `license_id` là `NULL` ⇒ hiện câu của `license_kind`, **không** một ô trống

### AC8 — Dựng từ tệp có mặt; xoá một `.db` ⇒ ghi công biến mất, không mồ côi

**Given** một tệp `.db` bị xoá khỏi thư mục từ điển
**When** mở lại ứng dụng rồi mở Attribution
**Then** ghi công của **mọi** nguồn trong tệp đó biến mất
**And** **0** ghi công mồ côi ở lại
**And** đường tra cứu **vẫn chạy đầy đủ** trên các lớp còn lại *(FR36 — nghiệm thu bằng
đúng phép thử của AD-10: xoá tệp rồi **chạy lại toàn bộ bộ test tra cứu**, phải vẫn xanh)*

### AC9 — `license_kind` không bị ép vào enum, và **chỗ giữ cho giấy phép riêng** đứng vững

> 🔴 **Ice chốt 2026-08-08: HVTĐTD KHÔNG TÌM ĐƯỢC NGUỒN.** AC gốc của epic
> *(`epics.md:1839-1841`)* neo vào **một nguồn cụ thể**, và nguồn đó sẽ không tới. AC này
> neo lại vào **cơ chế** — thứ vẫn đứng khi một nguồn **khác** thay chỗ nó.

**Given** bốn giá trị `license_kind` đo được trên dữ liệu thật (`open` · `public-domain` ·
`copyrighted` · `unknown`)
**When** hiển thị
**Then** cả bốn ra đúng câu của mình
**And** hai nguồn có `license_id = NULL` (`tran-van-chanh` · `vietphrase`) **không** hiện ô
trống — chúng đọc câu của `license_kind`

**Given** một `license_kind` **chưa gặp bao giờ**
**When** hiển thị
**Then** ra một câu mặc định **có nghĩa** — nêu được rằng giấy phép này **chưa được ứng dụng
phân loại** và trỏ người đọc vào trường `attribution` nguyên văn
**And** **không** rỗng, **không** hiện chuỗi máy thô, **không** panic
*(đối chứng âm bắt buộc: fixture với một `license_kind` bịa ra)*

**Given** một nguồn dùng theo **phép riêng do tác giả cấp** *(chỗ giữ — `license_kind =
"author-grant"`)*
**When** tệp `.db` của nó được thêm vào thư mục từ điển
**Then** nó hiện đủ tên · giấy phép · lớp · ghi công **mà KHÔNG sửa một dòng mã nào**
**And** câu của `author-grant` nêu **phép riêng tác giả cấp** và **KHÔNG thuộc GPL v3**
**And** 🔴 phần **danh tính tác giả** đọc từ `dict_source.attribution` của **chính tệp**,
**không** viết cứng một cái tên trong `src/**` hay `vi.json` — đó là điều kiện để chỗ giữ
này dùng lại được cho một nguồn khác
*(nghiệm thu bằng **fixture**, và bằng phép thử của AD-10: thả tệp vào ⇒ hiện; xoá đi ⇒ biến
mất, không mồ côi)*

**And** ⚠️ **GIỚI HẠN PHẢI GHI THẲNG trong §Completion Notes:** **0** nguồn thật nào mang
`license_kind = "author-grant"` hôm nay, và HVTĐTD sẽ không tới. Đây là một **chỗ giữ đã
nghiệm thu bằng fixture**, **không** một tính năng đã chạy trên dữ liệu thật — đừng đánh dấu
nó "đạt trên dữ liệu thật"

### AC10 — TẮT ≠ GỠ: nguồn bị tắt vẫn có mặt đầy đủ trong Attribution

**Given** người dùng đã tắt một nguồn
**When** mở Attribution
**Then** nguồn đó **vẫn** liệt kê đầy đủ kèm giấy phép và ghi công
**And** màn hình nói ra rằng **tắt chỉ giấu khỏi kết quả**, **gỡ** là xoá tệp và là việc của
người đóng gói *(FR112 — mockup `:180-181`)*

### AC11 — Command trong `CommandRegistry`, bàn phím đi hết

**Given** mọi thao tác story này thêm
**When** gọi
**Then** đi qua một command đăng ký trong `CommandRegistry` *(AD-34)*, gán lại được ở 1.21
**And** **0** handler `@click` nào làm gì khác ngoài đúng một `dispatch('<id>')`
**And** dải chip và bảng Attribution **duyệt được bằng bàn phím**; `Escape` đóng lớp phủ và
**trả tiêu điểm về chỗ cũ**
**And** số điểm dừng `Tab` mới **khai ra thành số**, không phát sinh ngoài khai báo

### AC12 — NFR1 KHÔNG hồi quy — ĐO LẠI, không suy từ số cũ

**Given** đường `commands::dict::lookup` trên **bốn tệp `.db` thật**, bản `--release`
**When** đo ≥ 100 lượt liên tiếp, có bộ lọc bật và tắt
**Then** ghi **p95** cho ba cấu hình: **0 nguồn tắt** · **1 nguồn tắt** · **9/10 nguồn tắt**
**And** cả ba dưới trần đầu-cuối **100 ms** của NFR1
**And** ghi số của ca xấu nhất tìm được *(1.17 đo `"山"` ⇒ p95 **6,535 ms** — đây là mốc so
sánh, không phải kết luận)*
**And** ghi tỉ lệ lượt tra **chạm trần `LIMIT`** trước và sau — nếu nó **tăng đáng kể** thì
Quyết định #2(b) *(lọc trong SQL)* thành một món nợ có số, ghi vào `deferred-work.md`

### AC13 — Cổng và sàn

**Given** chín lệnh DoD
**When** chạy sau story
**Then** `cargo test` xanh *(số ca **tăng**, ghi số trước/sau)* · `npm run build` exit 0 ·
**7 cổng `check:*`** exit 0
**And** mọi hằng `*_FLOOR` bị vượt đã nâng, kèm **số THẬT** ghi vào comment cạnh hằng
**And** **đỏ-rồi-xanh** cho ít nhất hai cổng bị đụng, mỗi ca kèm **đối chứng âm**

---

## Tasks / Subtasks

- [x] **Task 0 — Chốt năm quyết định + xử lý cây bẩn** *(CHẶN — trước dòng mã đầu tiên)*
  - [x] Hỏi Ice về bảy tệp chưa commit của 1.18b *(§Bối cảnh git)*; **không tự commit**
  - [x] ✅ **Quyết định #4 — CHỐT (a)** *(Ice, 2026-08-08)*: bảng chia đôi 1.19 / 10.4
  - [x] ✅ **AC9 — CHỐT chỗ giữ** *(Ice, 2026-08-08)*: HVTĐTD không tìm được nguồn ⇒ AC neo
        vào **cơ chế**, không vào một nguồn cụ thể
  - [x] Chốt Quyết định **#1 · #2 · #3 · #5**, ghi mỗi cái một dòng Change Log kèm **lý do**
        nếu chốt ngược
  - [x] Đo baseline **thật, không chép**: `cargo test` · `npm run build` · 7 cổng `check:*`

- [x] **Task 1 — Đọc sáu trường giấy phép từ chính tệp** (AC7, AC9)
  - [x] Kiểu `SourceAttribution` ở `core/dict/` — `code` · `display_name` · `license_kind` ·
        `license_id` · `attribution` · `source_version` · `source_url` · `layer` · `is_base`
        *(+ độ dài `license_text`, xem Quyết định #5)*
  - [x] Đường đọc trong `DictLayer` *(một truy vấn, lúc gọi — không giữ thường trực)*
  - [x] ⚠️ **Không** thêm cột, **không** đổi một dòng DDL nào của `tools/dict-build/src/schema.rs`
  - [x] `is_base` đọc từ `dict_meta('layer') == "base"`, **không** từ tên tệp *(AD-44 ① vá A2)*

- [x] **Task 2 — Bộ lọc ở tầng gom** (AC3, AC4)
  - [x] `lookup_grouped` nhận tập `code` bị tắt, áp **sau** `layer.source(code)`
  - [x] 🔴 `count_by_source` nhận **cùng** tập — nếu quên, thanh nhịp đếm cả nguồn đã tắt *(Bẫy 2)*
  - [x] `hidden_sources` chỉ gom nguồn **đang bật**
  - [x] `lookup_han_viet` nhận cùng tham số *(nếu Quyết định #3 chốt (a))*
  - [x] ⚠️ **0** lời đọc `Store` bên trong `core/dict/**`

- [x] **Task 3 — Hai command IPC** (AC1, AC7)
  - [x] `list_dict_sources` — vỏ `#[tauri::command]` bọc một **hàm thuần** *(khuôn `read_han_viet`)*
  - [x] `lookup_dictionary` đọc tập bị tắt từ `Store` **ở tầng command**, truyền xuống
  - [x] ⚠️ `try_state`, **không** `state()` *(`panic = "abort"`)*
  - [x] Cập nhật `tests/ipc_contract.rs` **cùng lượt** nếu `BootstrapConfig` mọc trường thứ sáu *(Bẫy 1)*

- [x] **Task 4 — Lưu xuống tầng Global** (AC5)
  - [x] Khoá mới của `ScopeKind::AppConfig` ở `scope/store.rs`, kèm hằng có tên
  - [x] Ghi qua `putConfig(SCOPE_APP_CONFIG, …)` — đường đã có, **không** đường thứ hai
  - [x] Đọc lúc khởi động; `code` không còn tệp ⇒ bỏ qua im lặng *(AC5)*

- [x] **Task 5 — Dải chip trong Panel Lookup** (AC2, AC6, AC11)
  - [x] Chip dẫn xuất từ `list_dict_sources`, **không** từ `groups` *(nguồn đang tắt không có
        nhóm nào, nên nó phải hiện được từ danh sách đầy đủ)*
  - [x] Trạng thái tắt phân biệt **không** bằng `opacity` trên chữ *(UX-DR6)*
  - [x] Tra lại **qua `runLookup`**, không quanh nó *(§KHÔNG-LÀM ③)*
  - [x] Chuỗi mới vào `vi.json`; **0** chuỗi trong `.vue`
  - [x] ⚠️ Vùng đầu mục khoá `height: 76px; overflow: hidden` — dải chip **không được** thêm
        một pixel nào vào đó *(Bẫy 4)*

- [x] **Task 6 — Bề mặt Attribution** (AC7, AC8, AC9, AC10, AC11)
  - [x] Lớp phủ mở bằng command `attribution.open`, đóng bằng `Escape`, trả tiêu điểm
  - [x] Bảng: tên · giấy phép · lớp · ghi công *(nguyên văn, không `ellipsis`)*
  - [x] Ánh xạ `license_kind` → câu ở `vi.json`, **có nhánh mặc định**
  - [x] Câu *"tắt ≠ gỡ"* (AC10) và câu FR112
  - [x] 🔴 `useSelectionSurface(ref, 'display')` cho bề mặt này + nâng `SELECTION_SURFACE_FLOOR`
        lên số thật *(Bẫy 8)*

- [x] **Task 7 — Test Rust** (AC3, AC4, AC5, AC8, AC9)
  - [x] `dict_sources.rs`: ca **trần `LIMIT`** của AC3 *(ca đắt nhất — dựng fixture đủ hàng)*
  - [x] Ca **xoá một tệp rồi chạy lại bộ test tra cứu** *(FR36 / AC8)*
  - [x] Ca `license_kind` **bịa ra, chưa gặp** ⇒ câu mặc định có nghĩa *(AC9)*
  - [x] Ca **chỗ giữ `author-grant`**: thả fixture vào ⇒ hiện đủ; xoá ⇒ biến mất, không mồ côi
  - [x] Ca **danh tính tác giả đọc từ `attribution` của chính tệp** — `grep` khẳng định **0**
        tên tác giả viết cứng trong `src/**` và `vi.json` *(AC9)*
  - [x] Ca **hai nguồn cùng tệp**, tắt một *(`dict-core.db` mang bảy — đây là hình dạng thật)*
  - [x] `dict_boundary.rs`: **0** `code` viết cứng trong `core/dict/**`

- [x] **Task 8 — Đo NFR1** (AC12)
  - [x] Bản `--release`, bốn tệp thật, ≥ 100 lượt, ba cấu hình bộ lọc
  - [x] Ghi bảng đầy đủ vào §Debug Log References — **số, không lời khen**
  - [x] Ghi tỉ lệ chạm trần `LIMIT` trước/sau

- [x] **Task 9 — Cổng, sàn, tài liệu** (AC13)
  - [x] Chín lệnh DoD; nâng `*_FLOOR` bị vượt kèm số THẬT
  - [x] Đỏ-rồi-xanh ≥ 2 cổng, mỗi ca có đối chứng âm
  - [x] `src/panels/README.md`: hàng 1.19 + §Bật tắt nguồn
  - [x] `deferred-work.md`: mở §1-19 với món nợ thật *(tối thiểu: WKWebView chưa đo)*

---

## Dev Notes

### Mã hiện có — trạng thái ĐÚNG hôm nay

| tệp | hôm nay làm gì | story này đổi gì |
|---|---|---|
| `src-tauri/src/ports/dict_source.rs` | Trait 5 method; `sources() -> &[SourceInfo]` | Có thể thêm method đọc giấy phép *(hoặc đọc thẳng trong `DictLayer` — Quyết định #5)* |
| `core/dict/layer.rs:305-319` | `DictLayer::open` `SELECT code, display_name` | Thêm đường đọc sáu trường; **giữ nguyên** phép kiểm trùng `code` |
| `core/dict/mod.rs:425-431` | `SourceInfo` hai trường; comment `:421-423` giao bốn trường giấy phép cho **story này** | ⚠️ Comment ghi **bốn**; số thật là **sáu** *(`license_text`, `source_version` bị bỏ quên)* — sửa comment cùng lượt |
| `core/dict/mod.rs:696-806` | `lookup_grouped` — `route`/`branch`/`limit` từ chỗ gọi | Thêm tập bị tắt, cùng doctrine |
| `core/dict/mod.rs:938-1016` | `lookup_han_viet` — `priority_order` rồi hàng đầu thắng | Lọc trước khi chọn ưu tiên *(Quyết định #3a)* |
| `commands/dict.rs:137-186` | `lookup()` thuần; `LOOKUP_PAGE_LIMIT = 20`; hai lượt `Exact` → `Substring` | Đọc tập bị tắt, truyền xuống **cả hai lượt** |
| `core/scope/kinds.rs` | 9 `ScopeKind`; `AppConfig => GlobalOnly` | Khoá mới, **không** biến thể mới *(Quyết định #1a)* |
| `core/scope/store.rs:43-70` | `KEY_THEME`/`KEY_MODE`/`KEY_LAYOUT` | Hằng thứ tư |
| `commands/config.rs:56-77` | `BootstrapConfig` **5 trường**, `snake_case`, **không** `rename_all` | Trường thứ sáu ⇒ sửa `tests/ipc_contract.rs` **cùng lượt** |
| `src/panels/lookupPanelState.ts:79-107` | `computeSpine` — *"1.19 chỉ được phép **THÊM một cờ**, không dựng lại cả hàm"* | Tôn trọng nguyên văn |
| `src/panels/LookupPanel.vue:474-490` | `.lookup-head` `height: 76px; overflow: hidden` | Dải chip **không** đổi chiều cao đó |
| `src/config/bootstrap.ts:118-119` | `SCOPE_APP_CONFIG`/`KEY_LAYOUT`, `putConfig` | Dùng lại, **0** đường ghi thứ hai |

### Tám cái bẫy — mỗi cái đã cắn ai đó rồi

**Bẫy 1 — `BootstrapConfig` là một hợp đồng ĐÓNG BĂNG.** `commands/config.rs:69-72` viết
sẵn: *"Trường thứ **năm** trên dây. `tests/ipc_contract.rs` đóng băng danh sách tên trường và
nó phải được sửa cùng lượt."* Và **không** `#[serde(rename_all = "camelCase")]` — đặt nó vào
biến `dict_sources_disabled` thành `dictSourcesDisabled` và `src/config/bootstrap.ts` nhận
`undefined`, **không lỗi nào được ném**.

**Bẫy 2 — `count_by_source` là đường thứ hai, và nó DỄ QUÊN.** Nó chỉ chạy khi
`result.truncated` — tức **phần lớn lượt tra không đi qua nó**, nên một bản quên lọc ở đó
chạy đúng trong mọi test nhỏ và sai đúng trên truy vấn đông kết quả. Ca test của AC3 phải
**ép** `truncated = true`.

**Bẫy 3 — tập bị tắt là `code`, và `code` chỉ duy nhất TRONG TOÀN TẬP LỚP, không trong một
tệp.** `conflict_with()` đã cưỡng chế điều đó *(`SkipReason::DuplicateSourceCode`)*, nên
khoá theo `code` là **an toàn** — nhưng khoá theo `id` thì **không**: `id = 1` tồn tại ở cả
bốn tệp và trỏ bốn nguồn khác nhau. `EntryHit::source_code` và `SourceInfo` đã ghi lý do này
bằng chữ; đừng đi ngược.

**Bẫy 4 — vùng đầu mục Panel Lookup KHOÁ chiều cao.** `--lookup-head-height: 76px` +
`overflow: hidden`, và Story 1.17/1.18 đã vỡ đúng chỗ này **hai lần** *(một lần với thanh
nhịp, một lần với vạch tiến trình)*. Dải chip nguồn là thứ thứ ba muốn chỗ trong đó. Nếu nó
không vừa, **nói ra và đo**, đừng nới hằng trong im lặng.

**Bẫy 5 — tắt `viwiktionary-en` là tắt CẢ ĐƯỜNG TIẾNG ANH.** Đo được: đúng **một** nguồn
mang `lang = 'en'`. Panel sẽ hiện *"không tìm thấy"* cho **mọi** truy vấn tiếng Anh, một câu
**SAI** — hệ thống không hề tra. AC6 tồn tại vì ca này; và nó gợi ý rằng vị từ *"mọi nguồn
đều tắt"* nên hỏi theo **đường đang tra** (`route`), không theo toàn tập.

**Bẫy 6 — `priority_order` làm bộ lọc ĐỔI ÂM, không chỉ giấu bớt.** Xem Quyết định #3. Một
test chỉ khẳng định *"`sources_used` không chứa nguồn đã tắt"* sẽ **xanh** trong khi âm hiển
thị đã đổi mà không ai đo. Ca test phải khẳng định **âm cụ thể** cho một ký tự cụ thể.

**Bẫy 7 — chuỗi trong `src-tauri/**/*.rs` phải viết KHÔNG DẤU.** `check-i18n.mjs` Kiểm A quét
`src-tauri/**/*.rs` và `commands/dict.rs`/`core/dict/**` **không** nằm trong miễn trừ.
Doc-comment thì có dấu thoải mái; chuỗi thì không. `tests/**` được miễn trừ.

**Bẫy 8 — bề mặt Attribution là một BỀ MẶT VĂN BẢN THỨ SÁU, và hợp đồng vùng chọn hỏi nó.**
`check-commands.mjs:1642` khai `SELECTION_SURFACE_FLOOR = 5` kèm mệnh đề nguyên văn: *"Sàn =
SỐ THẬT hôm nay; Story 1.20/3.4 sẽ **THÊM** bề mặt, không bớt."* Bảng Attribution chứa chữ
thật *(ghi công, giấy phép)*, nên nó rơi vào đúng lớp câu hỏi mà `useSelectionSurface` tồn
tại để trả lời.
🔴 **Vai phải là `'display'`, không `'source'`** — cùng lý do Bẫy 1 của Story 1.18 đã bắt ở
Panel Lookup: bôi đen một dòng ghi công để đọc kỹ mà phát ra một lượt tra cứu là thay chính
đoạn đang đọc dưới tay người đọc. ⇒ đăng ký tường minh và **nâng sàn lên số thật**; một bề
mặt văn bản im lặng đứng ngoài sổ là đúng thứ AC2 của 1.18 tồn tại để chặn.

### Câu SQL đọc giấy phép — hình dạng đề xuất

```sql
SELECT code, display_name, license_kind, license_id,
       length(license_text), attribution, source_version, source_url
  FROM dict_source
 ORDER BY code;
```

⚠️ `ORDER BY code` giữ hai hàng trùng kề nhau — cùng lý do `DictLayer::open` đang dùng, và
phép kiểm `DuplicateSourceCodeInFile` đứng lên nó.

### Chuỗi mới cần khai ở `vi.json` *(danh sách tối thiểu, không đóng)*

`panel.lookup.all_sources_off` · `panel.lookup.sources_label` · `panel.lookup.source_off_hint` ·
`command.lookup.toggle_source` · `command.attribution.open` · `attribution.title` ·
`attribution.col_source` · `attribution.col_license` · `attribution.col_layer` ·
`attribution.col_credit` · `attribution.layer_base` · `attribution.layer_detachable` ·
`attribution.off_is_not_removed` · `attribution.license_open` · `attribution.license_public_domain` ·
`attribution.license_copyrighted` · `attribution.license_unknown` ·
`attribution.license_author_grant` *(🔴 **chỗ giữ** — **0** nguồn thật mang nó; câu chỉ nói
**"phép riêng tác giả cấp · không thuộc GPL v3"**, danh tính tác giả đọc từ `attribution` của
tệp, **không** viết vào đây)* · `attribution.license_unrecognised` *(nhánh mặc định — AC9)*

### Testing

- **Rust:** `src-tauri/tests/dict_sources.rs` là chỗ đúng — nó đã dựng **ba tệp fixture** với
  `id = 1` trùng nhau **có chủ ý**, đúng hình dạng story này cần. Bốn luật của tệp đó *(thư mục
  tạm riêng · drop `ReadOnlyDb` trước khi xoá · không ngưỡng thời gian trong CI · đường dẫn qua
  `CARGO_MANIFEST_DIR`)* áp nguyên.
- **Frontend: KHÔNG có bộ chạy test** *(NFR15, Ice chốt ở 1.5)*. Vế DOM nghiệm thu bằng bàn đo
  chạy tay; ghi rõ bàn đo **không** được commit, như 1.18b đã làm.
- **Cổng:** `check:i18n` *(chuỗi + Kiểm A2 text node)* · `check:commands` *(`COMMAND_FLOOR = 18`,
  thật 22 → tăng 2)* · `check:tokens` *(`FILE_FLOOR`/`COMPONENT_FILE_FLOOR`)* · `check:layout`
  *(`FILE_FLOOR = 32`)* · `check:i18n` `RS_FLOOR = 34`/`VUE_FLOOR = 11`.

### Project Structure Notes

Tệp mới dự kiến: một component Attribution — **một lớp phủ dựng trong `App.vue`**, không một
chế độ thứ tư *(Quyết định #4 đã chốt)* — và có thể
một `src/config/` adapter cho `list_dict_sources`. **Không** thư mục mới ở `src-tauri/src/` —
`SourceAttribution` thuộc `core/dict/`, command thuộc `commands/dict.rs`.

⚠️ `src/config/` là thư mục **ngoài** cây nguồn khai trong `ARCHITECTURE-SPINE.md` và lý do
đã ghi ở `src/config/bootstrap.ts:5-19` — đi theo khuôn đó, đừng đặt adapter IPC vào
`src/commands/**` *(ba cổng nạp tệp ở đó bằng Node thuần)*.

### References

- `_bmad-output/planning-artifacts/epics.md:1807-1843` — §Story 1.19 *(sáu AC gốc)*
- `_bmad-output/planning-artifacts/epics.md:6174-6216` — §Story 10.4 *(cùng màn hình — Quyết định #4)*
- `_bmad-output/planning-artifacts/epics.md:148-158` — FR36 · FR37 · FR38 · FR39
- `_bmad-output/planning-artifacts/epics.md:296,310-316` — FR103 · FR109 · FR112
- `_bmad-output/planning-artifacts/architecture/…/ARCHITECTURE-SPINE.md:147-151` — **AD-10**
- `_bmad-output/planning-artifacts/architecture/…/ARCHITECTURE-SPINE.md:75-79` — **AD-1**
- `_bmad-output/planning-artifacts/ux-designs/…/mockups/sources-attribution.html` — **ba bề mặt**
- `_bmad-output/planning-artifacts/ux-designs/…/EXPERIENCE.md:338` — bảng mockup ↔ FR
- `_bmad-output/implementation-artifacts/1-10-dong-goi-bon-lop-go-roi-thanh-file-doc-lap.md:71-91` — bốn trường giấy phép, và vì sao `vietphrase` **không** là `public-domain`
- `_bmad-output/implementation-artifacts/1-17-panel-lookup-ban-ghi-co-cau-truc.md:66-73` — §KHÔNG-LÀM ② *(mệnh đề mà story này phải tôn trọng nguyên văn)*
- `_bmad-output/implementation-artifacts/deferred-work.md:292` — nghĩa vụ thông báo tác giả HVTĐTD *(chủ: 10.4)*
- `_bmad-output/implementation-artifacts/deferred-work.md:579-581` — rủi ro pháp lý `tran-van-chanh`, và lệch với `prd.md §8.2`
- `src-tauri/src/core/dict/mod.rs:421-423` — giao **trường giấy phép** cho story này *(ghi "bốn", số thật **sáu**)*
- `tools/dict-build/src/schema.rs:26-43` — `DICT_SOURCE_DDL` *(chín cột)*

---

## Ghi cho story xuôi dòng — hệ quả của quyết định HVTĐTD

Ice chốt 2026-08-08 rằng **không tìm được nguồn dữ liệu HVTĐTD**. Story này **không sửa file
quy hoạch** *(tiền lệ 1.10c: "ghi ra, KHÔNG sửa `epics.md`/`prd.md`")*, nên bốn chỗ dưới đây
**vẫn** mang mệnh đề cũ và người đọc sau sẽ vấp phải chúng:

| chỗ | mang gì | thành gì sau quyết định này |
|---|---|---|
| `epics.md:1839-1841` *(AC cuối của **chính story này**)* | *"Given lớp HVTĐTD… Then ghi rõ © Đặng Thế Kiệt"* | Thay bằng **AC9** ở trên — neo vào cơ chế, nghiệm thu bằng fixture |
| `epics.md:6202-6204` *(**Story 10.4**)* | AC **nguyên văn giống hệt** | 🔴 **Cùng số phận** — 10.4 phải neo lại y hệt, đừng đi đo lại từ đầu |
| `deferred-work.md:292` | *"nghĩa vụ thông báo tác giả HVTĐTD khi công cụ hoàn thành — chủ: Story 10.4"* | **Mất điều kiện kích hoạt** *(không đóng gói dữ liệu ⇒ không có phép sử dụng để thực hiện)*. Đóng hay giữ là quyết định của **10.4**, không của story này |
| `mockups/sources-attribution.html` | Vẽ HVTĐTD như một hàng **có thật**, và cả §3 *"Gỡ lớp thì chuyện gì xảy ra"* dựng trên đúng nguồn đó | Hàng thứ **ba** của mockup đã lệch dữ liệu *(sau HVTĐTD/Cổ hán văn thừa và Trần Văn Chánh thiếu)*. Bố cục vẫn dùng được; **dữ liệu thì không** |

⚠️ Và một hệ quả **không** hiển nhiên: mệnh đề bán hàng của cả AD-10 — *"nguồn duy nhất có
từ loại, ví dụ và trích dẫn **bằng tiếng Việt**"* — mất nguồn thực hiện nó. Lớp gỡ rời **vẫn**
có ba tệp thật *(`thieu-chuu` · `tran-van-chanh` · `vietphrase`)*, nên **FR36 và AD-10 không
lung lay**; chỉ ví dụ minh hoạ đẹp nhất của chúng biến mất. Đừng đọc quyết định này thành
*"kiến trúc lớp gỡ rời không còn cần thiết"*.

---

## Câu hỏi để lại cho Ice *(không chặn Task 1 trở đi, nhưng nên trả lời sớm)*

1. **Bảng Attribution của mockup lệch dữ liệu thật.** Nó liệt HVTĐTD/Cổ hán văn *(chưa dựng)*
   và bỏ Trần Văn Chánh *(đã dựng, `copyrighted`)*. Story này dựng màn hình **dẫn xuất từ dữ
   liệu**, nên lệch tự biến mất — nhưng `prd.md §8.2` và `docs/dics/README.md` **vẫn** ghi
   TVC ở nhóm *"đã loại"*. `deferred-work.md:581` đang treo việc đồng bộ đó cho *"lượt quy
   hoạch kế tiếp hoặc Story 10.4"*. Có muốn đóng nó **ở đây** không?
2. **Bốn nguồn `dict-core.db` cùng mang `license_text` 43.304 ký tự giống hệt nhau** *(cùng
   CC-BY-SA-4.0)*. Màn hình có nên gộp *"bốn nguồn này dùng chung một giấy phép"* hay giữ
   bốn hàng riêng? *(Mặc định đề xuất: **bốn hàng riêng** — AD-19 nói không hợp nhất, và ghi
   công CC-BY-SA đòi nêu đích danh từng tác phẩm.)*
3. **`tran-van-chanh` mang cảnh báo pháp lý trong `attribution`.** Cảnh báo đó có nên hiện
   **nổi bật hơn** một hàng bảng bình thường không? *(Mockup có cột "Ghi chú xuất xứ" cho
   đúng việc này — nhưng cột đó đã thuộc **10.4** theo Quyết định #4a đã chốt.)*
4. **Chỗ giữ `author-grant` có nên tồn tại không, khi HVTĐTD đã mất nguồn?** Mình giữ nó vì
   nó **rẻ** *(một hàng trong bảng ánh xạ `vi.json`)* và vì nó là **bằng chứng chạy được**
   cho mệnh đề AD-10 *"trường giấy phép không phải enum các giấy phép mở"* — thứ mà bốn giá
   trị thật hiện có *(`open`/`public-domain`/`copyrighted`/`unknown`)* **không** chứng minh
   nổi một mình, vì chưa cái nào là *"phép riêng do tác giả cấp"*. Nếu Ice muốn **bỏ hẳn**,
   AC9 vẫn đứng bằng nhánh mặc định — chỉ mất ca đối chứng dương.

---

## Dev Agent Record

### Agent Model Used

`claude-opus-5` (Claude Code), 2026-08-10.

### Debug Log References

#### Baseline — đo THẬT trước dòng mã đầu tiên (Task 0)

| lệnh | trước story | sau story |
|---|---|---|
| `cargo test` | **232** ca xanh | **250** ca xanh *(+18)* |
| `npm run build` | exit 0 | exit 0 |
| 7 cổng `check:*` | 7/7 PASS | 7/7 PASS |

#### AC12 — NFR1 ĐO LẠI với bộ lọc

`--release` · **bốn tệp `.db` THẬT** (`tools/dict-build/out/`, 4 lớp · **10 nguồn**) · **130
truy vấn KHÁC NHAU** *(cửa sổ trượt 1–3 ký tự trên văn xuôi Hán + 10 truy vấn tiếng Anh)* ·
đường sản phẩm `commands::dict::lookup` · **hai lượt đo độc lập** cho mỗi cấu hình *(Bẫy 8 của
Story 1.18 — nhiễu page-cache của lượt đầu)*.

Lệnh tái lập:

```sh
AURA_DICT_BENCH_DIR=<repo>/tools/dict-build/out \
  cargo test --release --manifest-path src-tauri/Cargo.toml --test dict_sources \
  -- --ignored --nocapture bench_the_source_filter
```

| cấu hình | lượt | p50 ms | **p95 ms** | p99 ms | max ms | chạm trần `LIMIT` | nhóm TB |
|---|---|---|---|---|---|---|---|
| **0 nguồn tắt** | 1 | 3,826 | **10,177** | 34,822 | 53,865 | **1,8 %** | 3,20 |
| **0 nguồn tắt** | 2 | 1,103 | **1,946** | 6,003 | 20,242 | **1,8 %** | 3,20 |
| **1 nguồn tắt** | 1 | 1,126 | **1,996** | 5,425 | 19,115 | **1,8 %** | 2,72 |
| **1 nguồn tắt** | 2 | 1,359 | **2,246** | 6,704 | 26,107 | **1,8 %** | 2,72 |
| **9/10 nguồn tắt** | 1 | 0,695 | **3,891** | 21,113 | **150,628** | **4,8 %** | 0,61 |
| **9/10 nguồn tắt** | 2 | 0,711 | **2,369** | 18,292 | 48,856 | **4,8 %** | 0,61 |

✅ **Cả ba cấu hình dưới trần đầu-cuối 100 ms của NFR1** — p95 cao nhất là **10,177 ms** (lượt
1, page-cache nguội), tức **dưới trần gần 10 lần**; ở lượt 2 cả ba nằm trong **1,9–2,4 ms**.

**Ca xấu nhất mà Story 1.17 tìm được (`"山"`, mốc so sánh p95 6,535 ms), 120 lượt:**

| cấu hình | p50 | **p95** | p99 | max |
|---|---|---|---|---|
| 0 nguồn tắt | 1,071 | **1,544** | 1,903 | 1,930 |
| 1 nguồn tắt | 0,906 | **1,204** | 1,508 | 1,554 |
| 9/10 nguồn tắt | 0,395 | **0,584** | 0,693 | 0,829 |

🔴 **Tỉ lệ chạm trần `LIMIT` — và đọc nó cho đúng.** **1,8 % → 1,8 % → 4,8 %**. Con số **KHÔNG
ĐỔI** khi tắt một nguồn, đúng như §Quyết định #2a tiên liệu: trần chạy **TRƯỚC** phép lọc, nên
các nguồn còn lại giữ **nguyên** tập đầu mục. Nó tăng ở cấu hình 9/10 vì lượt `Exact` rỗng
nhiều hơn ⇒ **đường lui `Substring` của Story 1.18** chạy nhiều hơn, và chính nó mới là thứ
chạm trần — **không phải bộ lọc**. ⇒ §Quyết định #2b *(lọc thẳng trong SQL)* ghi thành **món
nợ có số** ở `deferred-work.md`, kèm nguyên nhân, để lượt sau không đi sửa nhầm chỗ.

⚠️ **`max` 150,628 ms ở lượt 1 của cấu hình 9/10 VƯỢT 100 ms** — ghi ra chứ không làm tròn
xuống. NFR1 phát biểu trên **p95**, và p95 của chính cấu hình đó là 3,891 / 2,369 ms. Lượt 2
cho `max` 48,856 ms. Đây là nhiễu page-cache của lượt đầu (Bẫy 8); không kết luận trên một lượt.

#### Số đo mở màn — xác nhận lại BẰNG MÁY trên đường sản phẩm

Bàn đo in bảng 10 nguồn qua chính `list_source_attributions` *(đối chứng dương của AC1/AC7)*:
**4 giá trị `license_kind`** — `open` (7) · `public-domain` (1) · `copyrighted` (1) ·
`unknown` (1); **2 nguồn `license_id = NULL`** (`tran-van-chanh` · `vietphrase`);
`len(license_text)` từ **1.786** tới **43.304**. **0** nguồn `author-grant`.

#### Đỏ-rồi-xanh — sáu phép, mỗi phép có đối chứng âm

**Ba chỗ lọc phía Rust** *(gỡ từng chỗ, chạy `--test dict_sources`, rồi khôi phục)*:

| gỡ gì | ca đỏ | khôi phục |
|---|---|---|
| lọc ở vòng `hits` của `lookup_grouped` | **6 ca đỏ** *(`disabling_one_source…`, `the_filter_holds_at_the_limit_ceiling…`, `two_sources_in_one_file…`, `turning_every_source_off…`, `the_substring_fallback…`, `a_disabled_source_still_appears…`)* | 75 xanh |
| lọc ở `count_by_source` **(Bẫy 2)** | **1 ca đỏ** — `the_filter_holds_at_the_limit_ceiling_and_never_shrinks_the_survivors` | 75 xanh |
| lọc ở `lookup_han_viet` **(§QĐ #3a)** | **2 ca đỏ** — `disabling_a_detachable_source_changes_the_reading…`, `turning_off_every_reading_source…` | 75 xanh |

🔴 Chỗ thứ hai đáng chú ý: **chỉ MỘT** ca bắt được nó, và ca đó phải **ép `truncated = true`**.
Đúng như Bẫy 2 đã cảnh báo — một bản quên lọc ở `count_by_source` chạy đúng trong mọi test nhỏ.

**Ba cổng** *(sửa tạm rồi khôi phục)*:

| cổng | cách làm đỏ | kết quả |
|---|---|---|
| `check:commands` Kiểm A | đổi `@click="dispatch('attribution.close')"` thành `@click="closeAttribution()"` | `FAIL … @click không phải một lời gọi dispatch('<id>')` → khôi phục → PASS |
| `check:commands` `SELECTION_SURFACE_FLOOR` | comment dòng `useSelectionSurface(panel, 'display')` | `FAIL lời gọi đăng ký vùng chọn quét được — 5 (sàn 6)` → khôi phục → PASS |
| `check:i18n` Kiểm A/A2 | thay `{{ t('attribution.title') }}` bằng chữ tiếng Việt | `FAIL … chuỗi tiếng Việt ở vị trí mã` + `FAIL … văn bản KHÔNG đi qua t()` → khôi phục → PASS |

**Cổng Rust mới** *(`tests/dict_boundary.rs`)*: thêm `'tran-van-chanh'` vào bảng ánh xạ giấy
phép ở `dictSourcesState.ts` ⇒ `1 chỗ trong cây webview viết cứng một danh tính nguồn:
panels/dictSourcesState.ts:177` → khôi phục → 14 xanh.

#### Sàn đã nâng, kèm số THẬT

| hằng | trước | sau | số thật hôm nay |
|---|---|---|---|
| `check-commands.mjs::TS_FLOOR` | 21 | **23** | 28 tệp `.ts` |
| `check-commands.mjs::COMMAND_FLOOR` | 18 | **21** | **25** command |
| `check-commands.mjs::CLICK_FLOOR` | 7 | **9** | 11 thuộc tính `@click` |
| `check-commands.mjs::DISPATCH_FLOOR` | 11 | **13** | 16 lời gọi `dispatch()` |
| `check-commands.mjs::SELECTION_SURFACE_FLOOR` | 5 | **6** | **6** bề mặt *(= số thật)* |
| `check-i18n.mjs::VUE_FLOOR` | 11 | **12** | 14 tệp `.vue` |
| `check-tokens.mjs::FILE_FLOOR` | 34 | **37** | 45 tệp |
| `check-tokens.mjs::COMPONENT_FILE_FLOOR` | 32 | **35** | 42 tệp component |
| `check-layout.mjs::FILE_FLOOR` | 32 | **35** | 42 tệp `src/**` |

*(`check-i18n.mjs::RS_FLOOR = 34` và `check-dict-build.mjs::RS_FILE_FLOOR = 24` **không** bị
vượt — số thật vẫn 40 và 24. Không đụng.)*

### Completion Notes List

#### Năm quyết định — Ice chốt 2026-08-10

| # | chốt | ghi chú |
|---|---|---|
| **cây bẩn** | **commit riêng lượt 1.18b trước** | baseline **THẬT** = `030890f`; `baseline_commit: 51132cb` trong frontmatter **giữ nguyên** theo luật workflow |
| **#1** | **(a)** khoá mới của `ScopeKind::AppConfig`, tầng Global | `KEY_DICT_DISABLED = "dict_sources_disabled"` — khoá **thứ tư** của cùng cửa `theme`/`mode`/`workspace_layout`. **0** `ScopeKind` mới |
| **#2** | **(a)** ở **Rust**, tầng gom, `disabled` là **tham số** từ chỗ gọi | áp **sau** `layer.source(code)`, **trước** khi dựng nhóm; `count_by_source` nhận **cùng** tập |
| **#3** | **(a)** **CÓ** áp cho tab Hán Việt | `lookup_han_viet` nhận cùng tham số; hệ quả đổi-âm đã **đo và ghi ra** |
| **#4** | **(a)** *(đã chốt 2026-08-08)* | bề mặt tối thiểu đủ ba AC; 10.4 giữ nửa phải của bảng chia đôi |
| **#5** | **(a)** kiểu **RIÊNG**, command **RIÊNG**, đọc **một lần** | `SourceInfo` giữ **nguyên** hai trường; `license_text` **không** lên dây, chỉ độ dài |

#### Mười ba AC — trạng thái, và giới hạn của từng cái

- **AC1** ✅ — dải chip dẫn xuất từ `list_dict_sources`; `tests/dict_boundary.rs::the_webview_and_the_string_catalog_hardcode_no_source_identity` khẳng định **0** trên mười `code` thật trong toàn cây `src/**` **và** `vi.json` *(đỏ-rồi-xanh ở trên)*. Vế *"thêm một tệp `.db` ⇒ chip mới xuất hiện, không sửa một dòng mã"* nghiệm thu bằng fixture ở `the_author_grant_placeholder_lands_and_leaves_with_its_file`.
- **AC2** ✅ *(vế Rust)* / ⚠️ *(vế DOM)* — tra lại đi **qua** `runLookup`, không quanh nó; đối chứng âm *"bật lại ⇒ kết quả giống hệt trước khi tắt"* có test. Vế *"chip phân biệt được bằng mắt, không `opacity`"* cài bằng **màu + `line-through`** và nghiệm thu bằng **bàn đo chạy tay** — xem `deferred-work.md`.
- **AC3** ✅ — `disabling_one_source_leaves_every_other_group_untouched` so **cả cấu trúc** nhóm còn lại, không chỉ đếm. Ca **trần `LIMIT`** có riêng và nó là ca duy nhất bắt được Bẫy 2.
- **AC4** ✅ — `dict_boundary.rs` canh bằng máy **hai** mệnh đề: `core/dict/**` không gõ tên một `code` nào, và không đọc `Store` một lời nào.
- **AC5** ✅ — tầng Global; giá trị là tập **BỊ TẮT**; `code` mồ côi bỏ qua **im lặng** (`a_disabled_code_with_no_file_behind_it_is_ignored_in_silence`); đối chứng *"nguồn mới mặc định BẬT"* ở `the_stored_shape_is_the_disabled_set…`.
- **AC6** ✅ *(vế trạng thái)* / ⚠️ *(vế `viwiktionary-en`)* — `panel.lookup.all_sources_off` là chuỗi **thứ năm**, đứng **trước** `not_found` trong chuỗi ưu tiên; `layers_loaded` ở lại `true` (có test). 🔴 **Giới hạn:** vị từ hỏi *"toàn tập có còn nguồn nào bật không"*, **không** hỏi theo **đường đang tra** — tắt riêng `viwiktionary-en` vẫn cho ra câu *"không tìm thấy"* sai. Lý do và đường sửa ghi đầy đủ ở `deferred-work.md`.
- **AC7** ✅ — bảng liệt kê **mọi** nguồn của **mọi** tệp *(hôm nay 10 nguồn / 4 tệp, xác nhận bằng máy ở bàn đo)*; ghi công **nguyên văn, không `ellipsis`** (`vertical-align: top`, không `nowrap`); `license_id = NULL` ⇒ **không** ô trống.
- **AC8** ✅ — `deleting_a_file_removes_its_whole_attribution_block_and_leaves_no_orphan`, kèm **đối chứng dương** trước khi xoá và một lượt chạy lại `the_layer_independent_lookups_still_hold` *(phép thử của AD-10 cho FR36)*.
- **AC9** ✅ *(cơ chế)* / 🔴 **CHỖ GIỮ, KHÔNG PHẢI TÍNH NĂNG ĐÃ CHẠY TRÊN DỮ LIỆU THẬT** — bốn `license_kind` thật ra đúng câu; hai `license_id = NULL` đọc câu của `license_kind`; một `license_kind` **bịa ra** đi qua **nguyên văn**, không panic, và ra nhánh mặc định `attribution.license_unrecognised`. Ca `author-grant` nghiệm thu bằng **fixture**. ⚠️ **0** nguồn thật nào mang `author-grant` hôm nay, **và HVTĐTD sẽ không tới** *(Ice chốt 2026-08-08)*. **Đừng đánh dấu AC9 "đạt trên dữ liệu thật".** Danh tính tác giả đọc từ `dict_source.attribution` của chính tệp — canh bằng máy.
- **AC10** ✅ — `a_disabled_source_still_appears_in_full_in_the_attribution_table` khẳng định **cả hai vế** trong một ca: nguồn biến mất khỏi **kết quả** mà vẫn có mặt đầy đủ trong **bảng ghi công**. Câu *"tắt ≠ gỡ"* + FR112 hiện trên màn hình (`attribution.off_is_not_removed`).
- **AC11** ✅ *(vế khai báo)* / ⚠️ *(vế DOM)* — **ba** command tĩnh, **0** `@click` làm gì khác ngoài một `dispatch` *(Kiểm A, đỏ-rồi-xanh)*. `Escape` đóng lớp phủ và trả tiêu điểm. **Điểm dừng `Tab` mới, khai ra thành số: `10 chip nguồn` + `1 nút Nguồn dữ liệu` + `1 nút đóng lớp phủ` = 12** *(số chip = số nguồn có mặt, nên nó là **12 với bốn tệp `.db` thật** và **2 với 0 lớp gắn**)*. Vế duyệt-hết-bằng-bàn-phím nghiệm thu tay.
- **AC12** ✅ — bảng đầy đủ ở §Debug Log References. Cả ba cấu hình dưới trần; tỉ lệ chạm trần ghi kèm **nguyên nhân**.
- **AC13** ✅ — `cargo test` **232 → 250**; `npm run build` exit 0; **7/7** cổng PASS; **9** hằng `*_FLOOR` bị vượt đã nâng kèm số thật; **bốn** phép đỏ-rồi-xanh trên cổng *(vượt sàn "ít nhất hai")* cộng **ba** trên đường Rust.

#### Ba chỗ lệch so với đặc tả — ghi ra, không giấu

1. 🔴 **BA command tĩnh, không HAI** *(§KHÔNG-LÀM ⑤ kê "đúng hai")*. `attribution.close` là command thứ ba, và lý do là một **ràng buộc của cổng**, không một tính năng thêm: Kiểm A của `check:commands` đòi **mọi** `@click` là đúng một `dispatch('<id>')`, nên một **nút đóng** — thứ AC11 cần cho người dùng chuột — **không tồn tại được** nếu không có command của nó. Ba lý do mà §KHÔNG-LÀM ⑤ đưa ra vẫn đứng nguyên: id **tĩnh**, đếm được bằng `COMMAND_FLOOR`, gán lại được ở Story 1.21. *(`Escape` thì **không** đi qua registry — chiếm `Escape` cho cả ứng dụng là một quyết định khác hẳn.)*
2. 🔴 **`⌥1…6` cho từng nguồn — BÁC**, đúng như §KHÔNG-LÀM ⑤ đã kê, ba lý do đo được. Đây là lệch so với `mockups/sources-attribution.html:140`.
3. ⚠️ **Dải chip nằm NGOÀI `.lookup-head`, không TRONG nó.** Bẫy 4 nói *"nếu nó không vừa, **nói ra và đo**, đừng nới hằng trong im lặng"* — đo: đầu mục 24px/1.3 ≈ 31px + `margin-top` 7px + thanh nhịp ≈ 15px + `padding-bottom` ⇒ **76px đã đầy**. ⇒ một hàng **riêng**, `flex: none`, đứng **trên** vùng đầu mục — đúng thứ tự mockup vẽ (`.chips` trước `.hw`). `--lookup-head-height` giữ **nguyên** giá trị **và** vai trò; `overflow: hidden` ở lại.

#### Hai sửa ngoài dự kiến, cả hai là điều kiện để cổng đứng được

- `tests/dict_boundary.rs` mọc một **bộ che chú thích** (`mask_comments`): phép lọc theo dòng đầu (`starts_with("//")`) **không** che được một khối `<!-- … -->` nhiều dòng, nên cổng mới đỏ trên chính câu **giải thích luật nó canh** — đúng đường ngắn nhất tới việc cổng bị gỡ mà cổng `LIKE` đã ghi bằng chữ. Giới hạn của bộ che *(không theo dõi chuỗi ⇒ `//` trong một URL che nốt dòng)* ghi ngay tại chỗ; hướng lệch là **âm tính giả**.
- `tests/dict_sources.rs` mọc **hai vỏ mang đúng tên hàm thật** (`lookup_grouped`/`lookup_han_viet`) cộng hai vỏ command. Bốn mươi ca có trước story hỏi những câu **không liên quan** tới bộ lọc; rải `&BTreeSet::new()` vào cuối từng lời gọi làm chúng khó đọc hơn mà không canh thêm gì. Ca **về** bộ lọc gọi thẳng đường đầy đủ, và khác biệt đó đọc được ngay tại chỗ gọi.

#### Món nợ đã mở (`deferred-work.md` §1-19) — bảy mục

vế DOM chưa có bộ chạy test · `AttributionOverlay.vue` chưa đo trên WKWebView · `viwiktionary-en`
là nguồn duy nhất của đường `en` *(AC6 chưa hỏi theo `route`)* · §QĐ #2b thành **món nợ có số**
kèm nguyên nhân · `max` 150,628 ms của một lượt đo · số đo NFR1 vẫn là **đường Rust**, **và**
lượt đọc `global.db` mỗi lượt tra **chưa ai đo** · `prd.md §8.2` vs `tran-van-chanh` · chỗ giữ
`author-grant` chưa từng chạy trên dữ liệu thật.

### File List

**Mới (2):**

- `src/AttributionOverlay.vue`
- `src/panels/dictSourcesState.ts`

**Sửa (25):**

- `src-tauri/src/core/dict/mod.rs` — `SourceAttribution` · `list_source_attributions` · `disabled` cho `lookup_grouped`/`lookup_han_viet` *(3 chỗ lọc)*
- `src-tauri/src/core/dict/layer.rs` — `DictLayer::attributions()` *(đọc lúc gọi, `length(license_text)`)*
- `src-tauri/src/core/scope/store.rs` — `KEY_DICT_DISABLED` · `parse_disabled_sources` · hai getter
- `src-tauri/src/core/scope/mod.rs` — re-export `parse_disabled_sources`
- `src-tauri/src/commands/dict.rs` — `disabled_sources()` · `list_sources()` · ba vỏ `wire`
- `src-tauri/src/commands/config.rs` — `BootstrapConfig` trường **thứ sáu**
- `src-tauri/src/lib.rs` — đăng ký `list_dict_sources`
- `src-tauri/tests/dict_sources.rs` — **15** ca mới + bàn đo AC12 + `set_license` + bốn vỏ
- `src-tauri/tests/dict_boundary.rs` — **3** cổng mới + `mask_comments` + `walk_any`
- `src-tauri/tests/ipc_contract.rs` — trường thứ sáu *(Bẫy 1, sửa **cùng lượt**)*
- `src/config/dict.ts` — `SourceAttribution` · `listDictSources()`
- `src/config/bootstrap.ts` — `dict_sources_disabled` · `KEY_DICT_DISABLED`
- `src/panels/LookupPanel.vue` — dải chip · trạng thái AC6 · kiểu dáng
- `src/panels/sourcePanelState.ts` — `refreshHanViet()` *(§QĐ #3a hệ quả 2)*
- `src/commands/index.ts` — ba dep + ba command tĩnh
- `src/main.ts` — nối ba handler + `loadDictSources()`
- `src/App.vue` — gắn lớp phủ
- `src/i18n/vi.json` — **25** khoá mới *(80 → 105, đếm bằng máy ở code review 2026-08-10; bản đầu khai 21, và số thật lúc đó đã là 23)*
- `scripts/check-commands.mjs` · `scripts/check-i18n.mjs` · `scripts/check-tokens.mjs` · `scripts/check-layout.mjs` — nâng **9** sàn kèm số thật
- `src/panels/README.md` — hàng 1.19 + §Bật tắt nguồn từ điển
- `_bmad-output/implementation-artifacts/deferred-work.md` — §1-19, **8** món nợ
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — `in-progress` → `review`

### Review Findings

*Lượt code review 2026-08-10, ba lớp (Blind Hunter · Edge Case Hunter · Acceptance Auditor) chạy độc lập trên diff `030890f..worktree`. Mọi mức nghiêm trọng dưới đây do lượt triage chấm lại sau khi ĐỌC MÃ tại chỗ, không lấy mức của subagent.*

**Kết quả: 4 `decision` (Ice chốt cả bốn) + 7 `patch` — ĐÃ ÁP HẾT 11. 2 `defer`, 0 loại là nhiễu.**
Số sau lượt vá: `cargo test` **250 → 251** · `npm run build` exit 0 · **8/8** cổng PASS · `SCHEMA_VERSION` **2 → 3** · bốn tệp `.db` dựng lại, **ba lượt dựng độc lập cho cùng bốn `sha256`**.

🔴 **Ba phát hiện `high`, và cả ba nằm ở chỗ không phép thử nào của story chạm tới:**
① lượt tra lại sau toggle đọc tập bị tắt **CŨ** *(AC2 vỡ ở cả Panel Lookup lẫn tab Hán Việt —
test Rust truyền `disabled` làm tham số nên không bao giờ thấy thứ tự phía webview, và vế DOM
thì chính story đã khai là chưa có bộ chạy test)*; ② cổng canh AC1 trên `src-tauri/src/**` dùng
một danh sách **chín** mã lỗi thời **thiếu đúng `tran-van-chanh` và `en-wiktionary-vi`**
*(chứng minh bằng phép cấy lỗi: trước khi vá cho 14/14 xanh, sau khi vá đỏ đúng chỗ)*;
③ lớp phủ khai `aria-modal="true"` mà **không bẫy tiêu điểm**, và khi tiêu điểm ra ngoài thì
`Escape` không còn tới được handler — một cái bẫy thật, không một hộp thoại.

⚠️ **Hai lượt đếm sai trong chính tài liệu story, sửa cùng lượt:** "7 món nợ" *(thật: 8)* và
"21 khoá `vi.json` mới" *(thật: 23 lúc đó, 25 sau lượt vá)*.

- [x] [Review][Decision] **AC6 hỏi TOÀN TẬP thay vì theo ĐƯỜNG ĐANG TRA** — `everySourceIsOff` (`dictSourcesState.ts:70`) trả `false` khi bảy nguồn tiếng Trung còn bật, nên tắt riêng `viwiktionary-en` khiến mọi truy vấn tiếng Anh hiện `panel.lookup.not_found` *("không tìm thấy trong từ điển")*, một câu **SAI** vì hệ thống không hề tra. Đây đúng mệnh đề Given của AC6, không một biến thể ngoài phạm vi. Story đã tự khai và ghi vào `deferred-work.md`, lý do kỹ thuật đứng vững *(webview không biết `code → lang`, và dựng bảng tra đó là dựng đúng sổ đăng ký AD-44 ① vá A2 cấm)*. Cần Ice chốt: nhận giới hạn, hay để Rust trả thêm một trường `lang` trên `SourceAttribution` để vị từ hỏi được câu đúng.
- [x] [Review][Decision] **Tab Hán Việt không có trạng thái "mọi nguồn đều tắt"** — §QĐ #3a áp bộ lọc cho cả đường Hán Việt, nhưng khi mọi nguồn tắt thì mọi ký tự rơi về `READING_PLACEHOLDER` (`SourceHanViet.vue:189,289`), **không phân biệt được** với ca *"ký tự này thật sự không có âm ghi nhận"*. Panel Lookup đã có nhánh riêng cho đúng ca này; tab Hán Việt thì không. Cần chốt: thêm một câu riêng cho tab, hay nhận sự bất đối xứng.
- [x] [Review][Decision] **`dictSourcesError` xuất ra nhưng 0 nơi tiêu thụ** — `dictSourcesState.ts:44` xuất kênh lỗi, `:117` gán giá trị, và `grep` toàn `src/` cho thấy **không một** bề mặt nào đọc nó. `loadDictSources` chỉ gọi một lần lúc khởi động, không retry ⇒ một lượt `list_dict_sources` trượt để dải chip **vĩnh viễn** không xuất hiện, và bảng Attribution nói `attribution.empty` *("Chưa gắn lớp từ điển nào")* — một câu SAI vì nguyên nhân thật là lỗi tải. Cần chốt: hiện lỗi *(cần một khoá `vi.json` mới + chỗ đặt)*, hay gỡ kênh chết đi.
- [x] [Review][Decision] **Lớp phủ khai `aria-modal` nhưng không chặn command toàn cục, và `returnFocusTo` có thể trỏ node đã gỡ** — `attachKeyboard(window)` (`main.ts:225`) không hỏi `attributionIsOpen`, nên một hợp âm đổi preset bố cục vẫn `dispatch` được trong lúc lớp phủ mở, gọi `api.clear()` dựng lại panel bên dưới. Hệ quả trực tiếp: `returnFocusTo` (`AttributionOverlay.vue:56-69`) giữ tham chiếu tới node CŨ đã rời DOM, và `returnFocusTo?.focus()` lúc đóng là một lượt gọi **không tác dụng** — vỡ đúng UX-DR17 mà đoạn mã đó tồn tại để đáp ứng. Cần chốt: command nào được phép xuyên qua một lớp phủ modal.

- [x] [Review][Patch] Lượt tra lại sau toggle đọc tập bị tắt CŨ — `runLookup`/`refreshHanViet` phát TRƯỚC `putConfig`, mà Rust đọc tập bị tắt từ `Store` ở đầu mỗi command [src/panels/dictSourcesState.ts:147]
- [x] [Review][Patch] Cổng quét `src-tauri/src/**` dùng `SOURCE_CODES[9]` lỗi thời — thiếu `en-wiktionary-vi` và `tran-van-chanh`, thừa mã ma `hvtdtd` [src-tauri/tests/dict_boundary.rs:388]
- [x] [Review][Patch] `AttributionOverlay` khai `aria-modal="true"` nhưng không bẫy tiêu điểm — Tab rời hộp thoại vào nền đang bị `.attr-scrim` che [src/AttributionOverlay.vue:87]
- [x] [Review][Patch] `toggleFocusedDictSource` ưu tiên tiêu điểm DOM hơn chip vừa bấm chuột — toggle SAI nguồn trên WKWebView [src/panels/dictSourcesState.ts:263]
- [x] [Review][Patch] `refreshHanViet` không có số thứ tự lượt — hai lượt `read_han_viet` bay song song, lượt cũ đè lượt mới [src/panels/sourcePanelState.ts:273]
- [x] [Review][Patch] Hai lượt `put_config` liên tiếp không đảm bảo thứ tự ghi — đĩa có thể giữ ảnh chụp CŨ [src/panels/dictSourcesState.ts:150]
- [x] [Review][Patch] Change Log khai "**7** món nợ mở", `deferred-work.md` §1-19 thực tế có **8** mục [_bmad-output/implementation-artifacts/1-19-bat-tat-nguon-tu-dien-va-ghi-cong.md:40]

- [x] [Review][Patch] ~~Defer~~ **NÂNG LÊN `patch` VÀ ĐÃ VÁ 2026-08-10** — dải chip `overflow: hidden` cắt mất nút *"Nguồn dữ liệu"* *(đường chuột DUY NHẤT vào AC11)* cùng hai chip cuối, trong đó có **Trần Văn Chánh** (`copyrighted`). Lượt triage xếp `low`/`defer` vì tin chú thích CSS; Ice chạy app thật và ảnh chụp bác lại. Vùng chip nay cuộn riêng, nhãn và nút `flex: none` không bao giờ bị cắt [src/panels/LookupPanel.vue]
- [x] [Review][Defer] `list_source_attributions` không loại trùng `code` giữa các lớp, trong khi UI dùng `code` làm `:key` duy nhất [src-tauri/src/core/dict/mod.rs:940] — deferred, pre-existing; bất biến do phía dựng `.db` giữ, không với tới được bằng dữ liệu hiện có
