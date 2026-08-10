---
baseline_commit: 51132cb
---

# Story 1.19: Bật tắt nguồn từ điển và ghi công

Status: ready-for-dev

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

- [ ] **Task 0 — Chốt năm quyết định + xử lý cây bẩn** *(CHẶN — trước dòng mã đầu tiên)*
  - [ ] Hỏi Ice về bảy tệp chưa commit của 1.18b *(§Bối cảnh git)*; **không tự commit**
  - [x] ✅ **Quyết định #4 — CHỐT (a)** *(Ice, 2026-08-08)*: bảng chia đôi 1.19 / 10.4
  - [x] ✅ **AC9 — CHỐT chỗ giữ** *(Ice, 2026-08-08)*: HVTĐTD không tìm được nguồn ⇒ AC neo
        vào **cơ chế**, không vào một nguồn cụ thể
  - [ ] Chốt Quyết định **#1 · #2 · #3 · #5**, ghi mỗi cái một dòng Change Log kèm **lý do**
        nếu chốt ngược
  - [ ] Đo baseline **thật, không chép**: `cargo test` · `npm run build` · 7 cổng `check:*`

- [ ] **Task 1 — Đọc sáu trường giấy phép từ chính tệp** (AC7, AC9)
  - [ ] Kiểu `SourceAttribution` ở `core/dict/` — `code` · `display_name` · `license_kind` ·
        `license_id` · `attribution` · `source_version` · `source_url` · `layer` · `is_base`
        *(+ độ dài `license_text`, xem Quyết định #5)*
  - [ ] Đường đọc trong `DictLayer` *(một truy vấn, lúc gọi — không giữ thường trực)*
  - [ ] ⚠️ **Không** thêm cột, **không** đổi một dòng DDL nào của `tools/dict-build/src/schema.rs`
  - [ ] `is_base` đọc từ `dict_meta('layer') == "base"`, **không** từ tên tệp *(AD-44 ① vá A2)*

- [ ] **Task 2 — Bộ lọc ở tầng gom** (AC3, AC4)
  - [ ] `lookup_grouped` nhận tập `code` bị tắt, áp **sau** `layer.source(code)`
  - [ ] 🔴 `count_by_source` nhận **cùng** tập — nếu quên, thanh nhịp đếm cả nguồn đã tắt *(Bẫy 2)*
  - [ ] `hidden_sources` chỉ gom nguồn **đang bật**
  - [ ] `lookup_han_viet` nhận cùng tham số *(nếu Quyết định #3 chốt (a))*
  - [ ] ⚠️ **0** lời đọc `Store` bên trong `core/dict/**`

- [ ] **Task 3 — Hai command IPC** (AC1, AC7)
  - [ ] `list_dict_sources` — vỏ `#[tauri::command]` bọc một **hàm thuần** *(khuôn `read_han_viet`)*
  - [ ] `lookup_dictionary` đọc tập bị tắt từ `Store` **ở tầng command**, truyền xuống
  - [ ] ⚠️ `try_state`, **không** `state()` *(`panic = "abort"`)*
  - [ ] Cập nhật `tests/ipc_contract.rs` **cùng lượt** nếu `BootstrapConfig` mọc trường thứ sáu *(Bẫy 1)*

- [ ] **Task 4 — Lưu xuống tầng Global** (AC5)
  - [ ] Khoá mới của `ScopeKind::AppConfig` ở `scope/store.rs`, kèm hằng có tên
  - [ ] Ghi qua `putConfig(SCOPE_APP_CONFIG, …)` — đường đã có, **không** đường thứ hai
  - [ ] Đọc lúc khởi động; `code` không còn tệp ⇒ bỏ qua im lặng *(AC5)*

- [ ] **Task 5 — Dải chip trong Panel Lookup** (AC2, AC6, AC11)
  - [ ] Chip dẫn xuất từ `list_dict_sources`, **không** từ `groups` *(nguồn đang tắt không có
        nhóm nào, nên nó phải hiện được từ danh sách đầy đủ)*
  - [ ] Trạng thái tắt phân biệt **không** bằng `opacity` trên chữ *(UX-DR6)*
  - [ ] Tra lại **qua `runLookup`**, không quanh nó *(§KHÔNG-LÀM ③)*
  - [ ] Chuỗi mới vào `vi.json`; **0** chuỗi trong `.vue`
  - [ ] ⚠️ Vùng đầu mục khoá `height: 76px; overflow: hidden` — dải chip **không được** thêm
        một pixel nào vào đó *(Bẫy 4)*

- [ ] **Task 6 — Bề mặt Attribution** (AC7, AC8, AC9, AC10, AC11)
  - [ ] Lớp phủ mở bằng command `attribution.open`, đóng bằng `Escape`, trả tiêu điểm
  - [ ] Bảng: tên · giấy phép · lớp · ghi công *(nguyên văn, không `ellipsis`)*
  - [ ] Ánh xạ `license_kind` → câu ở `vi.json`, **có nhánh mặc định**
  - [ ] Câu *"tắt ≠ gỡ"* (AC10) và câu FR112
  - [ ] 🔴 `useSelectionSurface(ref, 'display')` cho bề mặt này + nâng `SELECTION_SURFACE_FLOOR`
        lên số thật *(Bẫy 8)*

- [ ] **Task 7 — Test Rust** (AC3, AC4, AC5, AC8, AC9)
  - [ ] `dict_sources.rs`: ca **trần `LIMIT`** của AC3 *(ca đắt nhất — dựng fixture đủ hàng)*
  - [ ] Ca **xoá một tệp rồi chạy lại bộ test tra cứu** *(FR36 / AC8)*
  - [ ] Ca `license_kind` **bịa ra, chưa gặp** ⇒ câu mặc định có nghĩa *(AC9)*
  - [ ] Ca **chỗ giữ `author-grant`**: thả fixture vào ⇒ hiện đủ; xoá ⇒ biến mất, không mồ côi
  - [ ] Ca **danh tính tác giả đọc từ `attribution` của chính tệp** — `grep` khẳng định **0**
        tên tác giả viết cứng trong `src/**` và `vi.json` *(AC9)*
  - [ ] Ca **hai nguồn cùng tệp**, tắt một *(`dict-core.db` mang bảy — đây là hình dạng thật)*
  - [ ] `dict_boundary.rs`: **0** `code` viết cứng trong `core/dict/**`

- [ ] **Task 8 — Đo NFR1** (AC12)
  - [ ] Bản `--release`, bốn tệp thật, ≥ 100 lượt, ba cấu hình bộ lọc
  - [ ] Ghi bảng đầy đủ vào §Debug Log References — **số, không lời khen**
  - [ ] Ghi tỉ lệ chạm trần `LIMIT` trước/sau

- [ ] **Task 9 — Cổng, sàn, tài liệu** (AC13)
  - [ ] Chín lệnh DoD; nâng `*_FLOOR` bị vượt kèm số THẬT
  - [ ] Đỏ-rồi-xanh ≥ 2 cổng, mỗi ca có đối chứng âm
  - [ ] `src/panels/README.md`: hàng 1.19 + §Bật tắt nguồn
  - [ ] `deferred-work.md`: mở §1-19 với món nợ thật *(tối thiểu: WKWebView chưa đo)*

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

*(điền khi dev-story chạy)*

### Debug Log References

### Completion Notes List

### File List

### Review Findings
