---
id: 1
type: story
title: "Build công khai qua GitHub Actions"
parent: epic-phat-hanh-va-tin-cay
covers: [FR107]
after: []
hitl: true
risk: low
---

# Build công khai qua GitHub Actions

## Description

Người dùng cẩn thận được: kiểm chứng được binary khớp với mã nguồn. Mục đích: tôi tin được một bản cài không ai ký tên vào.

Tiêu chí gốc (epics.md, nguyên văn):

**Given** một bản phát hành
**When** build
**Then** build công khai qua **GitHub Actions**, log xem được

**Given** dữ liệu từ điển
**When** CI cần
**Then** tải theo `dict-manifest.toml`
**And** **đối chiếu SHA-256** trước khi dùng

**Given** một file `.db` có checksum không khớp manifest
**When** CI chạy
**Then** build **thất bại**, không đóng gói tiếp

**Given** repo
**When** kiểm
**Then** chứa **mã build tool** và `dict-manifest.toml`
**And** **không chứa** nguồn thô 1,13 GB hay các file `.db` (~320 MB cho ba tệp)

**Given** parser định dạng từ điển
**When** kiểm bản phát hành
**Then** **không có mặt** — chúng chỉ sống trong `tools/dict-build`

**Given** bất kỳ ai
**When** đọc cấu hình CI
**Then** dựng lại được chuỗi từ mã nguồn tới artifact

Story này cũng nhận các mục nợ đứng tên `Chủ: Story 10.1` trong `deferred-work.md`. Phần từ điển và phát hành gồm bốn mục: đưa bốn tệp `.db` vào `bundle.resources` kèm test khẳng định có mặt; đo NFR6 trên bản cài thật; tải dump thô kaikki lên GitHub Release (phiếu #86); điều kiện khởi hành của FR107. Phần làm cứng CI gồm tám mục chuyển từ Story 1.3 ngày 2026-10-07.

Phần do người làm: Ice duyệt rồi tạo release `dict-v2` chứa bốn tệp `.db` khớp manifest, và tải dump thô kaikki lên GitHub Release. Cả hai đều đẩy dữ liệu ra ngoài repo.

Ranh giới: story dừng ở chỗ workflow cho ra `.dmg` và `.msi` làm artifact của lượt chạy. Tạo GitHub Release cho ứng dụng thuộc Story 10.2. Checksum công bố cho người dùng thuộc Story 10.3.

## Acceptance Criteria

Verify: Một lượt chạy `release.yml` trên một tag cho ra `.dmg` và `.msi` chứa đủ bốn tệp từ điển tải theo `dict-manifest.toml`, log xem công khai được; sửa một sha trong manifest thì job đỏ trước bước `tauri build`.

## References

- parent — _bmad-output/initiative-auratranslate/epic-phat-hanh-va-tin-cay/epic-phat-hanh-va-tin-cay.md
- constraint — _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md, AD-25, AD-10, AD-45
- debt — _bmad-output/initiative-auratranslate/deferred-work.md, các mục có `Chủ:` cuối là `Story 10.1` (grep, không đọc cả tệp)

## Notes

- Decision 2026-10-12: tám mục nợ làm cứng CI ở lại trong story này, không tách story riêng (Ice chọn).
- Decision 2026-10-12: tạo release `dict-v2` chứa bốn tệp khớp manifest, manifest trỏ sang đó; `dict-v1` giữ nguyên làm lịch sử (Ice chọn). Đo ngày 2026-10-12: `dict-v1` có 3 asset với sha cũ và thiếu `dict-tran-van-chanh.db`.
- Decision 2026-10-12: release `dict-v2` đã tạo trên tag `d3fb849`, bốn asset có sha-256 khớp `dict-manifest.toml`; không nhận nhãn Latest. `dict-manifest.toml` vẫn trỏ `dict-v1`: đổi bốn URL sang `dict-v2` là việc của lượt build (Ice chọn).
- Decision 2026-10-12: ba tệp kaikki (`Chinese.jsonl`, `vi-extract.jsonl`, `kaikki-en-vi.jsonl`) nén `gzip -9 -n` lên release `dict-raw-v1`, không nhận nhãn Latest (Ice chọn). Dựng lại từ `raw/` với `SOURCE_DATE_EPOCH=1754352000` ra đúng sha bốn `.db` của `dict-v2`.
- Decision 2026-10-12: dựng bản cài ở tệp `release.yml` riêng, chạy trên tag; điều này gỡ ràng buộc một tệp workflow của Story 1.3 AC4 (Ice chọn).
- Decision 2026-10-12: risk giữ `low` (Ice chọn).
- Open question: nợ dòng 341–342 ghi ba tệp; manifest nay có bốn tệp (thêm `dict-tran-van-chanh.db`), và test có mặt phải khẳng định cả bốn.
- Open question: riêng bốn tệp từ điển đã 373.239.808 byte, 93,3% trần NFR6; payload đầy đủ trong nợ là 396.895.366 byte. Đo trên bản cài thật trước khi hứa đóng gói.
- Open question: chưa có cổng cấm `src-tauri` hay bản phát hành chứa parser từ điển; cổng hiện có chỉ canh chiều `tools/dict-build` không trỏ sang `src-tauri`.
- Open question: số cảnh báo của `cargo clippy -D warnings` và kết quả `cargo-deny` chưa đo; cỡ phần làm cứng CI chưa có số.
