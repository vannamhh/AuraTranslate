---
title: 'Story 10.9 — Nghiệm thu cuối các ngưỡng phi chức năng'
type: 'chore'
ticket: '9'
created: '2026-10-21'
status: 'done'
baseline_revision: '9b71b277532f72b926f42bd07297968ea55b17e2'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: ['{project-root}/_bmad-output/initiative-auratranslate/tickets/epic-duong-nhap/tickets/story-do-lai-nfr3-nfr4-nfr5-tren-thu-vien-5-000-chuong-that-plan.md']
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** v1 chưa có bằng chứng cuối cho các ngưỡng chỉ đo được khi sản phẩm đã đủ. NFR3/4/5 chỉ có số sơ bộ: 5.14 trên fixture SQL, 6.18 dựng xong bộ đo nhưng chưa chạy NFR4/NFR5 lần nào. NFR6 theo số đo của 10.1 đã vượt trần ngay trước khi tính binary: từ điển 373.239.808 B + font 27.266.464 B + giấy phép 35.149 B = 400.541.421 B. Số dung lượng chưa vào ghi chú phát hành. Rà giấy phép, i18n và tương phản chưa có lượt cuối. 17 mục nợ đang có `Chủ: Story 10.9` (Ice chọn giữ cả năm mục trong một plan, 2026-10-21).

**Approach:** Chạy trọn bộ đo `6-18-ban-do/` (task 7–9 của plan 6.18, nhập vào đây) và ghi kết luận A6/A7/A8 vào PRD. Cho `release.yml` ghi số dung lượng hai dòng vào ghi chú phát hành rồi đo trên một lượt `workflow_dispatch`. Rà lần cuối giấy phép (NFR15), i18n (NFR16) và tương phản. Mỗi mục nợ của 10.9 phải được đóng hoặc chuyển chủ. Việc chỉ người làm được ghi thành mục nợ `Chủ: Epic 10`, phần Windows ghi `Chủ: B7`.

**Decisions (Ice, 2026-10-21):**
- Q1 NFR6 = đo rồi trình nâng trần. Lấy số thật từ một lượt `workflow_dispatch`, trình Ice; Ice ký thì sửa hàng NFR6 trong PRD tại chỗ bằng 🔵, ghi rõ đây là thay đổi tầng PRD. Không gỡ lớp nào.
- Q2 NFR1 = bỏ phép đo đầu-cuối và sửa cả PRD. Hàng NFR1 (:914) sửa tại chỗ bằng 🔵 có ngày: bỏ vế đầu-cuối, ngưỡng p95 < 100 ms áp cho đường Rust đã đo. Các mục nợ :945, :1178, :1187 đóng bằng `KHÔNG LÀM 2026-10-21 (Story 10.9)`. Không viết mã đo nào.
- Q3 NFR15 = rà một lần, ghi kết quả vào báo cáo; không thêm cổng.
- Q4 i18n = giữ AC theo nghĩa đen, chỉ `.rs`/`.vue`. Mục nợ `.ts` (:50) chuyển sang `Chủ: Ice`.
- Q5 WCAG = giữ Kiểm C như hiện nay, cộng một lượt Ice duyệt bằng mắt trong WKWebView ở hai theme, kể cả Chế độ đọc và diff (`Chủ: Epic 10`). Không thêm kiểm 3:1.

**Decisions mang sang từ 6.18 (Ice, 2026-09-14), không hỏi lại:** NFR3/4/5 chỉ đo trên macOS; lượt đo lại trên Windows là nợ `Chủ: Ice` trỏ B7. Thư viện dựng bằng `preview_bilingual_import` → `confirm_bilingual_import` và `lifecycle::set_chapter_status`, đây là đường nhập của sản phẩm. Hình dạng 50 Tác phẩm × 100 Chương × 10 segment.

## Boundaries & Constraints

**Always:**
- Mọi quy tắc §Always/§Never của plan 6.18 vẫn áp dụng nguyên văn: profile, máy nhàn, fail-closed, mỗi số mang commit/ngày/máy/toolchain/tải.
- Muốn ghi một số đã đổi hoặc một kết luận tầng PRD thì phải trình số cho Ice trước và chờ Ice ký, kể cả NFR6. Nếu NFR5 rơi vào khoảng 300.000.000–314.572.800 B thì dừng để Ice phân xử MB hay MiB.
- Mỗi ngưỡng A6/A7/A8 nhận đúng một kết luận, ghi vào PRD (hàng NFR3–5, A6–A8, Q4) và SPEC. Chỗ nào có số 5.14 thì ghi "sơ bộ, thay bởi 10.9".
- Số NFR6 của từng hệ điều hành gồm hai dòng: payload sản phẩm và WebView2. Lấy từ build thật qua `inspect-release`, không lấy từ phép cộng tay.
- Mục nợ đóng bằng chữ (✅ / 🟡 / KHÔNG LÀM / `→ … Chủ:`), không xoá mục nào. `check:debt-owner` báo 0 mục mồ côi và 0 mục chủ đã chết.

**Never:**
- Không tối ưu sản phẩm, không sửa ngưỡng để lật kết luận, không thêm crate hay gói npm, không đổi `tauri.conf.json`, `capabilities/**` hay `[profile.release]`.
- Không push tag `v*` và không tạo bản phát hành; chỉ chạy `workflow_dispatch`.
- Không commit `.db`, `.app`, `.msi`, `dist/`, `target/`, thư viện tạm.
- Không viết câu nào ngụ ý đã có số Windows mà thật ra chưa đo.
- Không đổi trạng thái `done` của ticket hay epic nào; việc đó là của Ice.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| NFR6 dưới trần | `inspect-release` trên `.app`/`.msi` | Ghi chú phát hành có 2 dòng × 2 hệ điều hành | Không có lỗi |
| NFR6 vượt trần | payload > 400.000.000 B | Đường release vẫn ghi đủ số; báo cáo ghi "thay đổi tầng PRD — Ice quyết" | Không đỏ build; trình cho Ice |
| Không đọc được kích thước | Thiếu tệp hoặc cây trống | Ghi chú không mang dòng 0 hay rỗng | Bước release đỏ, nêu tên tệp thiếu |
| Lượt đo NFR4/5 thiếu | Ít hơn 30 lần launch ok | Không có `REPORT.md` | `summarize.mjs` từ chối như hiện nay |

</frozen-after-approval>

## Code Map

- `_bmad-output/implementation-artifacts/6-18-ban-do/run.sh` (+ `build.sh`, `probe.js`, `summarize.mjs`): đo trọn NFR3 bench, NFR4 bằng `.app` release thật (10 phiên, lạnh/ấm) và NFR5 bằng `footprint` + RSS. `AURA_NFR_BENCH_SESSIONS` chỉ dùng cho lượt chẩn đoán. Số NFR3 đã có trong log bị git-ignore: p95 xấu nhất 115,8 ms. Dựng thư viện mất 204.894 ms.
- `src-tauri/tests/story_6_18_library.rs` (`#[ignore]`, `--profile bench-release`) là bộ dựng thư viện. Liên quan: `story_6_18_bench_transition.rs`, `story_6_18_debt_probes.rs`, `nfr_bench_wiring_contract.rs`, feature `nfr-bench` (`Cargo.toml:211`, `lib.rs:278`). Không sửa.
- Plan 6.18 (trong `context`): task 7–9 ở khoảng dòng 233–239 là phần việc còn lại. Mục 6.18 chuyển Windows sang B7 ở `deferred-work.md:10921`.
- PRD `prd-auratranslate.md`:
  - :914 NFR1
  - :916–918 NFR3–5
  - :926 và :932 NFR6 (WebView2 tách riêng)
  - :1037–1045 dự phóng
  - :1177–1179 A6–A8
  - :1222 Q4
- `spec-auratranslate.md:124-126,136` · `requirements.md:411` · `5-14-ban-do/REPORT.md` (README:39 đã ghi "sơ bộ").
- `scripts/inspect-release.mjs:61-70`: in bảng 3 hàng, không bao giờ đỏ. `inspectInstalledTree` (`scripts/lib/dict-fetch.mjs:134-163`) bỏ đường dẫn khớp `/webview2/i`.
- `.github/workflows/release.yml`:
  - :63 `fetch-dicts`
  - :75–116 inspect, hiện chỉ ghi vào `$GITHUB_STEP_SUMMARY`; Ice phải chép tay
  - :182 `release-checksums.mjs --notes` nối tiếp vào `.github/release-notes.md`
  - `workflow_dispatch` chỉ dựng và kiểm, không phát hành
- `scripts/check-deps.mjs`: chặn crate cấm và giữ allowlist plugin (:186), nhưng không kiểm giấy phép. #34 KHÔNG LÀM: không dùng cargo-deny hay license-checker.
- Spine §Stack: dòng 1165–1220 (50 hàng); câu "37 hàng (34+3)" ở :1270 đã cũ.
- `scripts/check-i18n.mjs`:
  - Kiểm A (~:966) quét `.rs`/`.vue`, `EXEMPT` :136
  - Hiện xanh: 147 `.rs` + 60 `.vue`
  - Không quét `.ts`; mục nợ `deferred-work.md:50` có chủ "Story 1.14 / 10.9"
- `scripts/check-tokens.mjs`:
  - Kiểm C (:1025–1150): 35 cặp, cả hai theme, sàn 4,5/3,0
  - Không có kiểm 3:1 cho thành phần không phải chữ (1.4.11): `outline` chưa đo; `ornament`/`draft`/`tm-rule` khoảng 2,4–2,6
  - Diff trong `ReviewSpans.vue` có gạch dưới và gạch ngang, nên không chỉ dựa vào màu (1.4.1)
- `src/panels/lookupTiming.ts`: harness NFR1 (n/p50/p95/p99/max), tắt mặc định, bật bằng `__auraLookupTiming.enable()`.
- `deferred-work.md` (chỉ grep):
  - 17 mục mở có `Chủ: Story 10.9` ở các dòng 36, 203, 410, 945, 1178, 1187, 5774, 5900, 8011, 8066, 8194, 8940, 9280, 9309, 9972, 11992, 12006
  - Thông báo tác giả: :393 (`Chủ: Ice`)
  - NFR17 hai hệ điều hành: :1513 (`Chủ: Ice`)
- `plan-tra-no-10-9.md`: lượt trước đã đóng 12 mục; 14 mục kia được giữ lại để chờ lượt đo này. Không làm lại các mục đã đóng.

## Tasks & Acceptance

**Execution:**
- [x] `6-18-ban-do/` — chạy `run.sh` trọn, trên máy nhàn → `REPORT.md`, `environment.txt`, TSV thô (task 7 của 6.18) -- bằng chứng NFR3/4/5 trên macOS.
- [x] Trình bảng số NFR3/4/5 cho Ice → ghi kết luận A6/A7/A8 vào PRD (:916–918, :1177–1179, đóng Q4 :1222), `spec-auratranslate.md`, `requirements.md`. Đánh dấu 5.14 là sơ bộ (task 8 của 6.18). Thêm một dòng trỏ sang 10.9 vào Implementation Notes của plan 6.18 -- AC Q4.
- [x] `scripts/inspect-release.mjs` + `.github/workflows/release.yml` — ghi bảng kích thước ra tệp; lượt tag nối hai dòng (payload, WebView2) của mỗi hệ điều hành vào ghi chú phát hành; lượt dispatch tải bảng lên làm artifact để `gh run download` đọc được. Thiếu số thì đỏ -- AC NFR6 "cả hai dòng vào ghi chú phát hành".
- [x] Test cho phần nối ghi chú (đặt cạnh test hiện có của `release-checksums`/`inspect-release`), có ca thiếu số -- canh nhánh lỗi.
- [x] Chạy `release.yml` bằng `workflow_dispatch` → đọc artifact → đối chiếu với 400.000.000 B → trình Ice số nâng trần. Ice ký thì sửa PRD :926 (và dự phóng :1037–1045 nếu còn trích trần cũ) tại chỗ bằng 🔵 -- AC NFR6, Q1.
- [x] `prd-auratranslate.md:914` — sửa hàng NFR1 tại chỗ bằng 🔵 theo Q2; đóng :945, :1178, :1187 bằng `KHÔNG LÀM` -- Q2.
- [x] NFR15: rà một lần toàn bộ cây phụ thuộc. Rust dùng `cargo metadata`; npm chỉ lấy gói `dependencies` được đóng gói; trường nào không có `license` thì đọc tệp giấy phép trong mã nguồn đã tải. Đối chiếu bảng Stack với các phụ thuộc trực tiếp, sửa trong spine những hàng lệch và câu ở :1270 -- AC NFR15, Q3.
- [x] NFR16 và tương phản: chạy lại `check:i18n` và `check:tokens`, ghi kết quả vào báo cáo; chuyển mục nợ :50 sang `Chủ: Ice` -- AC NFR16/WCAG, Q4/Q5.
- [x] `deferred-work.md` — mỗi mục trong 17 mục `Chủ: Story 10.9` được đóng bằng số đo của lượt này hoặc chuyển chủ có tên. Thêm mục nợ mới:
  - `Chủ: Epic 10`: vòng một Chương không chạm chuột trên macOS (NFR17); duyệt bằng mắt tương phản hai theme trong WKWebView, kể cả Chế độ đọc và diff
  - `Chủ: B7`: các nửa Windows của NFR14/NFR17
  - Mục :393 (thông báo tác giả) giữ `Chủ: Ice`
- [x] Báo cáo nghiệm thu `_bmad-output/implementation-artifacts/10-9-ban-do/REPORT.md`: mỗi AC một hàng, gồm số, nguồn và kết luận -- tránh để kết luận rải rác.

**Acceptance Criteria:**
- Given bộ đo 6.18 chạy trọn trên máy nhàn, when `summarize.mjs` chạy, then `REPORT.md` có p95 cho từng kiểu truy vấn NFR3, NFR4 lạnh/ấm và NFR5 tính bằng B/MB/MiB, kèm điều kiện đo.
- Given Ice đã ký số, when mở PRD, then A6, A7, A8 mỗi cái có đúng một kết luận ghi "macOS", Q4 ghi đã đóng, và không còn hàng NFR3–5 nào mang nhãn ngưỡng tạm.
- Given lượt `workflow_dispatch` của `release.yml`, when tải artifact về, then có payload và WebView2 của macOS và Windows bằng byte, và bản xem trước ghi chú phát hành có đủ bốn số đó.
- Given bảng kích thước thiếu một số, when bước nối ghi chú chạy, then bước đó đỏ và nêu tên số thiếu.
- Given báo cáo rà giấy phép, when đọc, then mỗi gói có giấy phép SPDX kèm kết luận tương thích GPLv3, mọi phụ thuộc trực tiếp có một hàng trong bảng Stack, và mọi hàng Stack có phụ thuộc thật.
- Given `node scripts/check-debt-owner.mjs`, when chạy, then không còn mục mở hay 🟡 nào có chủ cuối là Story 10.9.
- Given PRD sau lượt này, when đọc hàng NFR1 và NFR6, then NFR1 không còn đòi đo đầu-cuối, NFR6 mang trần mới Ice đã ký kèm số đo hai dòng, và cả hai sửa tại chỗ bằng 🔵 có ngày.

## Implementation Notes

- Rà NFR15 không thấy hàng Stack nào lệch: 25 tên crate trực tiếp và 20 gói npm trực tiếp đều có đúng một hàng, phiên bản trùng lock. Sửa duy nhất là câu "37 hàng (34+3)" của spine. Chi tiết ở `10-9-ban-do/REPORT.md`.
- NFR1 sửa luôn giả định A1 của PRD (cùng vế đầu-cuối) và bản chép ở `spec-auratranslate/requirements.md` hàng NFR1.
- Of the 17 debt items only the NFR3 one closed on this run's numbers; seven asked for measurements the 6.18 bench never takes (Chapter open, URL preview, Glossary counters) and moved to `Chủ: Ice`.

- NFR6 notes: `inspect-release.mjs --json` writes `{os,installerBytes,payloadBytes,webview2Bytes}`; `release-checksums.mjs --sizes` (twice) validates and appends a `### Dung lượng (NFR6)` section, red on a missing OS, payload 0, or Windows WebView2 0. macOS WebView2 is legitimately 0 (WKWebView), so its line is a fixed statement, not a number: the "four non-zero numbers" AC holds as three numbers plus that statement.
- Windows WebView2 is not a file in the `msiexec /a` tree: it is the MSI Binary stream, read from the installer itself (Plan Change Log). Push of the branch also surfaced `check:panel-refs` and `check:clippy` reds left by 10-7/10-8 (never pushed); fixed in their own commit.
- The 6.18 run went at load 8.7–9.75 (a browser left running); Ice signed the numbers anyway, A7 carries that caveat in the PRD. NFR4 and NFR5 exceed; NFR6 ceiling raised to 500,000,000 B (A2 wrong a second time).
- `workflow_dispatch` has no `publish` job, so no notes preview is built there; the `nfr6-sizes-<OS>` artifacts (JSON) carry the bytes and Phase M can feed them to `release-checksums.mjs --sizes` locally for the preview.

## Plan Change Log

- 2026-10-21 — Lượt dispatch `39576340751` (`68cd7fa`) ra Windows WebView2 = 0 và payload 630.621.072 B. Bộ cài offline WebView2 (214.156.024 B) nằm trong bảng Binary của `.msi`, nên `msiexec /a` không giải nén nó; còn bản sao `.msi` mà ảnh quản trị để ở gốc thư mục đích bị cộng vào payload. Ice chọn đọc thẳng từ MSI: `scripts/lib/msi-streams.mjs` đọc bảng thư mục CFB và lấy stream `Binary.*webview2*`, còn `inspectInstalledTree` bỏ tệp `.msi` ở gốc khỏi payload. Sau đó chạy lại dispatch.

## Review Triage Log

- quick 1 · low · patch — `inspect-release.mjs` summary still printed the 400,000,000 ceiling; now 500,000,000.
- quick 2 · low · patch — notes printed a sentence, not a number, for macOS WebView2, so AC 3 had three numbers; the line now reads `0 byte (không nhúng, …)`.
- quick 3 · maybe-false (medium if true) · defer — `publish` never ran with `--sizes`; artifact layout matches the pinned `download-artifact` README; first tag release confirms (`deferred-work.md`, `Chủ: Ice`).
- quick 4 · low · patch — four 🔵 added mid-sentence (PRD NFR6 note ×2, A2, `spec-auratranslate.md` NFR6 line); moved to sentence start or dropped.
- quick 5 · false — the A2 row already ends with a dated 🔵 sentence saying A2 was wrong a second time, with the real 432/416 MB payloads.
- quick 6 · low · patch — SPEC A6–A8 kept "Ngưỡng tạm" / "Chưa đóng" and Q4 "VẪN MỞ"; reworded so the 5.14 text reads as the superseded preliminary.
- quick 7 · low · rejected — unchecked sector shift and unguarded DIFAT loop only bite on a corrupt `.msi` from our own build; the fix adds guards.
- quick 8 · medium · patch — tests covered only a v3 single-sector file with a mirrored encoder, while the real MSI is v4/4096 B with a two-sector directory and DIFAT starts near 457 MB; added the real raw stream name plus v4 and DIFAT cases, each counter-checked by a real removal.

## Design Notes

Ghi chú phát hành do máy nối, không chép tay: `.github/release-notes.md` là tệp tĩnh dùng cho mọi lần phát hành, nên một con số chép tay sẽ trôi sai ở bản kế tiếp. Lối này theo đúng tiền lệ `release-checksums.mjs --notes`.

## Verification

**Commands:**
- `bash _bmad-output/implementation-artifacts/6-18-ban-do/run.sh` -- expected: `summarize.mjs` in đủ 30 lần launch ok.
- `node scripts/check-debt-owner.mjs` -- expected: exit 0.
- `npm run check:i18n && npm run check:tokens && npm run check:deps` -- expected: exit 0.
- `gh workflow run release.yml` rồi `gh run download <id>` -- expected: bảng có 4 số byte khác 0.
