Parser dữ liệu từ điển sống ở đây và **không vào bản phát hành** (AD-25).

Nguồn thô của lớp NỀN (**Story 1.9**, crate này): CVDICT · CC-CEDICT · Unihan ·
viwiktionary · en.wiktionary. Chạy qua đây trên máy người dựng, ra `dict-core.db`
phát hành qua GitHub Release và tải theo `../../dict-manifest.toml`.

Bốn lớp GỠ RỜI (Thiều Chửu · Cổ hán văn · VietPhrase · HVTĐTD) là **Story 1.10**,
KHÔNG thuộc crate này (AD-10) — mỗi lớp một tệp `.db` riêng, dùng lại đúng lược đồ
`dict_source` của `src/schema.rs`.

## Hình dạng (chốt ở Story 1.9)

`tools/dict-build` là một **crate Rust độc lập**, `[workspace]` rỗng của chính nó
(`Cargo.toml`) — cây phụ thuộc **không giao nhau về mặt vật lý** với `src-tauri`
(AC4, §Quyết định #1/#2 của Story 1.9). Có `Cargo.lock` riêng, **commit được** (đây
là một binary, không phải library).

```
tools/dict-build/
  Cargo.toml / Cargo.lock
  assets/licenses/          # văn bản giấy phép NGUYÊN VĂN, include_str! lúc biên dịch
  src/
    main.rs                 # CLI, chỉ gọi build::run — logic thật ở lib.rs
    lib.rs
    build.rs                 # điều phối: raw → parse → chèn → char_idx → finalize
    schema.rs                 # DDL hằng, chín bảng + ba FTS5 (§Quyết định #2/#3)
    model.rs                  # RawEntry/RawSense/... — hình dạng trung gian dùng chung
    insert.rs                 # RawEntry → hàng SQL
    char_idx.rs                # (ch, entry_id), phủ cả phồn lẫn giản (Bẫy 8)
    finalize.rs                # rebuild FTS · ANALYZE · VACUUM · journal_mode=DELETE
    licenses.rs / sources_meta.rs
    sources/{cvdict,cc_cedict,unihan,viwiktionary,en_wiktionary}.rs
    sources/{cedict_common,wiktextract_common}.rs   # code đọc dùng chung, KHÔNG hợp nhất nghĩa
  tests/
    fixtures/raw/<nguồn>/... # trích thật từ CVDICT/CC-CEDICT/Unihan/kaikki.org, nhỏ, commit được
    parse.rs / schema.rs
  raw/ out/ work/            # .gitignore — nguồn thô tải về + artifact trung gian
```

## Chạy

```
cargo run --manifest-path tools/dict-build/Cargo.toml -- \
  --raw tools/dict-build/raw --out tools/dict-build/out/dict-core.db
```

`--raw <dir>` phải chứa đúng quy ước thư mục con mà `tests/fixtures/raw/` minh hoạ:
`cvdict/CVDICT.u8` + `cvdict/SOURCE_VERSION.txt` · `cc_cedict/cedict.txt` ·
`unihan/Unihan_Readings.txt` + `unihan/Unihan_Variants.txt` ·
`viwiktionary/vi-extract.jsonl` · `en_wiktionary/Chinese.jsonl`. Tải năm nguồn theo
bảng ở Dev Notes §Thông tin kỹ thuật của Story 1.9 — build tool **không tự tải** gì
từ mạng (§Quyết định #6: AD-15 khoá điểm ra mạng, một nguồn tải ngầm là artifact
không ai biết phiên bản).

`Unihan.zip` được **giải nén tay** trước khi chạy (§Quyết định #6) — không thêm
crate `zip` cho một bước chạy một lần.

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

## Giấy phép năm nguồn dữ liệu

| Nguồn | Giấy phép | Văn bản |
|---|---|---|
| CVDICT | CC BY-SA 4.0 | `assets/licenses/CC-BY-SA-4.0.txt` |
| CC-CEDICT | CC BY-SA 4.0 | `assets/licenses/CC-BY-SA-4.0.txt` |
| Unihan | Unicode License v3 | `assets/licenses/Unicode-License-v3.txt` |
| viwiktionary | CC BY-SA 4.0 + GFDL 1.3 | `assets/licenses/{CC-BY-SA-4.0,GFDL-1.3}.txt` |
| en.wiktionary (mục tiếng Trung) | CC BY-SA 4.0 + GFDL 1.3 | như trên |

Mỗi văn bản là bản tải THẬT từ nguồn chính thức (creativecommons.org / unicode.org /
gnu.org), 2026-08-04 — không phải bản tóm tắt tự viết. Ghi vào cột `dict_source.
license_text` của `dict-core.db` lúc dựng.
