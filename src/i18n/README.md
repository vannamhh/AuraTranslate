Tài nguyên chuỗi giao diện — **không một chuỗi hiển thị nào nằm trong mã, kể cả chuỗi lỗi** (NFR16, AD-21). Cưỡng chế bằng lệnh, không bằng một comment.

**Story sở hữu nội dung: 1.5.**

---

## Bốn tệp, một hàm

| Tệp | Vai |
|---|---|
| `vi.json` | **Nguồn sự thật duy nhất.** Object **PHẲNG**, khoá chấm. v1 chỉ tiếng Việt |
| `resolve.ts` | `createResolver(catalog)` → `t(key, params?)`. Hàm thuần, **không `import` gì** |
| `index.ts` | Chỗ **DUY NHẤT** chạm `vi.json`. Export `t`, `tError`, kiểu `IpcError` |
| `README.md` | Tệp này |

```ts
// ⚠️ Đường dẫn TƯƠNG ĐỐI. Dự án không khai alias `@` ở `vite.config.ts` hay
// `tsconfig.json`, nên `from '@/i18n'` không phân giải được.
import { t, tError } from '../i18n'

t('err.unknown')
t('err.io.read_failed', { path: '/x/y.txt' })
tError(payloadLoiTuRust)             // nhận nguyên `{ code, message_key, params, retryable }`
```

## Hình dạng `vi.json` — phẳng, không lồng

```json
{
  "err.unknown": "…",
  "err.io.read_failed": "Không đọc được tệp tại {path} — nội dung chưa được nạp."
}
```

**Không lồng object.** `{"lookup": {"empty_result": "…"}}` là **sai hình dạng** — nguồn: `ARCHITECTURE-SPINE.md §Consistency Conventions` (*"tài nguyên chuỗi `vi.json` **phẳng** theo khoá chấm"*). Cưỡng chế ở hai phía: `check-i18n.mjs` Kiểm B nói bằng thông báo rõ ràng, và test Rust `every_message_key_exists_in_vi_json` deserialize vào `BTreeMap<String, String>` nên một object lồng gãy ngay ở đó.

Khoá phải khớp `^[a-z0-9]+(\.[a-z0-9_]+)+$` — chữ thường, gạch dưới, và **bắt buộc có tiền tố miền** (≥ 1 dấu chấm). Placeholder là `{ten_tham_so}`, tên khớp `[a-z_][a-z0-9_]*`; `{}`, `{Path}`, `{0}`, `{ path }` đều là FAIL ở cổng.

⚠️ **Khoá `vi.json` và id `CommandRegistry` dùng CÙNG một hình dạng nhưng KHÔNG cùng một không gian tên** (Story 1.6 AC2). Cùng hình dạng, hai danh mục. Đừng dựng một bảng tra dùng chung: một command và nhãn của nó là hai thứ đổi độc lập với nhau.

## Thêm một khoá

1. Thêm dòng vào `vi.json` — khoá chấm, giá trị là chuỗi không rỗng.
2. Soạn chuỗi theo năm quy tắc giọng văn ở §Giọng văn bên dưới.
3. Nếu **Rust** phát ra khoá đó: thêm một biến thể vào `message_keys!` ở `src-tauri/src/core/i18n/mod.rs`, **kèm danh sách tham số trong ngoặc vuông** — `IoReadFailed => "err.io.read_failed" ["path"]`. ⚠️ Chỉ thêm ở đó: `MessageKey::ALL`, `as_str()` và `required_params()` sinh từ cùng một khai báo nên không phải sửa ba chỗ, và `every_message_key_declares_the_params_its_string_needs` đối chiếu danh sách ấy với placeholder trong `vi.json` theo **cả hai chiều**.
4. `npm run check:i18n` **và** `cargo test --manifest-path src-tauri/Cargo.toml`.

**Dựng `IpcError` chỉ qua `IpcError::new()`.** Bốn trường là riêng tư có chủ ý — `new` là chỗ duy nhất nối `message_key` với `params`, và một struct literal đi vòng qua nó (`params: BTreeMap::new()` cho một khoá đòi `{path}`) biên dịch sạch, qua mọi phép kiểm khác, rồi đặt nguyên văn `{path}` lên màn hình người dùng.

**Đừng dựng sẵn một từ vựng khoá cho tính năng chưa tồn tại.** Thư mục này sở hữu **cơ chế**, không sở hữu **từ vựng**; mỗi story sau tự thêm khoá của nó cùng lúc với tính năng cần nó. Một `vi.json` 200 khoá cho panel chưa ai dựng là 200 chuỗi không ai nghiệm thu được, và chúng sẽ sai. Hôm nay có đúng **hai** khoá mồi, và đó là một quyết định.

## Giọng văn — năm quy tắc, và cái nào máy chấm được

`EXPERIENCE.md §Voice and Tone` — viết cho **một người dịch chuyên nghiệp**, không phải cho người dùng phổ thông cần dỗ dành.

| Quy tắc | Đúng | Sai | Máy chấm? |
|---|---|---|---|
| Nói việc, không nói cảm xúc | *"Đã gộp hai câu."* | *"Tuyệt vời! Đã gộp xong 🎉"* | ✗ *(mắt)* |
| Nêu hệ quả, không chỉ nêu sự kiện | *"Câu mới chưa xác nhận — lịch sử của hai câu cũ vẫn tra lại được."* | *"Đã gộp."* | ✗ *(mắt)* |
| Không đổ lỗi người dùng | *"Nhà cung cấp không phản hồi"* | *"Bạn đã nhập sai khoá"* | ✗ *(mắt)* |
| Số liệu là số liệu | *"412 token · ước tính ~0,004 USD"* | *"một chút chi phí"* | ✗ *(mắt)* |
| Vô nhân xưng | *"Không đọc được tệp tại {path}."* | *"Chúng tôi không đọc được tệp bạn chọn."* | ✅ **Kiểm D** |

Ba quy tắc "mắt" nghiệm thu bằng cách chép chuỗi mới vào Completion Notes của story kèm một câu vì sao chọn chữ đó — đây là lý do §Thêm một khoá cấm dựng sẵn một từ vựng lớn: 200 chuỗi thì không ai nghiệm thu nổi và quy tắc thoái hoá thành trang trí.

## Cổng cưỡng chế

`npm run check:i18n` — **năm** phép kiểm, mã thoát là phán quyết, đã gắn vào `ci.yml` trong job `check`, kề `check:tokens` và **trước** `npm run build`. Cả năm đã nghiệm thu **đỏ-rồi-xanh**: 16 ca lúc dựng, cộng **23 ca** của lượt code review 2026-08-04 (13 ca hỏng + 10 đối chứng âm). Xem §Review Findings của story 1.5.

| | Kiểm gì | AC |
|---|---|---|
| **A** | Không ký tự có dấu tiếng Việt ở **vị trí mã** của `src-tauri/**/*.rs` và `src/**/*.vue`. Quét có trạng thái: comment, char literal Rust, regex literal JS, và ba vùng của `.vue` đều được phân biệt; nội dung chuẩn hoá **NFC** trước khi so | AC2 |
| **B** | `vi.json` phẳng · khoá chấm có tiền tố miền · không giá trị rỗng · **không khoá trùng** (đọc văn bản thô, vì `JSON.parse` nuốt bản trùng im lặng) | AC1 |
| **C** | Placeholder khớp `{ten_tham_so}` — cùng dải `resolve.ts` nội suy — và **không còn ngoặc nhọn thừa** sau khi bóc placeholder hợp lệ | AC1 |
| **D** | Giọng văn vô nhân xưng: không *"chúng tôi"*, không *"bạn"* — so theo biên tiếng, `\s+` giữa các tiếng, chuẩn hoá NFC | AC5 |
| **E** | **Hành vi thật** của `resolve.ts` — nạp và gọi hàm, cả hai chiều — và chạy lại trên **`vi.json` THẬT**, không chỉ trên một catalog giả | AC4 |

**Ngưỡng sàn bắt buộc:** ≥ 14 tệp `.rs`, ≥ 1 tệp `.vue` **sau miễn trừ** (số thật hôm nay: 18 và 1). Quét được ít hơn ⇒ `abort()`, **không phải "đạt"**. Đây là bẫy mà `check-deps.mjs` đã đâm vào một lần: *"cây rỗng đọc thành sạch"*.

## Hai miễn trừ CÓ TÊN — và không có đường thứ ba

| Đường dẫn | Cho phép gì | Vì sao có |
|---|---|---|
| `src-tauri/tests/**` | thông báo `assert!` tiếng Việt | Không vượt IPC, không được render; người đọc chúng là người đang sửa test. Dịch sang tiếng Anh là mất giá trị tài liệu để đổi lấy con số không |
| `src/selftest/**` | chẩn đoán cho log CI | Debug-only, `import()` động, không vào bundle release. ⚠️ Hôm nay khớp **0 tệp** và con số đó được in ra có chủ ý: thư mục chỉ có `.ts`, mà Kiểm A đi trên `.rs` và `.vue`. Nghĩa là chuỗi tiếng Việt trong `src/selftest/*.ts` được che bởi **lỗ phạm vi**, không bởi miễn trừ này — đừng dùng "dời sang `.ts`" như một cách cho cổng xanh |

**Miễn trừ không được cài bằng cách thu hẹp glob quét.** Glob quét cả cây; miễn trừ là một bước lọc **có tên và có lý do**, và cổng **in ra số tệp đã miễn trừ ở mỗi lượt chạy** để nó không lặng lẽ phình lên. Cùng luật mà `check-tokens.mjs` áp cho danh sách cặp màu loại trừ.

## Ba thứ sẽ hỏng im lặng nếu không ai nói trước

### 1. `#[serde(rename_all = "camelCase")]` trên `IpcError` — hợp đồng AD-21 gãy, không lỗi nào được ném

Thói quen viết Tauri là đặt nó lên mọi struct qua IPC cho hợp phong cách JS. Ở đây nó biến `message_key` thành `messageKey`: Rust biên dịch sạch, và mọi chỗ đọc theo AD-21 nhận `undefined` rồi hiển thị chuỗi rỗng. **Bốn tên trường là dây, không phải sở thích.** `tests/ipc_contract.rs::ipc_error_wire_shape` so `to_value(...).keys()` với **đúng bốn chuỗi, đúng chính tả** — đã nghiệm thu đỏ thật với đúng thuộc tính này.

### 2. `#[derive(Serialize)]` trần trên `enum MessageKey` — khoá ra sai mà vẫn là chuỗi hợp lệ

Serde mặc định serialize unit variant thành **tên biến thể**: `MessageKey::IoReadFailed` → `"IoReadFailed"`. Đó là một chuỗi, JSON hợp lệ, không lỗi. Frontend tra `"IoReadFailed"`, không thấy, và theo đúng AC4 nó **hiện khoá nguyên văn rồi ghi cảnh báo** — tức hỏng đúng kiểu *"trông như đang chạy"*. `Serialize` viết tay (`serialize_str(self.as_str())`) là bắt buộc.

### 3. `resolve.ts` `import` một thứ gì đó — Kiểm E chết, AC4 quay về nghiệm thu bằng mắt

Dự án **không có bộ chạy test frontend** (thêm `vitest` là thêm một phụ thuộc phải rà GPLv3 và vào bảng Stack trước — NFR15, quyết định của Ice). Đường thay thế: **Node ≥ 22.18 bóc kiểu TypeScript mặc định**, nên cổng `import()` thẳng được `resolve.ts`. Điều kiện:

- Không một dòng `import` nào — Node không phân giải `./vi.json` theo luật bundler của Vite và không hiểu `.vue`.
- Cú pháp phải **"erasable-only"**: không `enum`, không `namespace`, không parameter property. `type` / `interface` / annotation thì được.

⇒ `resolve.ts` = hàm thuần + kiểu. `index.ts` = chỗ duy nhất chạm `vi.json` và Vue.

## Vì sao không dùng `vue-i18n`

`vue-i18n` 11.4.8 là **MIT** — tương thích GPL v3, không có gì sai về mặt pháp lý. Vẫn không dùng, ba lý do xếp theo sức nặng:

1. **NFR15 là một thủ tục, không phải một lượt kiểm.** Mọi phụ thuộc mới phải rà tương thích GPLv3 **trước khi** thêm **và ghi vào bảng Stack** — bảng có 19 hàng đã ghim và `check:deps` cưỡng chế nó. Thêm hàng thứ 20 là quyết định của Ice.
2. **Phần dự án này cần chỉ là một hàm.** v1 chỉ tiếng Việt (NFR16): không chuyển ngôn ngữ lúc chạy, không số nhiều, không định dạng theo vùng, không lazy-load bundle ngôn ngữ. Còn lại là tra một khoá phẳng, nội suy vài placeholder, xử lý khoá thiếu — khoảng 40 dòng.
3. **Một thư viện làm Kiểm E đắt hơn hẳn.** `resolve.ts` thuần thì Node nạp thẳng; một plugin Vue thì phải dựng app instance để kiểm, và điều đó kéo theo đúng bộ chạy test frontend mà lý do #1 vừa hoãn.

**Điều kiện mở lại:** khi có ngôn ngữ thứ hai thật, hoặc khi nhu cầu số nhiều/định dạng vùng xuất hiện. Lúc đó: rà GPLv3 → thêm vào bảng Stack → thay `createResolver` **sau** ranh giới `t()`. Ranh giới đó chính là thứ thư mục này dựng để lần đổi ý sau rẻ.
