---
title: 'Story 5.6: Lưới Tác phẩm, lọc và sắp xếp'
type: 'feature'
created: '2026-08-28'
status: 'blocked'
baseline_revision: '2b837fe1e836649f407385c89f1ff674a7904a13'
review_loop_iteration: 0
followup_review_recommended: false
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
warnings: ['oversized']
deferred: []
---

<intent-contract>

## Intent

**Problem:** Library là điểm vào ứng dụng nhưng hôm nay chỉ có một **danh sách phẳng**: không
bìa, không lưới, chỉ lọc được theo trạng thái (Story 5.4), không sắp xếp được, và cột "ngày
sửa" mà FR10 đòi sắp theo thì **đóng băng ở thời điểm tạo** — `work.updated_at` có đúng một
lượt ghi (`INSERT INTO work`, `commands/project.rs:265`) và **0** câu `UPDATE` toàn kho. Thêm
vào đó khối `.empty` của `LibraryMode.vue` (dòng ~529) khẳng định *"Library chưa có Tác phẩm
nào."* **không điều kiện** — nó nói câu đó kể cả khi lưới ngay bên trên đang hiện mười Tác
phẩm.

**Approach:** Ba việc, đi qua đúng những đường đã có, không dựng cơ chế mới:
① làm `work.updated_at` **sống** bằng cách tính nó tại `WorkMeta::rebuild_from_store` — chỗ
DUY NHẤT tính giá trị dẫn xuất (Story 5.5) — thay vì chép lại cột đóng băng;
② mở rộng `Indexer::list_works` để lọc theo **lĩnh vực** và **ngôn ngữ nguồn** và **sắp xếp**
trong SQL (AD-1: bộ lọc/sắp xếp tính ở Rust, không ở TypeScript), kèm trả về tập giá trị có
thật để giao diện dựng lựa chọn mà không tự suy diễn;
③ đổi danh sách phẳng thành **lưới** có khung bìa + biểu diễn thay thế nhất quán, ba bộ chọn
`<select>` (lĩnh vực · ngôn ngữ · sắp xếp), và gắn điều kiện thật cho câu "Library chưa có
Tác phẩm nào".

## Boundaries & Constraints

**Always:**
- **Lọc và sắp xếp tính trong SQL**, ở `Indexer::list_works` — AD-1. TypeScript không được
  `.filter()`/`.sort()` mảng `works`, và không được tự suy tập giá trị lựa chọn từ mảng đã
  tải về (mảng đó ĐÃ bị lọc; suy từ nó làm lựa chọn biến mất dần theo mỗi lượt lọc).
- **`updated_at` tính ở đúng một chỗ** — `WorkMeta::rebuild_from_store`. Không thành phần nào
  khác được tính nó; `Indexer` vẫn **chép trung thành** `meta.json` (§Boundaries Story 5.2).
- **Khoá sắp xếp là một danh mục ĐÓNG**, phân giải ở Rust (`from_wire` như
  `LifecycleStatus`). Một khoá lạ trên dây ⇒ `IpcError`, **không** im lặng rơi về mặc định.
- Mọi nhãn qua `t()` (NFR16, AD-21); mọi thao tác lọc/sắp/điều hướng lưới làm được bằng bàn
  phím (NFR17); tương phản đạt WCAG AA ở cả hai theme.
- Một màn hình **không được khẳng định điều nó không biết**: câu "chưa có Tác phẩm nào" chỉ
  được nói khi đã tải xong VÀ `total === 0`.

**Block If:**
- Xuất hiện nhu cầu **ghi `meta.json` ở một chỗ thứ ba** (ví dụ sau mỗi lượt flush segment)
  để `updated_at` sống theo thời gian thực. Đó là một **AD mới** (đường ghi tăng dần cho
  `library_work` / chi phí quét lại toàn thư viện mỗi lượt ghi), giao Winston — HALT với
  blocking condition `AD mới: đường ghi tăng dần cho library_work`, soạn hồ sơ bàn giao, KHÔNG
  tự soạn AD.
- Cần **thêm cột `cover`** vào `work`/`library_work` để đạt AC2/AC6 (xem §Design Notes — phép
  đo nói là KHÔNG cần). Nếu phép đo lại cho kết quả khác, HALT với blocking condition
  `cột cover cần chủ quyết định`.
- Bơm `updated_at` vào giao dịch flush làm cổng đang xanh
  `segment_contract.rs::a_flush_touches_exactly_target_text_and_updated_at_and_nothing_else`
  đỏ ⇒ HALT, không sửa cổng đó (nó là một AC đã ký của Story 2.3).

**Never:**
- **Không** thêm cột `cover` và **không** bump `META_SCHEMA_VERSION` cho nó ở lượt này — đo
  2026-08-28: **0** đường sản phẩm nào ĐẶT một ảnh bìa, và **0** story nào trong `epics.md`
  mở đường đó. Xem §Design Notes.
- **Không** đổi `LIBRARY_INDEX_MIGRATIONS` (mọi cột cần cho lọc/sắp đã có) và **không** thêm
  index cho `ORDER BY` ở lượt này — chưa phép đo nào nói nó cần (NFR3–5 nghiệm thu ở Story
  6.18).
- **Không** dựng đường mở lại một `.atproj` đã có trên đĩa, **không** cho đổi trạng thái một
  Tác phẩm không phải Tác phẩm đang mở, **không** danh sách Chương / mở Chương (Story 5.7).
- **Không** `v-for` chọn id `dispatch()` động cho các nút — `check:commands` Kiểm A đòi mỗi
  `@click` là đúng một `dispatch('<id>')` với id **literal**.
- **Không** tìm kiếm full-text, không hai chế độ dấu (Story 5.9/5.10).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Không lọc | `status=[]`, `genre=None`, `source_lang=None` | Mọi hàng, `matched == total` | Không lỗi |
| Lọc lĩnh vực | `genre=Some("Tiên hiệp")` | `WHERE genre = ?`; `matched` ≤ `total` | Không lỗi |
| Lọc ngôn ngữ | `source_lang=Some("zh")` | `WHERE source_lang = ?` | Không lỗi |
| Ba bộ lọc chồng | status ∧ genre ∧ lang | Giao của cả ba (`AND`), không phải hợp | Không lỗi |
| Lĩnh vực không tồn tại | `genre=Some("Không có")` | `matched = 0`, `total` giữ nguyên, câu "không khớp" | Không lỗi — 0 kết quả là kết quả hợp lệ |
| Sắp theo ngày sửa | `sort="updated_desc"` | `ORDER BY updated_at DESC, work_id` | Không lỗi |
| Sắp theo tên | `sort="name_asc"` | `ORDER BY name COLLATE NOCASE, work_id` | Không lỗi |
| Khoá sắp lạ | `sort="bừa"` | — | `IpcError` `library.unknown_sort` `{sort}`; KHÔNG rơi về mặc định |
| Sắp mặc định | `sort=None` | `updated_desc` (ngày sửa gần nhất — FR10 nêu nó trước) | Không lỗi |
| Hai Tác phẩm cùng `updated_at` | trùng mốc | Thứ tự **ổn định** nhờ `, work_id` phụ | Không lỗi |
| Tập lựa chọn | mọi lượt đọc | `genres`/`source_langs` = `DISTINCT` trên **toàn bảng CHƯA lọc**, cùng một lượt `Store::read` | Không lỗi |
| Bìa vắng mặt | mọi hàng hôm nay | Ô bìa vẽ biểu diễn thay thế (chữ cái đầu của tên), **không** ô trống | Không lỗi |
| Tên rỗng/khoảng trắng | `name = ""` | Biểu diễn thay thế dùng ký tự chốt `?`, không chuỗi rỗng | Không lỗi |
| Con trỏ ra ngoài sau lọc | con trỏ ở ô 7, lượt lọc còn 3 ô | Con trỏ kẹp về ô cuối cùng còn lại; luôn có đúng một ô đang chọn | Không lỗi |
| Con trỏ khi danh sách rỗng | `matched = 0` | Không ô nào đang chọn; `next`/`prev` là no-op, không ném | Không lỗi |
| Library rỗng | `total = 0`, đã tải | Khối `.empty` hiện; lưới không hiện | Không lỗi |
| Chưa tải xong | `worksHaveLoaded = false` | **Không** nói "chưa có Tác phẩm nào"; nói "chưa tải" | Không lỗi |
| `updated_at` chưa dựng lại | `meta.json` v1/v2 chưa qua `rebuild_from_store` | Vẫn sắp được (giá trị lúc tạo), lưới ghi rõ đó là mốc tạo | Không lỗi |

</intent-contract>

## Code Map

**Tầng nguồn sự thật — nơi `updated_at` thôi nói dối**

- `src-tauri/src/core/library/meta.rs` — `rebuild_from_store` (§fn, ~dòng 261–365). Câu
  `SELECT work_id, name, source_lang, genre, created_at, updated_at, status_override FROM work`
  (~dòng 272) hôm nay **chép** `updated_at` sang `WorkMeta` (~dòng 357). Đây là chỗ phải đổi:
  bỏ `updated_at` khỏi câu `SELECT` đó, tính nó bằng một câu riêng. `META_SCHEMA_VERSION = 3`
  ở dòng 46 — **không** bump (hình dạng không đổi, chỉ ngữ nghĩa giá trị).
- `src-tauri/src/commands/project.rs:265` — `INSERT INTO work (… updated_at)`, chỗ ghi
  `work.updated_at` **duy nhất**. Đọc để xác nhận, **không sửa**.
- `src-tauri/src/commands/lifecycle.rs:143` — `UPDATE chapter SET status = ?1, updated_at =
  strftime(…)`. **Bằng chứng `chapter.updated_at` ĐANG SỐNG** (Story 5.4 dựng). Không sửa.
- `src-tauri/src/commands/segment.rs:1186` (`save_segment_targets`) và `:709` (khôi phục) —
  hai chỗ bơm `segment.updated_at`. Đọc để biết mốc sửa văn bản sống ở đâu; **không sửa** —
  cổng `a_flush_touches_exactly_target_text_and_updated_at_and_nothing_else` đứng canh.

**Tầng chỉ mục — lọc và sắp**

- `src-tauri/src/core/library/indexer.rs` — `list_works` (~dòng 488–545): hằng `COLUMNS`
  (~494), `map_row` (~497), và ba nhánh `match filter` (~512–540) với `ORDER BY work_id` cứng.
  Đây là nơi dựng `WHERE`/`ORDER BY`. `struct IndexedWork` (~dòng 859) và `struct WorksReport`
  (~dòng 893) là hai hình dạng phải mở rộng (`WorksReport` thêm `genres`/`source_langs`).
  ⚠️ Doc-comment ~dòng 480 ghi rõ `Some(&[])` ở tầng này nghĩa "khớp 0 hàng" — giữ nguyên
  quy ước đó cho bộ lọc trạng thái khi thêm hai bộ lọc mới.
- `src-tauri/src/core/store/schema.rs:1618` — `LIBRARY_WORK_DDL`. **Mọi cột cần đã có**
  (`source_lang`, `genre`, `updated_at`, `status`, `chapter_done_count`). Dòng 1575 mang câu
  *"không `cover` (chủ Story 5.6)"* — mệnh đề đó cần sửa **tại chỗ** kèm 🔵 và ngày (phép đo
  ở §Design Notes), không xoá.

**Tầng dây IPC**

- `src-tauri/src/commands/library.rs` — `struct WorkRow` (~dòng 275, **không đổi**),
  `struct WorkListReport` (~dòng 317, thêm `genres`/`source_langs`), hàm thuần `list_works`
  (~dòng 324–356: nơi `from_wire` chuyển chuỗi thành `LifecycleStatus` — khuôn để chép cho
  khoá sắp xếp), và vỏ `library_list_works` trong `mod wire` (~dòng 358+).
- `src-tauri/src/commands/lifecycle.rs` — `unknown_status` là khuôn `IpcError` để chép cho
  `unknown_sort`.

**Tầng frontend**

- `src/config/library.ts` — `type WorkRow` (~dòng 135) và vị từ kiểm kiểu lúc chạy
  `isWorkRowArray` (~dòng 161–180). Vị từ phải kiểm hai mảng lựa chọn mới.
- `src/modes/libraryRescan.ts` — **KHUÔN CON TRỎ để chép**: `orphanCursor` (dòng 34),
  `currentLibraryOrphan` (dòng 69), kẹp biên sau mỗi lượt tải (dòng 83–84), `next`/`prev`
  (dòng 185/190), đặt lại (dòng 203). Cặp lệnh `library.orphan_next`/`orphan_prev`
  (`src/commands/index.ts:927–941`) là khuôn đăng ký. **Không dựng cơ chế điều hướng thứ hai.**
- `src/modes/libraryWorks.ts` — `statusFilter` (~dòng 45), `loadWorks` (~dòng 87–121, có cơ
  chế `worksReloadPending` chống nuốt lượt tải — **tái dùng, không dựng lại**),
  `toggleStatusFilter` (~134), `clearStatusFilter` (~146), `resetLibraryWorks` (~255).
- `src/modes/LibraryMode.vue` — khối `.works-block` (~dòng 303–455): sáu nút lọc (~318–375),
  dải `role="status"` (~380–390), `<ul class="works-list">` (~395–450, **đổi thành lưới**).
  Khối `.empty` (~dòng 529–532) là chỗ AC5 phải gắn điều kiện. `<select v-model="sourceLang">`
  (~dòng 543) là **tiền lệ có sẵn** cho bộ chọn trong chính tệp này. `<style scoped>` từ ~613.
- `src/commands/index.ts` — cụm lệnh `library.*` (~dòng 950–1010) và các cổng phụ thuộc
  (`deps.*`, `portMissing`). `Mod+Alt+W` đã thuộc `library.list_works`.
- `src/i18n/vi.json` — cụm `mode.library.*` và `command.library.*`. Đã có
  `mode.library.empty_body` (đúng câu AC5 đòi) và `mode.library.status`.

**Cổng và bàn đo**

- `scripts/check-commands.mjs` — Kiểm A chỉ canh `@click`; doc-comment dòng 33 ghi rõ
  `@change`/`@input` **không** thuộc luật. Đây là điều cho phép dùng `<select @change>` cho
  ba bộ chọn tập-mở. Sàn tệp/số `dispatch()` ở ~dòng 201–350 phải nâng nếu số thật đổi.
- `src-tauri/tests/library_index_contract.rs` — hàm dựng `.atproj` thật (~dòng 92–125) và hai
  chuỗi `meta.json` viết tay (~438, ~1325).
- `src-tauri/tests/library_commands_contract.rs` — khuôn ca cho tầng lệnh (~dòng 88).
- `src-tauri/tests/ipc_contract.rs:472–535` — đóng băng khoá `snake_case` của
  `WorkRow`/`WorkListReport`. Thêm khoá mới ⇒ sửa danh sách khẳng định ở đây.
- `src-tauri/tests/meta_write_boundary.rs` — cổng AC4 của Story 5.5 (đúng ba tệp ghi
  `meta.json`, đúng hai tệp đọc). Sửa `meta.rs` **không** được làm nó đỏ.
- `tests/frontend/libraryWorks.test.ts` — `invoke` giả, khuôn ca frontend.
- `e2e/specs/story-5-5-progress.e2e.mjs` — khuôn spec e2e gần nhất; móc
  `[data-lifecycle-filter="…"]`, `.works-block .works-list .works-row`, `realClick`.
  ⚠️ Spec này neo vào `.works-list`/`.works-row` — đổi sang lưới **sẽ làm nó đỏ**; cập nhật
  cùng lượt, đừng để lượt đỏ đó bị đọc thành hồi quy.

## Tasks & Acceptance

**Execution:**

- `src-tauri/src/core/library/meta.rs` -- trong `rebuild_from_store`, bỏ `updated_at` khỏi câu
  `SELECT … FROM work`, tính nó bằng `MAX` trên ba nguồn ĐANG SỐNG (`work.created_at`,
  `MAX(chapter.updated_at)`, `MAX(segment.updated_at)`) và ghi giá trị lớn nhất vào
  `WorkMeta.updated_at`; doc-comment nêu phép đo (0 `UPDATE` trên `work.updated_at`) và nêu
  **dư địa còn lại** (chỉ tươi tới lượt ghi `meta.json` gần nhất) -- để cột "ngày sửa" mà AC4
  sắp theo thôi nói dối, mà không cần một chỗ ghi `meta.json` thứ ba.
- `src-tauri/src/core/library/indexer.rs` -- `list_works` nhận một `WorkQuery` (bộ lọc trạng
  thái đã có + `genre` + `source_lang` + khoá sắp), dựng `WHERE` bằng `AND` và `ORDER BY` theo
  khoá, luôn kèm `, work_id` làm khoá phụ; `WorksReport` thêm `genres`/`source_langs` đọc bằng
  `SELECT DISTINCT` trên bảng **chưa lọc** trong **cùng** lượt `Store::read` -- AD-1, và để
  ba con số/ba tập không bao giờ đến từ hai ảnh chụp.
- `src-tauri/src/core/store/schema.rs` -- sửa **tại chỗ** doc-comment dòng ~1575 (*"không
  `cover` (chủ Story 5.6)"*) thành mệnh đề đúng hôm nay kèm 🔵 và ngày, dẫn phép đo ở
  §Design Notes -- một mệnh đề hết đúng để lại trong mã là một cái bẫy cho story sau.
- `src-tauri/src/commands/library.rs` -- `list_works` nhận thêm `genre`/`source_lang`/`sort`
  trên dây, phân giải khoá sắp qua một `from_wire` danh mục ĐÓNG, khoá lạ ⇒ `unknown_sort`
  (chép khuôn `lifecycle::unknown_status`); `WorkListReport` chở `genres`/`source_langs`;
  vỏ `library_list_works` truyền qua, **không quy tắc nào sống trong vỏ**.
- `src/config/library.ts` -- thêm kiểu khoá sắp (union đóng) và hai mảng lựa chọn vào kiểu
  báo cáo; mở rộng vị từ kiểm kiểu lúc chạy để kiểm chúng -- một dây IPC đổi hình mà vị từ
  không kiểm là đúng lớp "rỗng im lặng".
- `src/modes/libraryWorks.ts` -- thêm ref `genreFilter`/`sourceLangFilter`/`sortKey` và hai
  mảng lựa chọn; `loadWorks` gửi cả ba, đi qua **đúng** cơ chế `worksReloadPending` đã có;
  thêm hành động đặt từng thứ; `clearStatusFilter` mở rộng thành bỏ **mọi** bộ lọc;
  thêm `workCursor` + `currentLibraryWork` + `nextWork`/`prevWork` **chép khuôn `orphanCursor`
  của `libraryRescan.ts`**, kể cả lượt **kẹp biên sau mỗi lượt tải** (một lượt lọc làm danh
  sách ngắn đi để con trỏ trỏ ra ngoài mảng ⇒ ô "đang chọn" biến mất im lặng);
  `resetLibraryWorks` đặt lại mọi ref mới.
- `src/commands/index.ts` -- đăng ký lệnh mới cho ba bộ chọn, lệnh bỏ mọi bộ lọc, **và cặp
  `library.work_next`/`library.work_prev`** (chép khuôn `library.orphan_next`/`orphan_prev` ở
  dòng 927–941), kèm `labelKey` và cổng `portMissing` theo đúng khuôn cụm `library.*` đã có
  -- NFR17: mọi thao tác tới được bằng bàn phím, không chỉ bằng chuột.
- `src/modes/LibraryMode.vue` -- đổi `<ul class="works-list">` thành lưới (`.works-grid`) mỗi ô
  có **khung bìa** (biểu diễn thay thế nhất quán: chữ cái đầu của tên trên nền token, `?` khi
  tên rỗng), tên, tiến độ, trạng thái; thêm ba `<select>` (lĩnh vực · ngôn ngữ · sắp xếp) dùng
  `@change` (ngoài luật Kiểm A) với `<option>` dựng từ hai mảng lựa chọn do Rust trả về; gắn
  điều kiện `libraryWorksHaveLoaded && libraryWorksTotal === 0` cho khối `.empty`; đánh dấu ô
  **đang chọn** theo `workCursor` bằng một tín hiệu nhìn thấy được (không chỉ màu — WCAG AA)
  cộng `aria-current`, và cuộn ô đó vào tầm nhìn khi con trỏ chạy ra ngoài khung; thêm móc
  `data-library-*` cho e2e -- AC2/AC3/AC4/AC5/AC6/AC7.
- `src/i18n/vi.json` -- khoá cho ba nhãn bộ chọn, hai nhãn khoá sắp, nhãn "mọi lĩnh vực"/"mọi
  ngôn ngữ", `aria-label` khung bìa, thông điệp `err.library.unknown_sort`, và nhãn lệnh mới.
- `src-tauri/tests/library_index_contract.rs` -- ca cho **mọi** hàng §I/O Matrix thuộc tầng
  chỉ mục: ba bộ lọc riêng rẽ và chồng nhau, hai khoá sắp, thứ tự ổn định khi trùng
  `updated_at`, tập lựa chọn lấy trên bảng chưa lọc, và `updated_at` dẫn xuất đúng `MAX` ba
  nguồn (gồm ca một Chương đổi trạng thái làm mốc tiến lên).
- `src-tauri/tests/library_commands_contract.rs` -- ca khoá sắp lạ ⇒ `library.unknown_sort`
  (KHÔNG rơi về mặc định), ca `sort=None` ⇒ mặc định `updated_desc`.
- `src-tauri/tests/ipc_contract.rs` -- cập nhật khẳng định đóng băng khoá `snake_case` của
  `WorkListReport` cho hai trường mới.
- `tests/frontend/libraryWorks.test.ts` -- ca cho: đặt/bỏ từng bộ lọc gọi lại `loadWorks` với
  tham số đúng; đổi khoá sắp tải lại; `clearStatusFilter` bỏ cả ba; hai mảng lựa chọn không bị
  suy từ mảng `works` đã lọc; con trỏ bị kẹp biên khi lượt lọc làm danh sách ngắn lại, và
  `nextWork`/`prevWork` trên danh sách rỗng là no-op.
- `e2e/specs/story-5-6-library-grid.e2e.mjs` -- spec mới đi trọn đường qua WKWebView: khởi
  động vào Library; lưới hiện ô bìa thay thế; lọc theo lĩnh vực bằng **bàn phím**; chạy con
  trỏ qua các ô bằng phím và khẳng định `aria-current` chuyển đúng ô; đổi khoá sắp và khẳng
  định thứ tự tên đảo đúng; Library rỗng hiện khối giải thích.
- `e2e/specs/story-5-5-progress.e2e.mjs` -- cập nhật bộ chọn neo vào `.works-list`/`.works-row`
  sang móc của lưới mới -- lượt đỏ do đổi bố cục phải được sửa **cùng lượt**, không để nó bị
  đọc nhầm thành hồi quy tiến độ.

**Acceptance Criteria:**

- **AC1** — Given ứng dụng vừa khởi động, when màn hình đầu tiên hiện ra, then chế độ đang
  hoạt động là Library chứ không phải Workspace, và có một ca tự động neo mệnh đề đó (hôm nay
  `modeState.ts:33` đã là `ref('library')` — việc của story này là **neo**, không phải dựng).
- **AC2** — Given chỉ mục có ít nhất một Tác phẩm, when lưới hiện, then mỗi ô mang **bốn** thứ
  cùng lúc: khung bìa, tên, tiến độ, trạng thái.
- **AC3** — Given lưới đang hiện, when người dùng chọn một lĩnh vực và một ngôn ngữ nguồn cùng
  lúc với một bộ lọc trạng thái, then danh sách còn lại là **giao** của cả ba, và phép lọc
  chạy trong SQL (đối chứng: không lời gọi `.filter()` nào trên mảng `works` ở TypeScript).
- **AC4** — Given lưới đang hiện, when người dùng đổi khoá sắp giữa *ngày sửa gần nhất* và
  *tên*, then thứ tự ô đổi theo, và với hai Tác phẩm trùng mốc thời gian thứ tự **không đổi**
  giữa hai lượt tải liên tiếp.
- **AC5** — Given chỉ mục **không** Tác phẩm nào và lượt tải đã xong, when Library hiện, then
  khối giải thích *"Tác phẩm là gì và là một thư mục mang đi được"* hiện ra trước lời mời nhập;
  và given chỉ mục **có** Tác phẩm, when Library hiện, then câu *"Library chưa có Tác phẩm
  nào"* **không** xuất hiện.
- **AC6** — Given một Tác phẩm không có ảnh bìa (hôm nay là **mọi** Tác phẩm), when ô của nó
  hiện, then khung bìa vẽ một biểu diễn thay thế nhất quán, không phải một ô trống.
- **AC7** — Given chỉ dùng bàn phím, when người dùng đặt bộ lọc, đổi khoá sắp, và chạy con
  trỏ qua các ô của lưới, then làm được trọn vẹn cả ba không cần chuột, và ô đang chọn luôn
  nhìn thấy được; và given con trỏ đang ở ô cuối, when một lượt lọc làm danh sách ngắn lại,
  then con trỏ kẹp về ô cuối cùng còn lại chứ không trỏ vào khoảng trống.
- **AC8** — Given một Chương vừa đổi trạng thái, when `meta.json` được dựng lại, then
  `updated_at` của Tác phẩm tiến lên mốc mới (không còn đứng ở thời điểm tạo), và cổng
  `meta_write_boundary.rs` cùng cổng
  `segment_contract.rs::a_flush_touches_exactly_target_text_and_updated_at_and_nothing_else`
  **vẫn xanh**.

## Spec Change Log

## Review Triage Log

## Design Notes

### Ba mệnh đề của sổ nợ đã HẾT ĐÚNG — đo lại trước khi thi hành

Sổ nợ giao Story 5.6 một chồng lớn. Ba mục trong đó dựng trên tiền đề đo **2026-08-27**, và
phép đo lại **2026-08-28** (trên `2b837fe`) cho kết quả khác:

① *"`chapter.updated_at` cũng đóng băng — **0** `UPDATE` toàn cây"* (mục "Deferred from:
5-1-…", vế (2)). **Sai hôm nay:** `commands/lifecycle.rs:143` có
`UPDATE chapter SET status = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now')`, do
**Story 5.4 dựng — SAU lượt đo của sổ nợ**. `chapter.updated_at` đang sống. Đây chính là thứ
mở đường cho ①: `work.updated_at` có thể dẫn xuất từ nó mà **không** cần đụng giao dịch flush,
tức **không** làm đỏ cổng đã ký của Story 2.3 — cái giá mà sổ nợ nêu để chuyển chủ khỏi Story
5.2 **không còn phải trả**.

② *"Xung đột `work_id` trùng … KHÔNG có bề mặt HIỂN THỊ nào (chủ: Story 5.6)"*. **Đã đóng:**
`LibraryMode.vue` có `firstLibraryConflict`/`libraryConflicts` và hai khoá
`mode.library.conflict_warning`/`conflict_more`, dựng ở Story 5.3. Không việc gì cho story này.

③ *"thêm cột `cover` + di trú `META_SCHEMA_VERSION` **1→2**"*. Con số đã cũ —
`META_SCHEMA_VERSION` nay là **3** (5.4 → 2, 5.5 → 3), `library-index.db` ở **5**. Nhưng
phần quan trọng hơn con số là tiền đề: xem mục kế.

### Vì sao KHÔNG thêm cột `cover` ở lượt này

Tiền đề của sổ nợ là *"Story 5.6 là nơi bìa LẦN ĐẦU được NHÌN THẤY"*. Đo lại:
`grep -rni cover src-tauri/src src` ⇒ **3** kết quả, **0** cái nào là một trường dữ liệu (hai
cái là chữ "covering index", một là chính câu cấm ở `schema.rs:1575`). Và `grep -n "bìa"
_bmad-output/planning-artifacts/epics.md` ⇒ **0** story nào mở một đường cho người dùng
**ĐẶT** ảnh bìa — FR3 ghi *"ảnh bìa (tuỳ chọn)"*, không AC nào ở bất kỳ epic nào dựng đường
chọn tệp bìa.

⇒ Thêm cột bây giờ cho một cột **luôn `NULL`** và một giao diện **luôn** vẽ biểu diễn thay
thế. Đó đúng thứ Story 1.7 §Completion Notes #3 cấm, và đúng câu §Never của Story 5.1 đang
cấm. Kết quả người dùng nhìn thấy **giống hệt** giữa hai phương án, nên đây không phải một
lựa chọn giữa hai kết quả khác nhau — nó là lựa chọn giữa "có một cột chết" và "không".

⇒ Story này dựng **khung bìa + biểu diễn thay thế** (AC2/AC6 quan sát được, đo được), và
chuyển món nợ cột `cover` sang chủ mới: **story đầu tiên mở đường cho người dùng ĐẶT một ảnh
bìa** — story đó chưa tồn tại trong `epics.md`, nên mục nợ phải nêu **Ice** là người quyết
định kế tiếp, không phải một tên story giả cho có (`check:debt-owner` Kiểm A).

### `updated_at`: sống tới đâu, và dư địa còn lại phải nói thẳng

Hôm nay `rebuild_from_store` **chép** `work.updated_at` (`meta.rs:272` → `:357`), mà cột đó
có 1 lượt `INSERT` và 0 lượt `UPDATE`. Story này đổi nó thành giá trị **tính**:

```
updated_at = MAX( work.created_at,
                  (SELECT MAX(updated_at) FROM chapter),
                  (SELECT MAX(updated_at) FROM segment) )
```

Ba nguồn đều **đang sống**: `chapter.updated_at` từ `lifecycle.rs:143`, `segment.updated_at`
từ `segment.rs:1186`/`:709`, `work.created_at` làm sàn cho một Tác phẩm chưa có gì.

🔴 **Dư địa còn lại, không được làm tròn lên "đã đóng":** `rebuild_from_store` chỉ chạy ở
**hai** chỗ (`commands/project.rs` sau `create_work`, `commands/lifecycle.rs` sau mỗi lượt đổi
trạng thái Chương — cổng `meta_write_boundary.rs` cưỡng chế đúng ba tệp ghi). Nên sau một loạt
**sửa văn bản thuần** mà không đổi trạng thái Chương nào, `meta.json` không được ghi lại và
`updated_at` **vẫn đứng yên** tới lượt ghi kế tiếp. Câu đúng là *"tươi tới lượt ghi `meta.json`
gần nhất"*, không phải *"thời gian thực"*. Đóng nốt vế đó cần một chỗ ghi `meta.json` **thứ
ba** trên đường flush — một **AD mới** (nó kéo theo `reindex_library` quét toàn thư viện mỗi
lượt auto-save), nên nó nằm ở §Block If, giao Winston, không tự quyết ở đây.

### Vì sao `<select>` chứ không thêm nút

`check:commands` Kiểm A đòi mỗi `@click` là **đúng một** `dispatch('<id>')` với id **literal**
— vì thế Story 5.4 phải viết **bốn nút riêng** cho bốn trạng thái. Trạng thái là danh mục
**đóng** (4 giá trị) nên viết tay được; **lĩnh vực** và **ngôn ngữ nguồn** là tập **MỞ** (dữ
liệu người dùng gõ), không thể có một nút literal cho mỗi giá trị.

Doc-comment của chính cổng đó (`scripts/check-commands.mjs:33`) ghi: *"Chỉ `@click`.
`@keydown`, `@input`, `@change`, `@submit` KHÔNG thuộc luật Kiểm A"*. Và `LibraryMode.vue` đã
có tiền lệ trong chính tệp này: `<select v-model="sourceLang">` ở form nhập (~dòng 543).
⇒ Ba `<select>` với `@change` là đường hợp lệ, và `<select>` gốc HTML đã thao tác được bằng
bàn phím — AC7 không cần một cơ chế điều hướng tự chế.

⚠️ `<option>` phải dựng từ hai mảng **do Rust trả về** trên toàn bảng **chưa lọc**, không phải
từ `works` đã tải. Suy từ `works` làm lựa chọn **teo dần**: lọc "Tiên hiệp" xong thì mảng chỉ
còn Tiên hiệp, và mọi lĩnh vực khác biến mất khỏi ô chọn — người dùng kẹt, không đường quay
lại. Đây cũng đúng là điều AD-1 cấm.

### Bàn đo sẽ đỏ vì bố cục, không vì hồi quy

`e2e/specs/story-5-5-progress.e2e.mjs` neo vào `.works-list`/`.works-row`. Đổi sang lưới làm
nó đỏ **theo thiết kế**. Sửa bộ chọn cùng lượt, và ghi rõ trong commit rằng lượt đỏ đó là do
đổi bố cục — nếu không, lượt đỏ kế tiếp sẽ bị đọc nhầm là hồi quy tiến độ, đúng cái bẫy mà
Story 5.5 §Design Notes đã mắc một lần.

## Verification

**Commands:**
- `cargo test --manifest-path src-tauri/Cargo.toml` -- expected: 0 đỏ; các ca mới của
  `library_index_contract.rs`/`library_commands_contract.rs`/`ipc_contract.rs` xanh; hai cổng
  `meta_write_boundary.rs` và
  `segment_contract.rs::a_flush_touches_exactly_target_text_and_updated_at_and_nothing_else`
  **vẫn xanh**.
- `npm run test` -- expected: 0 đỏ, gồm ca mới ở `tests/frontend/libraryWorks.test.ts`.
- `npm run build` -- expected: `vue-tsc` 0 lỗi (hai `tsconfig`), `vite build` xong.
- `npm run check:commands` -- expected: xanh; nếu số `.vue`/`.ts`/`@click`/`dispatch()` thật
  đổi thì **nâng sàn kèm con số đã đếm**, không nới luật.
- `npm run check:i18n` -- expected: xanh; mọi khoá mới có mặt, không khoá mồ côi.
- `npm run check:lint` · `npm run check:tokens` · `npm run check:layout` ·
  `npm run check:gates` · `npm run check:debt-owner` -- expected: tất cả xanh.
- `npm run test:e2e` -- expected: `story-5-6-library-grid` xanh, `story-5-5-progress` xanh sau
  khi cập nhật bộ chọn, và **không** spec nào khác chuyển từ xanh sang đỏ.

**Đối chứng bắt buộc (chạy THẬT, ghi kết quả — không suy luận):**
- Gỡ phép tính `MAX` ở `rebuild_from_store`, trả lại lượt chép `work.updated_at` ⇒ ca AC8
  phải **đỏ**. Khôi phục và xác nhận xanh lại.
- Gỡ mệnh đề `AND genre = ?` khỏi `list_works` ⇒ ca "ba bộ lọc chồng" phải **đỏ**.
- Gỡ khoá phụ `, work_id` khỏi `ORDER BY` ⇒ ca "thứ tự ổn định khi trùng `updated_at`" phải
  **đỏ** (nếu nó vẫn xanh thì ca đó chưa dựng được hai hàng trùng mốc — sửa ca, không bỏ qua).
- Đổi hai mảng lựa chọn sang suy từ `works` đã lọc phía TypeScript ⇒ ca "lựa chọn không teo
  dần" phải **đỏ**.
- Gỡ lượt kẹp biên con trỏ sau mỗi lượt tải ⇒ ca "con trỏ kẹp biên sau lọc" phải **đỏ**.

## Auto Run Result

Status: blocked
Blocking condition: matrix ambiguity

Dừng ở **step-03 → Matrix Test Audit**, SAU khi mọi lệnh §Verification đã xanh. Mã đã dựng
nằm nguyên trên đĩa, chưa commit.

### Chỗ chặn — một mâu thuẫn trong chính spec này, không phải lỗi thi hành

Hàng ma trận **"`updated_at` chưa dựng lại"** viết:

> `meta.json` v1/v2 chưa qua `rebuild_from_store` ⇒ *Vẫn sắp được (giá trị lúc tạo), **lưới ghi
> rõ đó là mốc tạo***

Hai vế, và cả hai đều hở:

① **Không ca nào phủ hàng này.** Đo: `sorting_by_updated_at_orders_the_most_recently_touched_work_first`
không dựng hàng `status = None` nào; `a_true_v1_meta_json_indexes_with_status_null_and_matches_no_filter`
(ca cũ, Story 5.4) có dựng `meta.json` v1 thật nhưng **không gọi một lượt sắp xếp nào**. Giao
của hai ca là rỗng.

② **Vế "lưới ghi rõ đó là mốc tạo" KHÔNG dựng được** dưới chính §Never của spec này, và đó là
lỗi của tôi lúc viết spec. Để lưới nói được câu đó, nó phải phân biệt được một `meta.json` đã
qua `rebuild_from_store` với một cái chưa. Các dấu hiệu sẵn có đều **không** làm được việc đó:
- `status === null` / `chapter_done_count === null` chỉ nhận ra `meta.json` **v1/v2**. Một
  `meta.json` **v3** ghi TRƯỚC story này cũng mang `updated_at` đóng băng nhưng `status` khác
  `null` — nó lọt qua dấu hiệu này im lặng.
- So `updated_at == created_at` cũng không phân biệt được: một Tác phẩm **vừa tạo** hợp lệ
  cũng có đúng đẳng thức đó.
- Dấu hiệu duy nhất phân biệt được là **bump `META_SCHEMA_VERSION` 3→4** — mà §Never của
  chính spec này cấm tường minh ("không bump `META_SCHEMA_VERSION`").

⇒ Hàng ma trận đòi một hành vi mà cùng spec đó cấm phương tiện thực hiện. Giao thức step-03
nói rõ: *"never edit the expectation to match the code"* — nên tôi **không** sửa hàng ma trận
cho khớp mã đã dựng, và dừng.

### Hai đường gỡ, bạn chọn

**(A) Hạ vế thứ hai — rẻ, và có lý.** Bỏ mệnh đề *"lưới ghi rõ đó là mốc tạo"*, giữ vế
*"Vẫn sắp được"*, rồi thêm một ca sắp xếp trên hàng `meta.json` v1. Lý lẽ đỡ nó: AC2 của epic
liệt kê đúng bốn thứ mỗi ô phải mang (bìa · tên · tiến độ · trạng thái) — **ngày sửa không
nằm trong đó**, và lưới hôm nay không hiển thị ngày, nên nó không khẳng định điều gì sai với
người dùng. Thiệt hại thật thu về đúng một điều: một Tác phẩm có `meta.json` ghi trước story
này bị **xếp theo ngày TẠO** trong lượt sắp "ngày sửa gần nhất", cho tới lần đổi trạng thái
Chương kế tiếp.

**(B) Bump `META_SCHEMA_VERSION` 3→4** để lưới phân biệt được và nói thật. Đắt hơn: một lượt
di trú, và nó lật một câu §Never mà spec đã ký.

Tôi nghiêng về **(A)** — nhưng đây là lật một mệnh đề trong `<intent-contract>`, không phải
việc tôi tự quyết khi chạy không người trực.

### Nghiệm thu ĐÃ CHẠY và ĐÃ XANH (tôi tự chạy, không lấy báo cáo của agent)

- `cargo test` — **882 xanh, 0 đỏ** (35 nhị phân test + doctest).
- Hai cổng bắt buộc giữ xanh, không bị sửa: `meta_write_boundary.rs` **14/14**;
  `segment_contract.rs::a_flush_touches_exactly_target_text_and_updated_at_and_nothing_else`
  **1/1**.
- `npm run test` — **45/45 tệp, 618/618 ca**.
- `npm run build` — `vue-tsc` 0 lỗi, `vite build` xong.
- Bảy cổng tĩnh — `check:commands` · `i18n` · `lint` · `tokens` · `layout` · `gates` ·
  `debt-owner`: **PASS** cả bảy.
- `npm run test:e2e` — **CHƯA CHẠY**. Không được đọc thành đạt.

### 15 hàng ma trận còn lại đều có ca phủ đã chạy

Ba bộ lọc riêng rẽ và chồng nhau · lĩnh vực không tồn tại giữ nguyên `total` · hai khoá sắp ·
khoá sắp lạ bị từ chối · mặc định `updated_desc` · thứ tự ổn định nhờ khoá phụ `work_id` · tập
lựa chọn `DISTINCT` trên bảng chưa lọc · bìa vắng mặt · tên rỗng dùng `?` · Library rỗng ·
chưa tải xong · con trỏ kẹp biên · con trỏ trên danh sách rỗng · không lọc.

### Đối chứng bắt buộc — agent báo đã chạy cả năm

Trong đó một phát hiện đáng giữ lại: ca "thứ tự ổn định" lúc đầu **xanh giả** — cách đặt tên
fixture làm thứ tự quét vật lý tình cờ trùng thứ tự kỳ vọng, nên gỡ khoá phụ `, work_id` nó
vẫn xanh. Fixture đã dựng lại, và sau đó đối chứng mới đỏ đúng chỗ.
⚠️ Năm đối chứng này tôi **lấy từ báo cáo của agent**, chưa tự chạy lại — ghi rõ nguồn thay
vì để nó lẫn vào khối tôi tự đo ở trên.
