Parser dữ liệu từ điển sống ở đây và **không vào bản phát hành** (AD-25).

Nguồn thô của lớp NỀN (**Story 1.9**, crate này): CVDICT · CC-CEDICT · Unihan ·
viwiktionary · en.wiktionary. Chạy qua đây trên máy người dựng, ra `dict-core.db`
phát hành qua GitHub Release và tải theo `../../dict-manifest.toml`.

Bốn lớp GỠ RỜI (Thiều Chửu · Cổ hán văn · VietPhrase · HVTĐTD) thuộc **Story 1.10**
(AD-10) — mỗi lớp một tệp `.db` riêng, dùng lại đúng lược đồ `dict_source` của
`src/schema.rs`. **Story 1.10 giao HAI trong bốn lớp** (Thiều Chửu · VietPhrase); hai
lớp còn lại (HVTĐTD · Cổ hán văn) chưa có nguồn thô — xem `deferred-work.md`.

## Hình dạng (chốt ở Story 1.9)

`tools/dict-build` là một **crate Rust độc lập**, `[workspace]` rỗng của chính nó
(`Cargo.toml`) — cây phụ thuộc **không giao nhau về mặt vật lý** với `src-tauri`
(AC4, §Quyết định #1/#2 của Story 1.9). Có `Cargo.lock` riêng, **commit được** (đây
là một binary, không phải library).

```
tools/dict-build/
  Cargo.toml / Cargo.lock   # version 0.2.0 từ Story 1.10
  assets/licenses/          # văn bản giấy phép NGUYÊN VĂN, include_str! lúc biên dịch
  src/
    main.rs                 # CLI --raw --out-dir [--layer], chỉ gọi build::run — logic thật ở lib.rs
    lib.rs
    build.rs                 # điều phối: raw → parse → chèn → char_idx → finalize (base + từng lớp gỡ rời)
    schema.rs                 # DDL hằng, chín bảng + ba FTS5 (§Quyết định #2/#3 của Story 1.9) — 0 dòng đổi ở 1.10
    model.rs                  # RawEntry/RawSense/... — hình dạng trung gian dùng chung
    insert.rs                 # RawEntry → hàng SQL
    char_idx.rs                # (ch, entry_id), phủ cả phồn lẫn giản (Bẫy 8)
    finalize.rs                # rebuild FTS · ANALYZE · VACUUM · journal_mode=DELETE — DÙNG CHUNG mọi lớp
    licenses.rs / sources_meta.rs   # BASE_ALL[5] + DETACHABLE_ALL[2] (Story 1.10)
    sources/{cvdict,cc_cedict,unihan,viwiktionary,en_wiktionary}.rs   # lớp NỀN — Story 1.9
    sources/{thieu_chuu,vietphrase}.rs                                # lớp GỠ RỜI — Story 1.10
    sources/{cedict_common,wiktextract_common}.rs   # code đọc dùng chung, KHÔNG hợp nhất nghĩa
  tests/
    fixtures/raw/<nguồn>/... # trích thật, nhỏ, commit được — ⛔ không bịa giá trị
    parse.rs / schema.rs / layers.rs
  raw/ out/ work/            # .gitignore — nguồn thô tải về + artifact trung gian
```

## Chạy

```
cargo run --manifest-path tools/dict-build/Cargo.toml -- \
  --raw tools/dict-build/raw --out-dir tools/dict-build/out [--layer <code>]
```

`--layer` nhận `base` · `thieu-chuu` · `vietphrase` · `all` (mặc định `all`).
`--layer all` dựng **đúng ba** tệp — `dict-core.db` · `dict-thieu-chuu.db` ·
`dict-vietphrase.db` — và hỏng nếu **bất kỳ** lớp nào thiếu nguồn thô (⛔ không có
chế độ bỏ qua lớp thiếu). `--out` (tham số cũ, một tệp) không còn được nhận — CLI
báo lỗi tường minh nêu tên tham số thay thế thay vì âm thầm hiểu nhầm.

`--raw <dir>` phải chứa đúng quy ước thư mục con mà `tests/fixtures/raw/` minh hoạ:

| Lớp | Thư mục con | Tệp |
|---|---|---|
| NỀN (Story 1.9) | `cvdict/` | `CVDICT.u8` + `SOURCE_VERSION.txt` |
| | `cc_cedict/` | `cedict.txt` |
| | `unihan/` | `Unihan_Readings.txt` + `Unihan_Variants.txt` |
| | `viwiktionary/` | `vi-extract.jsonl` |
| | `en_wiktionary/` | `Chinese.jsonl` |
| GỠ RỜI (Story 1.10) | `thieu_chuu/` | `TudienThienChuu.txt` |
| | `vietphrase/` | `VietPhrase.txt` |

Năm nguồn NỀN tải theo bảng ở Dev Notes §Thông tin kỹ thuật của Story 1.9. Build tool
**không tự tải** gì từ mạng (§Quyết định #6: AD-15 khoá điểm ra mạng, một nguồn tải
ngầm là artifact không ai biết phiên bản).

`Unihan.zip` được **giải nén tay** trước khi chạy (§Quyết định #6) — không thêm
crate `zip` cho một bước chạy một lần.

### Hai nguồn GỠ RỜI (Story 1.10) — chép-dán từ `docs/dics/`

> 🔴 **`docs/dics/` KHÔNG có trong repo** — nó nằm trong `.gitignore` *(~344 MB nhị phân:
> `tudien-2.2.zip` 289 MB · `VietPhrase.txt` 24 MB · `hanodict_tbl_dictionary.pbix` 31 MB)*.
> Đó là **kho cục bộ trên máy Ice**. Một `git add -A` vô ý sẽ nhét ngần ấy byte vào lịch
> sử git **vĩnh viễn** — không gỡ ra được, nên nó bị chặn ở `.gitignore`.
>
> **Nếu bạn clone repo và không có `docs/dics/`,** tải lại từ nguồn gốc rồi đối chiếu
> SHA-256 ở bảng bên dưới *(hash là hợp đồng, ⛔ không phải thư mục)*:
>
> | Tệp | Nguồn gốc | Ghi chú |
> |---|---|---|
> | `TudienThienChuu.txt` | `github.com/catusf/tudien` → `tudien-2.2.zip` → `tudien-2.2/dict/` | CC0 1.0. ⛔ Không dùng `.tab`/`.dict.dz`/`.mobi` — bản dẫn xuất dưới `output/` |
> | `VietPhrase.txt` | `github.com/truyencuatui/VietPhrase` | ⚠️ Kho gốc phát hành **UTF-16LE** — xem khối `iconv` bên dưới |
>
> ⚠️ **Giữ `tudien-2.2.zip` lại trên máy** — nó **là bằng chứng giấy phép CC0** của bản số
> hoá Thiều Chửu *(đối chiếu byte-for-byte với `TudienThienChuu.txt` bên trong)*, ⛔ không
> phải rác.

```sh
mkdir -p tools/dict-build/raw/thieu_chuu tools/dict-build/raw/vietphrase
cp "docs/dics/Thieu chuu/TudienThienChuu.txt" tools/dict-build/raw/thieu_chuu/TudienThienChuu.txt
cp "docs/dics/VietPhrase.txt" tools/dict-build/raw/vietphrase/VietPhrase.txt
```

Cả hai tệp trong `docs/dics/` hôm nay **đã là UTF-8, không BOM** — dùng thẳng, ⛔
không chạy `iconv`. Đối chiếu SHA-256 trước khi tin:

| Tệp | SHA-256 |
|---|---|
| `TudienThienChuu.txt` | `20ec62ba8dd9ec79d408bb824d7aba8b5eca4afca3c91426e45d693e77275f5f` |
| `VietPhrase.txt` | `5cb6a00d9697642c4e3cf735c24e88369ec199c69fcde45cb193036a0e26d617` |

```sh
shasum -a 256 tools/dict-build/raw/thieu_chuu/TudienThienChuu.txt
shasum -a 256 tools/dict-build/raw/vietphrase/VietPhrase.txt
```

Lệch bất kỳ hash nào ⇒ **DỪNG**, tệp không phải bản đã khảo sát.

⚠️ **Nếu tải lại VietPhrase từ kho gốc `truyencuatui/VietPhrase`** (không phải từ
`docs/dics/`): kho gốc phát hành **UTF-16LE**. Đọc UTF-16 như UTF-8 ⛔ không hỏng
ngay — nó đẻ ra hàng trăm nghìn `ParseIssue` rồi build vẫn chạy tiếp tới khi
`require_nonempty` chặn lại, tốn nửa giờ đi tìm sai chỗ. Chuyển mã BẰNG TAY trước khi
dùng, và bỏ dòng BOM đứng riêng nếu `iconv` để lại:

```sh
iconv -f UTF-16LE -t UTF-8 VietPhrase-raw.txt | sed '1s/^\xEF\xBB\xBF//' > VietPhrase.txt
```

⛔ Không thêm crate dò mã hoá cho bước này (§Quyết định #6 của Story 1.9) — build
tool chỉ đọc UTF-8, luôn luôn.

## Build TÁI LẬP ĐƯỢC — điều kiện để `sha256` trong manifest có nghĩa (AD-25)

**Cùng một cây `raw/` ⇒ cùng một tệp `.db` byte-for-byte ⇒ cùng một SHA-256.**

`dict_meta('built_at')` ⛔ **không** lấy từ đồng hồ hệ thống. Nó dẫn xuất theo thứ tự:

1. Biến môi trường **`SOURCE_DATE_EPOCH`** *(quy ước reproducible-builds)* — dùng khi
   muốn ghim cứng một mốc cho một lượt phát hành;
2. **`mtime` mới nhất** trong số nguồn thô mà lượt dựng đó đọc — một thuộc tính của ĐẦU VÀO;
3. Epoch 0, kèm cảnh báo ra stderr — chỉ khi ⛔ không đọc được metadata nào.

```sh
# Ghim mốc cho một lượt phát hành (tuỳ chọn) — cùng giá trị ⇒ cùng hash, trên mọi máy.
SOURCE_DATE_EPOCH=1754352000 cargo run --release -- --raw tools/dict-build/raw --out-dir tools/dict-build/out
```

🔴 **Vì sao dòng này tồn tại:** trước Story 1.10 (lượt code review), `built_at` dùng
`strftime('%Y-%m-%dT%H:%M:%fZ','now')` — độ phân giải **mili-giây**. Hai lượt build liên
tiếp từ **cùng** một cây fixture cho ra hai tệp khác byte, nên **mọi** giá trị `sha256`
trong `dict-manifest.toml` chỉ đúng cho **đúng một** lượt chạy. Một lần `cargo run` lại
trước khi upload là mọi máy khách fail checksum — và ⛔ **không cổng nào bắt được**, vì
`check-dict-manifest.mjs` cố ý không đọc một byte `.db` nào.

Test khoá hành vi này: `tests/layers.rs::two_builds_from_the_same_raw_tree_produce_identical_checksums`.

## Giấy phép từng crate (NFR15 — để lượt rà sau không phải đoán)

Bảng dưới không vào bảng Stack của `ARCHITECTURE-SPINE.md` (AD-25: giấy phép parser
không ràng buộc sản phẩm). Ghi lại giấy phép crate như một tài liệu tham khảo, không
phải một cưỡng chế.

| Crate | Phiên bản (Cargo.lock, 2026-08-04) | Giấy phép |
|---|---|---|
| `rusqlite` (feature `bundled`) | 0.37.0 | MIT |
| `serde` + `serde_json` | 1.0.229 / 1.0.151 | MIT OR Apache-2.0 |
| `sha2` | 0.10.9 | MIT OR Apache-2.0 |
| `tempfile` (dev-only, test) | 3.x | MIT OR Apache-2.0 |

## Giấy phép bảy nguồn dữ liệu (5 nền + 2 gỡ rời)

| Nguồn | `license_kind` | Giấy phép | Văn bản |
|---|---|---|---|
| CVDICT | `open` | CC BY-SA 4.0 | `assets/licenses/CC-BY-SA-4.0.txt` |
| CC-CEDICT | `open` | CC BY-SA 4.0 | `assets/licenses/CC-BY-SA-4.0.txt` |
| Unihan | `open` | Unicode License v3 | `assets/licenses/Unicode-License-v3.txt` |
| viwiktionary | `open` | CC BY-SA 4.0 + GFDL 1.3 | `assets/licenses/{CC-BY-SA-4.0,GFDL-1.3}.txt` |
| en.wiktionary (mục tiếng Trung) | `open` | CC BY-SA 4.0 + GFDL 1.3 | như trên |
| Thiều Chửu *(Story 1.10)* | `public-domain` | CC0 1.0 *(bản số hoá)* | `assets/licenses/{CC0-1.0,thieu-chuu}.txt` |
| VietPhrase *(Story 1.10)* | `unknown` | không xác định được tác giả | `assets/licenses/vietphrase.txt` |

Mỗi văn bản là bản tải/soạn THẬT — năm nguồn nền từ nguồn chính thức
(creativecommons.org / unicode.org / gnu.org), 2026-08-04; hai nguồn gỡ rời soạn
2026-08-05 kèm xuất xứ đã kiểm chứng (§Thông tin kỹ thuật, Story 1.10) — không phải
bản tóm tắt tự viết cho phần văn bản giấy phép nguyên văn. Ghi vào cột
`dict_source.license_text` của tệp `.db` tương ứng lúc dựng.

🔴 **Thiều Chửu** — attribution nêu **tên tác giả (Nguyễn Hữu Kha, 1902–1954)** là
nghĩa vụ pháp lý theo quyền nhân thân vô thời hạn, không phải phép lịch sự.
