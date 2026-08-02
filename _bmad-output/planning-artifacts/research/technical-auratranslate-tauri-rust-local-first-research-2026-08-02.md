---
stepsCompleted: [1, 2, 3, 4, 5, 6]
inputDocuments:
  - '_bmad-output/planning-artifacts/briefs/brief-AuraTranslate-2026-08-02/brief.md'
  - '_bmad-output/planning-artifacts/briefs/brief-AuraTranslate-2026-08-02/addendum.md'
workflowType: 'research'
lastStep: 6
research_type: 'technical'
research_topic: 'AuraTranslate — nền tảng kỹ thuật cho translation workstation Tauri/Rust local-first (Anh/Trung → Việt, GPL)'
research_goals: 'Đóng gói và truy vấn Embedded Dictionary offline trong Tauri; xác minh bản quyền Thiều Chửu và truy nguồn VietPhrase; tách từ tiếng Trung và stemming/lemmatization tiếng Anh bằng Rust; thư viện diff cho Diff Viewer; tích hợp local LLM (Ollama/LM Studio) và BYOK; tương thích GPL của hệ sinh thái crate'
user_name: 'Ice'
date: '2026-08-02'
web_research_enabled: true
source_verification: true
---

# Research Report: technical

**Date:** 2026-08-02
**Author:** Ice
**Research Type:** technical

---

## Research Overview

Nghiên cứu kỹ thuật cho **AuraTranslate** — translation workstation desktop chạy local-first trên macOS và Windows, dịch Anh/Trung → Việt, xây bằng Tauri + Rust + SQLite, phát hành mã nguồn mở theo GPL. Nghiên cứu phủ sáu mục tiêu do Ice đặt: đóng gói và truy vấn Embedded Dictionary offline, xác minh xuất xứ dữ liệu từ điển, xử lý ngôn ngữ trong Rust, thư viện diff, tích hợp LLM, và tương thích giấy phép.

Kết luận tổng quát: **không có rào cản kỹ thuật nào chặn dự án.** Mọi thành phần đều có lời giải với giấy phép tương thích GPL. Nghiên cứu đảo ngược thứ tự ưu tiên rủi ro ban đầu — những thứ tưởng khó (tách từ tiếng Trung, kích thước bundle, khớp segment) đã có lời giải sẵn, trong khi bốn cái bẫy thật nằm ở chỗ mặc định của công cụ không khớp ca sử dụng: FTS5 mù tiếng Trung, plugin Stronghold bị khai tử, client SSE tự reconnect gây tính phí hai lần, và chất lượng dữ liệu từ điển Trung–Việt.

Tài liệu được xây theo sáu bước, mỗi bước có trích dẫn nguồn và thang tin cậy 🟢🟡🔴. Xem **Executive Summary** trong mục Tổng hợp ở cuối tài liệu để có toàn cảnh kết luận và năm khuyến nghị hành động.

---

<!-- Content will be appended sequentially through research workflow steps -->

## Technical Research Scope Confirmation

**Research Topic:** AuraTranslate — nền tảng kỹ thuật cho translation workstation Tauri/Rust local-first (Anh/Trung → Việt, GPL)

**Research Goals:**

1. Đóng gói Embedded Dictionary offline vào bundle Tauri và chiến lược truy vấn
2. Xác minh bản quyền Thiều Chửu và truy nguồn dữ liệu VietPhrase
3. Tách từ tiếng Trung và stemming/lemmatization tiếng Anh bằng Rust (Language-aware Matching)
4. Thư viện diff cho Diff Viewer
5. Tích hợp local LLM (Ollama / LM Studio) và BYOK
6. Tương thích GPL của hệ sinh thái crate

**Technical Research Scope:**

- Architecture Analysis — mô hình thiết kế, framework, kiến trúc hệ thống
- Implementation Approaches — phương pháp phát triển, mẫu code
- Technology Stack — ngôn ngữ, framework, công cụ, nền tảng
- Integration Patterns — API, giao thức, khả năng liên thông
- Performance Considerations — khả năng mở rộng, tối ưu hoá

**Research Methodology:**

- Dữ liệu web hiện hành, kiểm chứng nguồn nghiêm ngặt
- Xác thực đa nguồn cho mọi khẳng định kỹ thuật quan trọng
- Gắn mức độ tin cậy cho thông tin chưa chắc chắn
- Bộ lọc tương thích GPL áp lên mọi khuyến nghị

**Giới hạn đã thống nhất:** Mục tiêu 2 là câu hỏi pháp lý/xuất xứ, không phải kỹ thuật. Kết quả sẽ là bằng chứng thu thập được kèm mức độ tin cậy, **không phải kết luận pháp lý**.

**Rủi ro dự kiến cao nhất:** kích thước bundle từ điển; độ mỏng của hệ sinh thái tách từ tiếng Trung trong Rust; khả năng VietPhrase không có xuất xứ bản quyền rõ ràng.

**Scope Confirmed:** 2026-08-02

---

## Technology Stack Analysis

**Ghi chú điều chỉnh phạm vi:** khuôn mẫu chuẩn có mục *Cloud Infrastructure and Deployment*. AuraTranslate là local-first, không cloud sync — mục đó không áp dụng và được thay bằng *Domain-Specific Libraries* (NLP, diff, LLM) và *License Compatibility*, là những vùng thực sự quyết định dự án này.

**Thang tin cậy dùng trong tài liệu:** 🟢 Cao (nguồn phát biểu tường minh) · 🟡 Trung bình (suy luận từ nguồn) · 🔴 Cần kiểm chứng thêm.

### Programming Languages

Lựa chọn Rust + TypeScript trong brief được xác nhận là phù hợp, và quan trọng hơn — **tương thích với quyết định GPL**.

- **Rust (lõi backend)** — hệ sinh thái crate chủ yếu phát hành **dual MIT + Apache-2.0**. Bản thân trình biên dịch Rust và phần lớn crate công khai dùng mô hình này. 🟢
- **Lý do dual-license này quan trọng với bạn:** nó tồn tại *có chủ đích* để giữ tương thích GPLv2. MIT gần như là tập con của Apache-2.0, nhưng không hoàn toàn — và tương thích GPLv2 chính là trường hợp đặc biệt khiến MIT được giữ lại. 🟢
- **TypeScript + React/Vue (frontend)** — không có ràng buộc giấy phép đáng kể.

_Source: [Rationale of Apache dual licensing](https://internals.rust-lang.org/t/rationale-of-apache-dual-licensing/8952) · [credativ: Understanding Open Source Licenses](https://www.credativ.de/en/blog/credativ-inside/understanding-open-source-licenses-gpl-mit-apache-compared/)_

### Development Frameworks and Libraries

- **Tauri v2** — với file không thuộc frontend hoặc quá lớn để nội tuyến vào binary, dùng thuộc tính `resources` trong `tauri.conf.json`. File được đóng gói nằm ở `$RESOURCES/` và **giữ nguyên cấu trúc thư mục gốc**. Đây chính là cơ chế cho Embedded Dictionary. 🟢
- **Tauri SQL Plugin** — quản lý kết nối và truy vấn SQLite. Khuyến nghị khai báo `DATABASE_URL` trong module database với đường dẫn tương đối theo path context của Tauri. 🟢
- **Tối ưu kích thước:** JavaScript chiếm phần lớn dung lượng một app Tauri điển hình; dùng Vite/webpack/rollup có cấu hình minify đúng. 🟢

_Source: [Tauri v2 — Embedding Additional Files](https://v2.tauri.app/develop/resources/) · [Tauri v2 Performance and Bundle Size Optimization](https://www.oflight.co.jp/en/columns/tauri-v2-performance-bundle-size) · [Tauri SQLite Discussion #5440](https://github.com/tauri-apps/tauri/discussions/5440)_

### Database and Storage Technologies

**🔴 PHÁT HIỆN QUAN TRỌNG NHẤT CỦA BƯỚC NÀY — FTS5 mặc định không dùng được cho tiếng Trung.**

| Tokenizer | Hành vi | Dùng được cho |
|---|---|---|
| `unicode61` (mặc định FTS5) | Tách từ theo **khoảng trắng** | ✅ Tiếng Anh; ✅ **Tiếng Việt** (xử lý dấu và viết thường đúng) |
| `unicode61` với tiếng Trung | Tiếng Trung **không có ranh giới từ** → toàn bộ chuỗi 10 ký tự `變壓器絕緣油試驗規範` bị coi là **một token duy nhất** | ❌ Vô dụng |
| `trigram` (từ SQLite 3.34.0, 2020) | Lập chỉ mục chuỗi ba ký tự → khớp chuỗi con nhanh | ✅ CJK, đổi lại **chỉ mục lớn hơn** |

**Chiến lược lai được khuyến nghị:** dùng `unicode61` cho Latin/Việt và `trigram` cho CJK song song; định tuyến truy vấn bằng hàm kiểu `hasCJK(query)`; với token CJK ngắn 1–2 ký tự thì fallback sang `LIKE`. 🟢

> **Hệ quả trực tiếp với AuraTranslate:** phát hiện này chi phối **cả hai** tính năng — full-text search của Library *và* Language-aware Matching của TM. Nếu thiết kế schema mà không tính tới điều này ngay từ đầu, việc sửa sau sẽ phải xây lại toàn bộ chỉ mục.

_Source: [Full-text CJK Search with SQLite FTS5: Trigram Tokenizer and Hybrid Strategy](https://zenn.dev/kanseilink/articles/kanseilink-fts5-trigram-cjk-20260507?locale=en) · [SQLite FTS5 Tokenizers: unicode61 and ascii](https://audrey.feldroy.com/articles/2025-01-13-SQLite-FTS5-Tokenizers-unicode61-and-ascii) · [SQLite FTS5 Extension](https://www.sqlite.org/fts5.html) · [SQLite FTS5 bigram fix for Chinese](https://dev.to/foxck016077/sqlite-fts5-wont-tokenize-chinese-heres-the-7-line-bigram-fix-that-did-4fcc)_

### Domain-Specific Libraries — NLP, Diff, LLM

#### Tách từ tiếng Trung

- **`jieba-rs`** — bản cài đặt Jieba bằng Rust. **Giấy phép MIT** ✅ tương thích GPL. 330.593 lượt tải/tháng, được dùng trong 162 crate, **nhanh hơn cppjieba 33%**, bản ổn định 0.10, đã dùng Rust edition 2024. 🟢
- Có feature `default-dict` bật sẵn — **cần theo dõi vì ảnh hưởng kích thước bundle**. 🟡
- **`opencc-jieba-rs`** — kết hợp Jieba với hệ từ điển OpenCC nhiều tầng, hữu ích nếu cần chuyển đổi phồn thể ↔ giản thể. MIT. 🟢

> Lo ngại ban đầu của mình về "hệ sinh thái tách từ tiếng Trung trong Rust mỏng" **đã được bác bỏ**. `jieba-rs` trưởng thành, nhanh, và giấy phép sạch.

_Source: [jieba-rs GitHub](https://github.com/messense/jieba-rs) · [jieba-rs on crates.io](https://crates.io/crates/jieba-rs) · [opencc-jieba-rs](https://github.com/laisuk/opencc-jieba-rs)_

#### Stemming tiếng Anh

- **`rust-stemmers`** — các thuật toán Snowball biên dịch sang Rust, phần lớn phát hành theo **BSD** ✅. Yêu cầu đầu vào đã viết thường sẵn. 🟢
- ⚠️ **Cảnh báo bảo trì:** tài liệu của `tantivy-stemmers` mô tả `rust-stemmers` là *"a less then alive library"* — tức ít được bảo trì. 🟡
- **`tantivy-stemmers`** — gom nhiều nguồn OSS, hỗ trợ nhiều ngôn ngữ hơn, đang được bảo trì tích cực. Là phương án thay thế. 🟢

> **🔴 Khoảng trống so với PRD v8.0:** PRD ghi *"Stemming/Lemmatization"*. Các crate trên chỉ làm **stemming** (cắt gốc từ theo quy tắc), **không phải lemmatization** (đưa về từ nguyên dạng theo từ điển). Lemmatization thật cần từ điển hình thái hoặc mô hình. Đây là điểm cần Ice quyết: chấp nhận stemming, hay đầu tư thêm cho lemmatization.

_Source: [rust-stemmers GitHub](https://github.com/CurrySoftware/rust-stemmers) · [tantivy-stemmers](https://crates.io/crates/tantivy-stemmers)_

#### Diff engine

| Crate | Đặc điểm | Giấy phép |
|---|---|---|
| **`similar`** (mitsuhiko) | Không phụ thuộc bên ngoài; diff ở mức **dòng, từ, ký tự và grapheme**. Xây cho thư viện snapshot testing `insta` | 🔴 Cần xác nhận (nhiều khả năng Apache-2.0) |
| **`dissimilar`** (dtolnay) | Dựa trên diff-match-patch của Google; thuật toán Myers kèm **semantic cleanup** để tăng khả năng đọc của con người bằng cách loại bỏ các điểm trùng khớp ngẫu nhiên | **Apache-2.0 + MIT dual** ✅ 🟢 |

> **Khuyến nghị:** `similar` phù hợp hơn về mặt kỹ thuật vì có **grapheme-level diff** — quan trọng với dấu tiếng Việt và ký tự Trung. Nhưng *semantic cleanup* của `dissimilar` chính là thứ Diff Viewer cần, vì người đọc là con người đang lướt tìm reviewer đã sửa gì. Đề xuất thử nghiệm cả hai trên dữ liệu thật trước khi chốt.

_Source: [similar GitHub](https://github.com/mitsuhiko/similar) · [dissimilar GitHub](https://github.com/dtolnay/dissimilar) · [dissimilar crates.io](https://crates.io/crates/dissimilar)_

#### Tích hợp LLM

**🟢 PHÁT HIỆN LÀM ĐƠN GIẢN HOÁ KIẾN TRÚC:** Ollama phơi ra API **tương thích OpenAI** tại `POST /v1/chat/completions`. Mọi client SDK viết cho OpenAI chạy được với Ollama chỉ bằng cách đổi `base_url`. LM Studio cũng vậy.

> **Hệ quả:** yêu cầu "AI mở — BYOK **và** local LLM" trong PRD v8.0 **không cần hai đường code riêng**. Một HTTP client duy nhất trỏ tới endpoint tương thích OpenAI phục vụ được cả cloud lẫn local. Đây là một trong những phần tưởng phức tạp nhất của PRD hoá ra lại đơn giản nhất.

- **`ollama-rs`** — thư viện Rust cho Ollama API, hỗ trợ tool use và điều phối chat. 🟢
- **`ferrous-llm-ollama`** — cài đặt đầy đủ API local của Ollama: chat completion, sinh văn bản, streaming, embedding. 🟢
- **Cân nhắc:** dùng crate chuyên biệt sẽ trói vào Ollama; dùng `reqwest` thẳng tới endpoint tương thích OpenAI thì phủ được cả BYOK cloud lẫn local với một đường code. 🟡

_Source: [Using Ollama with Rust 2026](https://rustify.rs/articles/rust-ollama-local-llm-integration-2026) · [LM Studio & Ollama OpenAI API Docs](https://www.promptquorum.com/local-llms/local-llm-openai-compatible-api) · [ollama-rs](https://crates.io/crates/ollama-rs) · [ferrous-llm-ollama](https://crates.io/crates/ferrous-llm-ollama)_

### Development Tools and Platforms — Lưu trữ khoá BYOK

**🔴 CẢNH BÁO: đừng dùng Stronghold.**

| Phương án | Đánh giá |
|---|---|
| **`tauri-plugin-stronghold`** | ⛔ **Không còn được khuyến nghị, sẽ bị loại bỏ ở Tauri v3.** Phần lớn hướng dẫn cũ vẫn chỉ dùng cái này 🟢 |
| **`tauri-plugin-keyring`** | ✅ **Khuyến nghị.** Truy cập keychain/keyring gốc của hệ điều hành (macOS Keychain, Windows Credential Manager) để lưu API key an toàn 🟢 |
| **`tauri-plugin-store`** | Lưu key-value thường — ❌ **không dùng cho bí mật** 🟢 |

_Source: [Tauri Stronghold plugin](https://v2.tauri.app/plugin/stronghold/) · [tauri-plugin-keyring (HuakunShen)](https://github.com/HuakunShen/tauri-plugin-keyring) · [tauri-plugin-keyring (charlesportwoodii)](https://github.com/charlesportwoodii/tauri-plugin-keyring/tree/master) · [Tauri Store plugin](https://v2.tauri.app/plugin/store/)_

### License Compatibility — ràng buộc bao trùm mọi lựa chọn

| Giấy phép nguồn | Đưa vào GPLv2 | Đưa vào GPLv3 |
|---|---|---|
| **MIT** | ✅ Được | ✅ Được |
| **Apache-2.0** | ❌ **Không** — GPLv2 không chấp nhận điều khoản sáng chế của Apache | ✅ Được |
| **BSD** | ✅ Được | ✅ Được |

**🔑 Quyết định phái sinh mà Ice cần đưa ra: GPLv2 hay GPLv3?**

- Dữ liệu FVDP phát hành theo **"GPL v2 hoặc bất kỳ phiên bản sau nào"** — nên Ice **được quyền chọn v3**. 🟢
- Nếu chọn **GPLv3**: mọi crate Apache-2.0 dùng thoải mái. Rủi ro giấy phép gần như bằng không.
- Nếu chọn **GPLv2**: crate chỉ có Apache-2.0 (không kèm MIT) sẽ **không dùng được**. Thực tế phần lớn crate Rust là dual MIT+Apache nên nhánh MIT vẫn cứu được — nhưng đây là một bãi mìn không cần thiết.
- ⚠️ Lưu ý: tác giả GPLv3 coi việc **chỉ liên kết (linking)** tới phần mềm GPLv3 đã tạo ra tác phẩm phái sinh. 🟢

> **Khuyến nghị: chọn GPLv3.** Nó tương thích với toàn bộ hệ sinh thái Rust mà không cần kiểm tra từng crate xem có nhánh MIT hay không.

_Source: [Apache License v2.0 and GPL Compatibility (ASF)](https://www.apache.org/licenses/GPL-compatibility.html) · [credativ: GPL, MIT, Apache Compared](https://www.credativ.de/en/blog/credativ-inside/understanding-open-source-licenses-gpl-mit-apache-compared/) · [Apache License — Wikipedia](https://en.wikipedia.org/wiki/Apache_License)_

### Tổng hợp tương thích GPL của stack đề xuất

| Thành phần | Crate/Công nghệ | Giấy phép | Trạng thái |
|---|---|---|---|
| Tách từ tiếng Trung | `jieba-rs` | MIT | ✅ |
| Stemming tiếng Anh | `rust-stemmers` / `tantivy-stemmers` | BSD | ✅ |
| Diff engine | `dissimilar` | Apache-2.0 + MIT | ✅ |
| Diff engine (thay thế) | `similar` | 🔴 Cần xác nhận | Chờ |
| Lưu khoá BYOK | `tauri-plugin-keyring` | 🔴 Cần xác nhận | Chờ |
| LLM client | `ollama-rs` / `reqwest` | 🔴 Cần xác nhận | Chờ |
| Từ điển Anh-Việt | FVDP / OVDP | GPL v2+ | ✅ (là lý do chọn GPL) |
| Từ điển đối chiếu | CC-CEDICT | CC-BY-SA 4.0 | ✅ |

### Khoảng trống nghiên cứu còn lại sau bước này

1. 🔴 **Chưa có số liệu kích thước thật** của từng bộ từ điển và tổng bundle — sẽ xử lý ở bước Performance.
2. 🔴 **Chưa xác nhận giấy phép** của `similar`, `tauri-plugin-keyring`, `ollama-rs`.
3. 🔴 **Bản quyền Thiều Chửu và xuất xứ VietPhrase** chưa động tới — dành cho bước Implementation Research.
4. 🟡 **Lemmatization thật** (khác stemming) chưa có phương án trong hệ sinh thái Rust.

---

## Integration Patterns Analysis

**Ghi chú điều chỉnh phạm vi:** khuôn mẫu chuẩn của bước này bao gồm microservices, service mesh, ESB, service discovery, circuit breaker, saga, Kafka/RabbitMQ, OAuth 2.0, mutual TLS. **Không mục nào áp dụng cho một desktop app local-first, một người dùng, không backend.** Ghi lại điều này để người đọc sau không tưởng là bỏ sót.

Với AuraTranslate tồn tại đúng **bốn ranh giới tích hợp thật**:

```
[ Frontend TS ] ←── ① Tauri IPC ──→ [ Rust Core ] ←── ② HTTP/SSE ──→ [ LLM cloud/local ]
                                          │
                                          ├── ③ .docx / .md  ←→ Reviewer (Google Docs)
                                          └── ④ StarDict / CC-CEDICT → SQLite
```

### ① Tauri IPC — ranh giới trung tâm

| Cơ chế | Dùng khi | Ghi chú |
|---|---|---|
| **Command** | Yêu cầu–đáp ứng thường (tra từ điển, truy vấn TM) | Mặc định |
| **Channel API** (mới ở v2) | **Streaming khối lượng lớn** | Hiệu quả hơn hẳn việc phát hàng trăm event lẻ 🟢 |
| **`ipc::Response`** | Trả về `Vec<u8>` nhị phân thô | Cho dữ liệu nhị phân 🟢 |
| **Event** | Thông báo rời rạc | Không dùng cho luồng liên tục |

**Đặc tính hiệu năng đã xác minh:**

- Tauri **serialize mọi payload IPC thành JSON**. Với luồng số liệu tần suất cao, đây là nút thắt cổ chai. 🟢
- Object/array lớn hơn 10KB được serialize dưới dạng `JSON.parse('{...}')`; giới hạn an toàn khoảng **1GB**. 🟢
- Tồn tại `tauri-wire` — giao thức đóng khung nhị phân thay JSON, công bố **nhanh hơn 28–33 lần** khi mã hoá/giải mã và **nhỏ hơn 44%** trên đường truyền. 🟡 *(số liệu do chính dự án công bố, chưa có kiểm chứng độc lập)*

> **🔑 Đường nóng của AuraTranslate là Auto-Lookup.** Mỗi lần người dùng bôi đen một cụm từ là một vòng IPC. Đây là thao tác lặp lại nhiều nhất trong cả phiên làm việc, và brief hứa *"kết quả hiện ngay"*. Payload nhỏ nên JSON nhiều khả năng đủ nhanh — nhưng **đây là chỗ đầu tiên cần đo**, không phải chỗ để đoán.
>
> **Streaming bản dịch AI phải dùng Channel API**, không dùng event. Đây là đúng ca sử dụng mà Channel được thêm vào v2 để giải quyết.

_Source: [Tauri IPC and Frontend-Backend Communication](https://deepwiki.com/tauri-apps/tauri/3-ipc-and-communication) · [IPC in Tauri — Commands vs Custom IPC](https://dev.to/hiyoyok/ipc-in-tauri-tauri-commands-vs-custom-ipc-what-to-use-when-2ab4) · [Tauri IPC Improvements Discussion #5690](https://github.com/tauri-apps/tauri/discussions/5690) · [tauri-wire](https://github.com/userFRM/tauri-wire)_

### ② Giao thức LLM — HTTP + Server-Sent Events

Nối tiếp phát hiện ở bước 2 (Ollama/LM Studio tương thích OpenAI), tầng streaming đã rõ:

| Crate | Vai trò | Ghi chú |
|---|---|---|
| **`reqwest-sse`** | Mở rộng `reqwest` bằng method `.events()` | Nhẹ, ergonomic 🟢 |
| **`sseer`** | `response_to_stream` chuyển thẳng `reqwest::Response` thành `EventStream` | 🟢 |
| **`sse-rs`** | Client SSE **tự kết nối lại** | ⚠️ Xem cảnh báo dưới |
| **`async-openai`** | Thư viện Rust không chính thức cho OpenAI, bám theo OpenAPI spec, hỗ trợ SSE | Trói vào hình dạng API của OpenAI 🟡 |

> **⚠️ Đừng dùng client SSE tự kết nối lại.** Tài liệu của `sseer` nêu rõ: với API kiểu OpenAI, **bạn không thể nối lại một luồng đã đứt**. Tự động reconnect sẽ tạo ra một yêu cầu mới hoàn toàn — người dùng bị tính phí hai lần và nhận về văn bản trùng lặp. Xử lý đứt luồng phải là quyết định tường minh của ứng dụng, không phải hành vi ngầm của thư viện.

**Đường ống streaming đầy đủ:**

```
LLM endpoint ──SSE──▶ Rust (reqwest-sse) ──Channel──▶ Frontend ──▶ Panel 3
```

**Nguyên tắc bảo mật của ranh giới này:** API key **không bao giờ được đi qua IPC sang frontend**. Key nằm trong keyring của hệ điều hành, được Rust đọc, và chỉ Rust gọi ra ngoài. Frontend chỉ biết "đang stream" và nhận token.

_Source: [reqwest-sse](https://docs.rs/reqwest-sse/latest/reqwest_sse/) · [sseer](https://docs.rs/sseer/latest/sseer/) · [sse-rs](https://github.com/PizzasBear/sse-rs) · [async-openai](https://github.com/kitalia/async-openai) · [OpenAI streaming Rust demo](https://github.com/a-poor/openai-stream-rust-demo)_

### ③ Định dạng file — cầu nối tới Reviewer

Yêu cầu từ brief: xuất `.docx` **dạng bảng hai cột** (gốc | dịch), và `.md` giữ liên kết ảnh.

| Crate | Đánh giá | Giấy phép |
|---|---|---|
| **`docx-rs`** | Crate DOCX phổ biến nhất — **1 triệu+ lượt tải, 500+ sao**, đọc và ghi | **MIT** ✅ 🟢 |
| **`rdocx`** | Mới hơn, API cấp cao lấy cảm hứng từ python-docx: đoạn văn, **bảng**, ảnh, header/footer, style, list. Có sẵn layout engine render ra PDF/HTML/Markdown | 🔴 Cần xác nhận |
| **`ooxml-rs`** | ⛔ Hiện **chỉ hỗ trợ XLSX** — không dùng được cho Word | — |

> **Khuyến nghị:** `docx-rs` cho bản đầu vì độ trưởng thành và giấy phép MIT đã xác nhận. `rdocx` có API bảng thân thiện hơn cho yêu cầu hai cột — đáng đánh giá, nhưng phải xác nhận giấy phép trước.
>
> **Phần khó không nằm ở thư viện.** Yêu cầu *Structural Index Mapping* trong PRD — khớp lại đoạn văn khi import ngược file reviewer đã sửa — là bài toán riêng của bạn, không crate nào giải hộ. Reviewer có thể gộp đoạn, tách đoạn, hoặc xoá hẳn. Đây là rủi ro cài đặt thật, cần thiết kế ở bước Architecture.

_Source: [docx-rs](https://docs.rs/docx-rs/latest/docx_rs/) · [rdocx](https://lib.rs/crates/rdocx) · [ooxml-rs](https://github.com/zitsen/ooxml-rs)_

### ④ Định dạng dữ liệu từ điển — và một quyết định kiến trúc

**Cấu trúc StarDict** (định dạng OVDP phân phối) đã xác minh: 🟢

| File | Nội dung |
|---|---|
| `.ifo` | Metadata dạng văn bản thuần `option=value`: version, wordcount, bookname, idxfilesize, date, sametypesequence |
| `.idx` | Nhị phân, mỗi bản ghi: `[Word: n bytes][0: 1 byte][Article-start: 4 bytes][Article-length: 4 bytes]` |
| `.dict` | UTF-8 thuần, các mục từ nối tiếp nhau không phân tách; thường nén dzip thành `.dict.dz` |
| `.syn` | Từ đồng nghĩa |

**Crate Rust sẵn có:** `stardict` (có module ifo/idx/dict/error) và `opendict-rs` (đọc **cả StarDict lẫn MDict** qua một API thống nhất). 🟢

> ### 🔑 Quyết định kiến trúc cần Ice chọn: đọc StarDict lúc chạy, hay chuyển sang SQLite lúc build?
>
> | | Đọc StarDict lúc chạy | **Chuyển sang SQLite lúc build** |
> |---|---|---|
> | Bundle | Giữ nguyên file gốc | Một file `.db` duy nhất |
> | Truy vấn | Qua crate riêng, ngoài SQL | Cùng một tầng SQL với TM/Glossary/Library |
> | FTS5 | Không dùng được | ✅ Tích hợp thẳng |
> | Gộp nhiều từ điển | Phải tự viết tầng hợp nhất | ✅ Một bảng, cột `source` |
> | **Ghi nguồn định nghĩa** | Phải tự làm | ✅ Tự nhiên |
>
> **Khuyến nghị mạnh: chuyển sang SQLite ở bước build.** Lý do quyết định không phải hiệu năng mà là **nguyên tắc nền của brief** — *panel Lookup luôn hiển thị nguồn của mỗi định nghĩa*. Khi mọi từ điển nằm chung một schema có cột `source`, yêu cầu đó là hệ quả tự nhiên của mô hình dữ liệu. Còn nếu đọc từng định dạng riêng lúc chạy, bạn phải tự xây tầng hợp nhất và tự gắn nhãn nguồn — làm lại đúng thứ SQLite cho không.
>
> Kèm theo: chỉ cần một công cụ chuyển đổi chạy lúc build, không cần parser StarDict trong bản phát hành.

_Source: [StarDict File Format (chính thức)](https://github.com/huzheng001/stardict-3/blob/master/dict/doc/StarDictFileFormat) · [stardict crate](https://docs.rs/stardict) · [opendict-rs](https://crates.io/crates/opendict-rs) · [GoldenDict-ng Dictionary Formats](https://xiaoyifang.github.io/goldendict-ng/dictformats/) · [Notes about StarDict format](http://dhyannataraj.github.io/blog/2010/10/04/Notes-about-stardict-dictionry-format/)_

### Tổng hợp giấy phép bổ sung sau bước này

| Thành phần | Crate | Giấy phép | Trạng thái |
|---|---|---|---|
| Xuất/nhập .docx | `docx-rs` | **MIT** | ✅ |
| Xuất/nhập .docx (thay thế) | `rdocx` | 🔴 Cần xác nhận | Chờ |
| Streaming SSE | `reqwest-sse` / `sseer` | 🔴 Cần xác nhận | Chờ |
| Parser từ điển (chỉ lúc build) | `stardict` / `opendict-rs` | 🔴 Cần xác nhận | Chờ — nếu chỉ dùng ở build tool thì **không ràng buộc bản phát hành** |

### Rủi ro tích hợp đã nhận diện

1. 🔴 **Structural Index Mapping** — khớp lại đoạn khi import file reviewer đã sửa. Không thư viện nào giải; reviewer có thể gộp/tách/xoá đoạn. Rủi ro cài đặt cao nhất trong nhóm tích hợp.
2. 🟡 **Auto-Lookup là đường nóng nhất** và đi qua JSON IPC. Cần đo sớm, không đoán.
3. 🟢 **Đứt luồng SSE** — phải xử lý tường minh, tuyệt đối không dùng auto-reconnect.

---

## Architectural Patterns and Design

**Ghi chú điều chỉnh phạm vi:** khuôn mẫu chuẩn tập trung vào microservices/serverless/cloud-native, horizontal scaling, load balancing, distributed consensus. Với ứng dụng một người dùng chạy trên một máy, **các mục đó không áp dụng**. Bốn vùng kiến trúc thật của AuraTranslate được phân tích dưới đây.

### System Architecture Patterns — mô hình bảo mật Tauri v2

Tauri v2 thay hệ thống allowlist của v1 bằng mô hình ba tầng, và điều này **định hình cấu trúc ứng dụng chứ không chỉ là cấu hình bảo mật**.

**Nguyên tắc nền: default-deny.** Mọi command và tài nguyên bị chặn cho tới khi được cho phép tường minh. Tauri v2 **coi webview là không đáng tin cậy theo mặc định** và buộc mọi bề mặt IPC phải khai báo: cửa sổ nào được gọi command nào, với scope nào. 🟢

| Tầng | Vai trò | Nơi khai báo |
|---|---|---|
| **Capabilities** | File JSON gắn quyền vào cửa sổ hoặc nền tảng cụ thể | `src-tauri/capabilities/` |
| **Permissions** | Cho phép hoặc từ chối từng command và tính năng | Định danh kiểu `dialog:default`, `shell:allow-open` |
| **Scopes** | Giới hạn *dữ liệu/tài nguyên* mà command được thao tác; nhận bất kỳ kiểu nào serde serialize được | Trong capability |

**Kiến trúc plugin-first:** ở v1, dialog/notification/shell là lõi. Ở v2 chúng là **crate Cargo và package JS riêng biệt**, mỗi cái có định danh quyền riêng. 🟢

> **Hệ quả với AuraTranslate:**
>
> 1. **Scope là công cụ thực thi lời hứa local-first.** Brief cam kết *"không ai đọc được tài liệu của bạn"*. Scope cho phép giới hạn truy cập hệ thống file **chỉ trong thư mục dự án** — biến cam kết đó từ lời văn thành ràng buộc do framework cưỡng chế.
> 2. **Cấu trúc cửa sổ là quyết định bảo mật, không chỉ là quyết định UX.** Nếu Workspace và Library nằm ở hai cửa sổ khác nhau, mỗi cửa sổ có capability riêng — Library chỉ đọc có thể bị cấm hoàn toàn quyền ghi file.
> 3. Mọi plugin đã chọn (`sql`, `keyring`, `fs`, `dialog`) đều cần khai báo quyền tường minh. Đây là việc phải làm, không phải tuỳ chọn.

_Source: [Tauri Security and Capabilities System](https://deepwiki.com/tauri-apps/tauri-docs/5.8-security-and-capabilities-system) · [Tauri Capabilities](https://v2.tauri.app/security/capabilities/) · [Tauri Permissions](https://v2.tauri.app/security/permissions/) · [Tauri ACL Capability reference](https://v2.tauri.app/reference/acl/capability/)_

### Data Architecture Patterns — SQLite dưới áp lực auto-save

Brief yêu cầu *"tự động lưu định kỳ mà không làm gián đoạn UI"*. Đây là ràng buộc kiến trúc, không phải chi tiết cài đặt.

**WAL mode** — đổi `PRAGMA journal_mode = WAL` ngay sau khi mở kết nối. Thay đổi được ghi vào file `.db-wal` riêng; **người đọc và người ghi không chặn nhau**. Nhiều ứng dụng thấy hiệu năng tăng 10 lần trở lên. 🟢

**⚠️ Nhưng WAL không cho ghi song song.** SQLite chỉ cho **một transaction ghi tại một thời điểm**, kể cả ở WAL mode.

> **🔑 Khuyến nghị áp dụng thẳng cho AuraTranslate:** với desktop app có auto-save và **chỉ một tiến trình dùng database**, nên dùng **hàng đợi ghi ở tầng ứng dụng** thay vì dựa vào `SQLITE_BUSY`. Cách này cho thứ tự ghi công bằng hơn và tránh hiện tượng writer starvation. 🟢
>
> Đây đúng là hoàn cảnh của bạn: một tiến trình, auto-save nền, người dùng gõ liên tục ở panel Editor.

| Pragma / kỹ thuật | Vì sao quan trọng |
|---|---|
| `journal_mode = WAL` | Đọc/ghi không chặn nhau |
| `busy_timeout` | Không có nó, ghi đồng thời **thất bại tức thì** thay vì chờ khoá. Một nguồn dẫn chứng tỉ lệ crash giảm **40%** chỉ nhờ pragma này 🟡 |
| Auto-checkpoint (mặc định 1000 trang) | Có thể gây **gai trễ (latency spike)** — nguy hiểm cho lời hứa "không gián đoạn UI" |
| **WAL2** (SQLite 3.37+) | Dùng hai file WAL, cho phép checkpoint **mà không chặn người ghi** 🟢 |

_Source: [SQLite Optimizations For Ultra High-Performance (PowerSync)](https://powersync.com/blog/sqlite-optimizations-for-ultra-high-performance) · [The Write Stuff: Concurrent Write Transactions in SQLite](https://oldmoe.blog/2024/07/08/the-write-stuff-concurrent-write-transactions-in-sqlite/) · [SQLite in Production: WAL, Concurrency, VFS](https://micrologics.org/blog/sqlite-in-production-optimizing-wal-mode-concurrency-and-vfs-layers-for-low-latency-app-servers)_

### Data Architecture — Translation Memory theo chuẩn ngành

**Mô hình dữ liệu chuẩn đã được xác lập từ lâu:** TM lưu văn bản nguồn và bản dịch **chia thành segment**, các segment được **căn chỉnh (align)** với nhau thành **translation unit**. Mỗi translation unit lưu kèm metadata: lĩnh vực chủ đề, khách hàng, ngày tạo, tên người dịch. 🟢

> ### 🔑 PHÁT HIỆN QUAN TRỌNG NHẤT CỦA BƯỚC NÀY
>
> Ở bước 3 mình xếp **Structural Index Mapping** là *"rủi ro cài đặt cao nhất, không crate nào giải hộ"*. Điều đó **không còn đúng nữa**.
>
> Bài toán đó chính là **segment alignment** — một bài toán đã được giải trong ngành CAT tool, có công cụ riêng và **có mẫu UX đã được kiểm chứng**:
>
> > *Công cụ chia cả hai văn bản thành segment rồi tự động khớp. Người dùng sau đó **điều chỉnh cặp, gộp hoặc tách segment khi cần**, kiểm tra lại, rồi xuất ra TMX.*
>
> Nghĩa là bạn **không cần phát minh lại**, và quan trọng hơn — **đừng cố khớp tự động 100%**. Chuẩn ngành là *tự động khớp, rồi để con người sửa*. Chính xác cùng triết lý bạn đã chốt cho phần AI. Rủi ro hạ từ 🔴 xuống 🟡.

**TMX (Translation Memory eXchange)** — định dạng XML trung lập, chuẩn xuất/nhập TM giữa các CAT tool, đảm bảo tính khả chuyển. 🟢

_Source: [What is translation memory alignment? (POEditor)](https://poeditor.com/blog/what-is-translation-memory-alignment/) · [Maxprograms: Reuse translations with TM and TMX](https://www.maxprograms.com/articles/tmx.html) · [Translation memory systems (AIETI)](https://www.aieti.eu/enti/memories_ENG/entry.html) · [TMX — Grokipedia](https://grokipedia.com/page/Translation_Memory_eXchange)_

### Design Principles — đối chiếu với chuẩn local-first

Thuật ngữ *local-first* đến từ bài nghiên cứu năm 2019 của Ink & Switch, với **bảy nguyên tắc**. Đối chiếu AuraTranslate:

| # | Nguyên tắc | AuraTranslate | Ghi chú |
|---|---|---|---|
| 1 | Không có spinner — thao tác diễn ra cục bộ, cảm giác tức thì | ✅ | Embedded Dictionary phục vụ đúng điều này |
| 2 | Dữ liệu là của bạn — nằm trên máy bạn, ở định dạng bạn kiểm soát | ✅ | |
| 3 | Mạng là tuỳ chọn | ✅ | Chỉ AI cần mạng, và local LLM gỡ luôn ràng buộc đó |
| 4 | Cộng tác hoạt động (tự động giải quyết xung đột) | ⬜ **Không áp dụng** | Ice đã chốt: cộng tác qua trao đổi file, một người dùng |
| 5 | **Dữ liệu sống lâu hơn phần mềm — lưu ở định dạng mở, không độc quyền** | ⚠️ **Khoảng trống** | `.atproj` là định dạng độc quyền |
| 6 | Bảo mật và riêng tư mặc định | ✅ | Được Tauri scope cưỡng chế |
| 7 | Người dùng toàn quyền kiểm soát | ✅ | |

> **⚠️ Nguyên tắc 5 là khoảng trống duy nhất, và nó có lời giải sẵn: xuất TMX.**
>
> Brief đặt định dạng dự án `.atproj`. Nếu AuraTranslate ngừng phát triển — đúng số phận QuickTranslator — người dùng bị kẹt. Bổ sung **xuất TMX** cho phép mang toàn bộ TM sang OmegaT hay bất kỳ CAT tool nào. Với một dự án tự nhận là kế nhiệm của một công cụ đã chết, đây không phải tính năng phụ mà là **lập trường**.

> **🟢 Một gánh nặng bạn KHÔNG phải mang:** local-first thường gắn liền với **CRDT** để giải quyết xung đột nhiều người dùng. Vì Ice đã chọn một người dùng và cộng tác qua file, **AuraTranslate không cần CRDT** — tiết kiệm một khối lượng phức tạp khổng lồ. Cần nói rõ điều này ở tài liệu Architecture để không ai vô tình kéo nó vào.

_Source: [Local-first software: You own your data (Ink & Switch)](https://www.inkandswitch.com/local-first/) · [The Architecture Of Local-First Web Development (Smashing, 2026)](https://www.smashingmagazine.com/2026/05/architecture-local-first-web-development/) · [Local-First Software Principles](https://www.fernandohermida.com/notes/architecture/local-first-software-principles) · [Local-First Software in 2026](https://verity.salient.community/research/local-first-software-in-2026.html)_

### Cập nhật bảng rủi ro kiến trúc

| Rủi ro | Bước 3 | Sau bước 4 | Lý do |
|---|---|---|---|
| Structural Index Mapping | 🔴 Cao | 🟡 **Trung bình** | Là segment alignment — bài toán đã giải, có mẫu UX chuẩn |
| Auto-save gây gián đoạn UI | Chưa xét | 🟡 Trung bình | Có lời giải: WAL + hàng đợi ghi tầng ứng dụng + WAL2 |
| Khoá chặt người dùng vào `.atproj` | Chưa xét | 🟡 Trung bình | Có lời giải: xuất TMX |
| Cấu hình sai capability Tauri | Chưa xét | 🟡 Trung bình | Default-deny nghĩa là lỗi sẽ lộ ra sớm, không âm thầm |
| Auto-Lookup nghẽn qua JSON IPC | 🟡 | 🟡 Giữ nguyên | Vẫn cần đo thật |

---

## Implementation Approaches and Technology Adoption

**Ghi chú điều chỉnh phạm vi:** khuôn mẫu bước này hỏi về CI/CD doanh nghiệp, tổ chức đội ngũ, incident response, infrastructure-as-code, tối ưu chi phí đám mây. Với dự án mã nguồn mở do một người làm, phần lớn không áp dụng. Phần được giữ lại và đào sâu là **hai câu hỏi xuất xứ dữ liệu**, **chi phí phát hành thật**, và **trình tự xây dựng**.

### Xuất xứ dữ liệu ① — Thiều Chửu: nhiều khả năng ĐÃ vào phạm vi công cộng

**Dữ kiện đã xác minh:**

| Dữ kiện | Nguồn |
|---|---|
| Thiều Chửu = Nguyễn Hữu Kha, sinh 1902, **mất 1954** | Nhiều nguồn tiểu sử độc lập 🟢 |
| Luật SHTT Việt Nam 2022: quyền tác giả được bảo hộ **suốt cuộc đời tác giả + 50 năm sau khi mất** | Điều 2.A.2.5, phù hợp Công ước Berne 🟢 |

**Suy luận:** 1954 + 50 = **2004**. Quyền tài sản đối với *Hán Việt tự điển* (1942) nhiều khả năng đã hết hạn cuối năm 2004, tác phẩm vào phạm vi công cộng **từ 2005**. 🟡 *(suy luận pháp lý từ dữ kiện đã xác minh — không phải ý kiến pháp lý)*

> **⚠️ Ba điều kiện phải giữ:**
>
> 1. **Quyền nhân thân được bảo hộ vô thời hạn** ngay cả sau khi tác phẩm vào phạm vi công cộng. **Bắt buộc ghi công Thiều Chửu** trong ứng dụng và tài liệu. Đây không phải phép lịch sự mà là nghĩa vụ pháp lý.
> 2. Kết luận này áp dụng cho **bản gốc 1942**. Các bản tái bản có bổ sung chú giải, hiệu đính hay biên tập mới **có thể mang bản quyền riêng** của người biên tập.
> 3. Một **bộ dữ liệu số cụ thể** do ai đó số hoá và biên soạn có thể kèm tuyên bố quyền riêng đối với công sức số hoá. Cần kiểm tra từng nguồn tải, không chỉ tác phẩm gốc.

_Source: [Thiều Chửu Nguyễn Hữu Kha (1902-1954) — Tạp chí Nghiên cứu Phật học](https://tapchinghiencuuphathoc.vn/thieu-chuu-nguyen-huu-kha-1902-1954.html) · [Thư viện Hoa Sen](https://thuvienhoasen.org/a33711/thieu-chuu-nguyen-huu-kha-1902-1954-) · [Thời hạn bảo hộ quyền tác giả — SBLaw](https://vi.sblaw.vn/quyen-tac-gia-duoc-bao-ho-bao-lau/) · [Thư viện Pháp luật](https://thuvienphapluat.vn/cong-dong-dan-luat/thoi-han-bao-ho-quyen-tac-gia-202202.html)_

### Xuất xứ dữ liệu ② — VietPhrase: 🔴 KHÔNG XÁC ĐỊNH ĐƯỢC

Dự đoán ở bước 1 đã đúng. Kiểm tra trực tiếp kho mã nguồn QuickTranslator (`dynamotn/QuickTranslator`):

- ❌ **Không có file LICENSE**
- ❌ **Không có tuyên bố xuất xứ, ghi công, hay giấy phép** cho `Vietphrase.txt`
- 📦 Kho đã **lưu trữ đóng băng từ 30/12/2020**
- Dữ liệu do cộng đồng bồi đắp qua hơn một thập kỷ, không có người sở hữu xác định

🟢 *(kết luận về sự vắng mặt của giấy phép là quan sát trực tiếp, độ tin cậy cao)*

> ### 🔑 QUYẾT ĐỊNH CỦA ICE (2026-08-02): app phải hoạt động độc lập
>
> Ice bác đề xuất "chỉ ship importer" với lý do chính đáng: **không phải ai cũng cài QuickTranslator**, và bắt người dùng mới đi cài một phần mềm Windows đã chết chỉ để lấy dữ liệu là vô lý.
>
> Rủi ro xuất xứ vẫn được ghi nhận và không biến mất. Nhưng nghiên cứu bổ sung tìm được **lối đi thoả mãn yêu cầu của Ice mà vẫn có giấy phép sạch** — xem mục kế tiếp.

### Xuất xứ dữ liệu ③ — Từ điển Trung–Việt CÓ giấy phép (tìm thêm sau quyết định của Ice)

Tồn tại các bộ Trung–Việt cấp từ/cụm từ với giấy phép rõ ràng, đủ để **app chạy độc lập ngay sau khi cài, không cần QuickTranslator**:

| Nguồn | Nội dung | Giấy phép | ⚠️ Lưu ý chất lượng |
|---|---|---|---|
| **CVDICT** (`ph0ngp/CVDICT`) | **Hơn 122.000 từ và cụm từ tiếng Trung**, kèm phồn/giản thể và pinyin. Có cả từ hiện đại không có trong từ điển Hán Việt truyền thống. File `CVDICT.u8` | **CC BY-SA 4.0** ✅ | 🔴 **Phần lớn bản dịch do mô hình ChatGPT-4o tinh chỉnh thực hiện**, tác giả rà soát tay các mục đáng ngờ. Phái sinh từ CC-CEDICT |
| **`Trannosaur/published_dicts`** — `zhm2vi.tsv` | Trung → Việt, định dạng TSV, nguồn **Panlex 2017** | **CC BY-SA 4.0** ✅ | 🔴 **Trộn lẫn nghĩa Việt hiện đại và nghĩa Hán-Nôm mà không phân biệt** — ví dụ 四 cho ra cả "bốn" lẫn "tứ" |
| **`published_dicts`** — Việt–Trung | Nguồn zh.wiktionary.org | CC-BY-SA 3.0 ✅ | Cập nhật 07/2020 |
| **`published_dicts`** — Việt–Anh | Nguồn en.wiktionary.org / kaikki.org | CC-BY-SA 3.0 ✅ | **Cập nhật 01/2026** — mới hơn hẳn FVDP, đáng đánh giá |
| **`truyencuatui/VietPhrase`** | `VietPhrase.txt` trên GitHub | ❓ **Không rõ** | Dữ liệu cộng đồng, đã được kiểm chứng qua thực tế dịch truyện |

> ### 🔑 Chiến lược đề xuất: nền có giấy phép + lớp cộng đồng
>
> **Lớp nền (đóng gói sẵn, giấy phép sạch):** Unihan + Thiều Chửu + CVDICT + `zhm2vi` + CC-CEDICT + FVDP.
> → App **hoạt động đầy đủ ngay sau khi cài**, đúng yêu cầu của Ice, và không có mục nào trong bản phát hành phụ thuộc vào dữ liệu không rõ xuất xứ.
>
> **Lớp cộng đồng (VietPhrase):** vẫn đóng gói theo quyết định của Ice, nhưng **về mặt kiến trúc là một lớp có thể tháo rời** — kèm ghi công rõ ràng, nêu rõ đây là dữ liệu cộng đồng không xác định được tác giả, và có chính sách gỡ bỏ nếu chủ sở hữu lên tiếng.
>
> **Vì sao tách lớp lại quan trọng:** nếu sau này có tranh chấp về VietPhrase, bạn gỡ đúng lớp đó ra và **app vẫn chạy được**. Nếu trộn lẫn tất cả vào một khối, một vấn đề pháp lý sẽ hạ gục toàn bộ sản phẩm.

> ### ⚠️ Và đây chính là điều Ice lo ngại từ đầu — nay đã có bằng chứng
>
> Ice nêu ngay từ phiên brief: *"nhiều từ điển từ Trung Quốc có thể giải nghĩa sai"*. Nghiên cứu xác nhận mối lo đó **áp dụng cho cả các nguồn có giấy phép sạch**, chỉ là sai theo kiểu khác:
>
> - **CVDICT** — dịch máy bằng GPT-4o, chưa rà soát toàn bộ
> - **`zhm2vi`** — trộn nghĩa hiện đại với nghĩa Hán-Nôm không phân biệt
> - **VietPhrase** — không rõ xuất xứ, nhưng **được cộng đồng dịch giả kiểm chứng qua thực tế** nhiều năm
>
> **Kết luận:** không nguồn nào đáng tin tuyệt đối, và mỗi nguồn sai theo một kiểu riêng. Điều này **nâng nguyên tắc "luôn hiển thị nguồn của mỗi định nghĩa" từ tính năng hay lên thành yêu cầu bắt buộc**. Người dịch cần biết mình đang đọc nghĩa do máy dịch, nghĩa từ Panlex, hay nghĩa cộng đồng đã dùng hàng nghìn lần — vì mức độ tin cậy của ba loại đó khác nhau hoàn toàn.

_Source: [CVDICT (ph0ngp)](https://github.com/ph0ngp/CVDICT) · [Trannosaur/published_dicts](https://github.com/Trannosaur/published_dicts) · [truyencuatui/VietPhrase](https://github.com/truyencuatui/VietPhrase) · [viet-yomitan](https://github.com/onlyduyy/viet-yomitan)_

_Source: [dynamotn/QuickTranslator (GitHub, đã lưu trữ)](https://github.com/dynamotn/QuickTranslator) · [Trannosaur/published_dicts](https://github.com/Trannosaur/published_dicts)_

### Kích thước bundle — lo ngại ban đầu đã bị bác bỏ

Ở bước 1 mình cảnh báo bundle có thể phình tới **hàng trăm MB**. Số liệu thật khiêm tốn hơn nhiều:

| Bộ dữ liệu | Kích thước | Ghi chú |
|---|---|---|
| CC-CEDICT (thô) | **~4,3 MB** | 124.727 mục, bản 22/07/2026 |
| CC-CEDICT (StarDict) | ~6,8 MiB | bản 2014 |
| Unihan (văn bản) | **~7,9–10 MB** | |
| Unihan (StarDict) | ~8,6 MiB | bản 2014 |

🟢 Tổng riêng hai bộ này khoảng **15 MB**. Kể cả khi cộng thêm từ điển Anh-Việt của FVDP và tự điển Thiều Chửu, tổng tải trọng từ điển nhiều khả năng nằm trong khoảng **vài chục MB** — hoàn toàn chấp nhận được cho một desktop app, **không cần cơ chế tải về sau khi cài**.

> **Ghi chú:** sau khi chuyển sang SQLite (khuyến nghị ở bước 3), kích thước sẽ tăng do chỉ mục — đặc biệt là chỉ mục `trigram` cho CJK vốn được ghi nhận là **lớn hơn đáng kể**. Cần đo thật, nhưng xuất phát điểm thấp nên vẫn còn nhiều dư địa.

_Source: [CC-CEDICT download (MDBG)](https://www.mdbg.net/chinese/dictionary?page=cedict) · [Unihan Database (simonwiles.net)](https://simonwiles.net/projects/unihan/) · [UAX #38: Unicode Han Database](https://www.unicode.org/reports/tr38/)_

### Deployment và Operations — chi phí thật của việc phát hành desktop app

Đây là mục hay bị bỏ quên nhất, và là **chi phí tiền mặt định kỳ duy nhất** của dự án.

| Nền tảng | Yêu cầu | Ghi chú |
|---|---|---|
| **macOS** | Chứng chỉ ký từ **Apple Developer Program** | ⚠️ **Tài khoản Apple Developer miễn phí KHÔNG notarize được** — app vẫn hiện cảnh báo "chưa xác minh" khi mở 🟢 |
| **macOS — notarization** | Nộp app đã ký lên máy chủ Apple để quét mã độc tự động; đạt thì Apple cấp "ticket" đính kèm app | Bắt buộc nếu muốn người dùng mở được không cảnh báo 🟢 |
| **Windows** | Chứng chỉ **EV code signing** để loại bỏ hoàn toàn cảnh báo | 💰 **Trên 400 USD** và **bắt buộc token phần cứng** 🟢 |
| **Windows — thay đổi từ 06/2023** | CA không còn cấp chứng chỉ OV dạng file xuất được; chứng chỉ mới **phải nằm trên HSM** | 🟢 |

> ### 🔑 QUYẾT ĐỊNH CỦA ICE (2026-08-02): KHÔNG ký số, không đăng ký gì
>
> Ice chốt: **không có kinh phí cho việc này**. Mọi bản phát hành sẽ **không ký**. Ràng buộc này được chấp nhận và ghi lại. Không có phương án thay thế miễn phí nào cho notarization của Apple hay chứng chỉ EV của Windows — đây là ràng buộc thật, không phải thiếu sót cần khắc phục.
>
> **Hệ quả phải xử lý bằng thiết kế và tài liệu, không bằng tiền:**
>
> | Hệ quả | Cách giảm nhẹ (miễn phí) |
> |---|---|
> | macOS chặn app chưa ký/chưa notarize | Hướng dẫn cài đặt rõ ràng, có ảnh chụp màn hình: chuột phải → Mở, hoặc System Settings → Privacy & Security |
> | Windows SmartScreen cảnh báo | Uy tín tích luỹ dần theo lượt tải; hướng dẫn "More info → Run anyway" |
> | Người dùng nghi ngờ tính an toàn | Phát hành qua GitHub Releases, công bố **checksum SHA-256**, build công khai qua GitHub Actions để ai cũng kiểm chứng được binary khớp với mã nguồn |
> | Cộng đồng dịch giả phổ thông e ngại | Video hướng dẫn cài đặt; dựa vào uy tín truyền miệng trong cộng đồng — đúng cách QuickTranslator từng lan toả |
>
> **Ghi chú thẳng thắn:** đây là rào cản đón nhận có thật đối với người dùng không rành kỹ thuật, và không thể xoá bỏ bằng kỹ thuật. Nếu về sau dự án có nguồn tài trợ hoặc quyên góp từ cộng đồng, ký số nên là khoản chi ưu tiên hàng đầu. Thông tin về Azure Key Vault + `relic` được giữ lại dưới đây làm tham chiếu cho thời điểm đó.
>
> _Tham chiếu cho tương lai: phương án rẻ nhất cho lập trình viên độc lập là **Azure Key Vault** làm HSM đám mây kết hợp **`relic`** — công cụ ký mã nguồn mở xác thực với Azure Key Vault và ký file thực thi Windows._ 🟢

_Source: [Tauri macOS Code Signing](https://v2.tauri.app/distribute/sign/macos/) · [Tauri Windows Code Signing](https://v2.tauri.app/distribute/sign/windows/) · [Code Signing and Notarization for Cross-Platform Desktop Apps (KeyQ)](https://www.keyq.cloud/blog/code-signing-and-notarization-for-macos-desktop-apps/) · [Ship Your Tauri v2 App Like a Pro](https://dev.to/tomtomdu73/ship-your-tauri-v2-app-like-a-pro-code-signing-for-macos-and-windows-part-12-3o9n)_

---

## Technical Research Recommendations

### Implementation Roadmap

> **Đây là TRÌNH TỰ XÂY DỰNG, không phải cắt giảm phạm vi.** Ice đã chốt v1 gồm toàn bộ. Thứ tự dưới đây nhằm gỡ rủi ro sớm và đạt trạng thái dùng được sớm nhất có thể.

**Giai đoạn 0 — Ba mũi thăm dò trước khi cam kết kiến trúc**

Không viết code sản phẩm cho tới khi ba câu hỏi này được trả lời bằng số đo thật:

1. **Đo độ trễ Auto-Lookup qua IPC** — đường nóng nhất của app. JSON có đủ nhanh không?
2. **Dựng công cụ chuyển từ điển sang SQLite** — xác nhận giả định về kích thước và chỉ mục.
3. **Kiểm chứng chiến lược tokenizer lai** (`unicode61` + `trigram`) trên dữ liệu Trung và Việt thật.

**Giai đoạn 1 — Đã vượt QuickTranslator**

Embedded Dictionary + panel Source (kèm tab Hán Việt) + panel Lookup + Auto-Lookup.

> 🔑 **Chỉ riêng giai đoạn này đã là sản phẩm dùng được cho Ice** — nó làm được mọi thứ QuickTranslator làm, trên macOS. Đây là mốc có giá trị sớm nhất, và cũng là bằng chứng thuyết phục nhất để mời cộng đồng thử.

**Giai đoạn 2** — panel Editor + AI Translation (BYOK/local qua endpoint tương thích OpenAI) + Glossary + Smart RAG Injector
**Giai đoạn 3** — Library: mô hình dữ liệu, trạng thái vòng đời, tìm kiếm, chế độ đọc
**Giai đoạn 4** — Translation Memory + tái sử dụng segment + xuất TMX
**Giai đoạn 5** — Export/Import `.docx`/`.md` + segment alignment + Diff Viewer
**Giai đoạn 6** — AI Proofreader

### Technology Stack Recommendations

| Vùng | Khuyến nghị | Giấy phép |
|---|---|---|
| Giấy phép dự án | **GPLv3** (không phải v2 — tương thích Apache-2.0) | — |
| Tách từ tiếng Trung | `jieba-rs` | MIT ✅ |
| Stemming tiếng Anh | `tantivy-stemmers` (được bảo trì tốt hơn `rust-stemmers`) | BSD ✅ |
| Diff | `dissimilar` (semantic cleanup) hoặc `similar` (grapheme-level) — thử nghiệm cả hai | Apache+MIT ✅ / cần xác nhận |
| `.docx` | `docx-rs` | MIT ✅ |
| Client LLM | `reqwest` + `reqwest-sse` — **không dùng client tự reconnect** | cần xác nhận |
| Lưu khoá API | `tauri-plugin-keyring` — **không dùng Stronghold** | cần xác nhận |
| Từ điển | Chuyển sang SQLite lúc build; parser chỉ nằm trong build tool | không ràng buộc bản phát hành |
| Database | SQLite + FTS5 lai (`unicode61` + `trigram`), WAL, hàng đợi ghi tầng ứng dụng | — |
| Ký số Windows | Azure Key Vault + `relic` | — |

### Success Metrics và KPIs kỹ thuật

Bám theo tiêu chí thành công trong brief (chất lượng, không phải tốc độ):

| Chỉ số | Ngưỡng đề xuất | Vì sao |
|---|---|---|
| Độ trễ Auto-Lookup | Cảm giác tức thì với người dùng | Brief hứa "kết quả hiện ngay"; là thao tác lặp nhiều nhất |
| Gián đoạn UI khi auto-save | Không có gai trễ cảm nhận được | Yêu cầu tường minh trong brief |
| Tra cứu khi ngoại tuyến | 100% hoạt động không cần mạng | Điều kiện tồn tại của sản phẩm |
| Ghi nguồn định nghĩa | 100% mục từ hiển thị nguồn | Nguyên tắc nền của brief |
| Khả năng mang dữ liệu đi | Xuất được TMX | Nguyên tắc local-first số 5 |

### Risk Assessment và Mitigation — bảng tổng hợp cuối

| # | Rủi ro | Mức | Biện pháp |
|---|---|---|---|
| 1 | **VietPhrase không rõ xuất xứ** | 🟡 | Ice chốt đóng gói. Giảm nhẹ: tách thành **lớp có thể tháo rời**, ghi công rõ, có chính sách gỡ bỏ. Nền có giấy phép (CVDICT + `zhm2vi`) đảm bảo app vẫn chạy nếu phải gỡ |
| 1b | **Chất lượng dữ liệu từ điển** — CVDICT dịch máy GPT-4o; `zhm2vi` trộn nghĩa hiện đại với Hán-Nôm | 🔴 **Mới** | Không nguồn nào đáng tin tuyệt đối → **bắt buộc hiển thị nguồn mỗi định nghĩa**; VietPhrase làm đối trọng đã qua kiểm chứng thực tế |
| 2 | Bản quyền Thiều Chửu | 🟡 | Nhiều khả năng đã public domain (mất 1954 + 50 năm); **bắt buộc ghi công**; kiểm tra từng nguồn số hoá |
| 3 | **Phát hành không ký số** | 🟡 | Ice chốt: không kinh phí. Giảm nhẹ bằng tài liệu hướng dẫn cài, checksum SHA-256, build công khai qua GitHub Actions |
| 4 | Auto-Lookup nghẽn qua JSON IPC | 🟡 | Đo ở Giai đoạn 0; dự phòng `tauri-wire` |
| 5 | Segment alignment khi import | 🟡 | Bài toán đã giải trong ngành; mẫu UX: máy khớp, người sửa |
| 6 | Gai trễ auto-save | 🟡 | WAL + `busy_timeout` + hàng đợi ghi + WAL2 |
| 7 | Khoá người dùng vào `.atproj` | 🟡 → 🟢 | Xuất TMX |
| 8 | Chỉ mục trigram làm phình database | 🟡 | Đo ở Giai đoạn 0 |
| 9 | Không có lemmatization thật trong Rust | 🟢 | Stemming đủ cho khớp Glossary; ghi rõ giới hạn trong PRD |
| 10 | Phạm vi v1 quá lớn cho một người | 🔴 | Ngoài tầm xử lý kỹ thuật — là quyết định của Ice, đã ghi trong brief |

---
---

# Tổng hợp nghiên cứu — AuraTranslate

> **Ghi chú về cấu trúc:** khuôn mẫu bước tổng hợp yêu cầu dựng lại tài liệu thành 12 chương lặp lại nội dung năm bước trước. Nội dung đó **đã nằm nguyên vẹn ở trên**, nên nhân đôi chỉ làm tài liệu khó dùng. Phần này chỉ chứa những gì thực sự là *tổng hợp*: kết luận, nhận định xuyên suốt, nguồn, và bước tiếp theo.

## Executive Summary

Nghiên cứu kỹ thuật cho AuraTranslate xác nhận **stack đã chọn là đúng** và **không gặp rào cản kỹ thuật nào không vượt qua được**. Tauri v2 ổn định từ 10/2024, đang được dùng trong các sản phẩm thật như Hoppscotch, Spacedrive, AppFlowy, với mức đón nhận tăng 35% theo năm và được xem là **lựa chọn mặc định cho dự án desktop mới năm 2026**. App tối thiểu dưới 10 MB, RAM nhàn rỗi 20–100 MB — phù hợp với một công cụ chạy nền suốt ngày làm việc. 🟢

Sáu mục tiêu nghiên cứu đều có câu trả lời. Quan trọng hơn, nghiên cứu **thay đổi thứ tự ưu tiên rủi ro**: những thứ tưởng khó hoá ra đã có lời giải sẵn, còn những thứ tưởng đơn giản lại giấu bẫy.

**Phát hiện then chốt:**

- 🔴 **FTS5 mặc định không dùng được cho tiếng Trung.** Tokenizer `unicode61` coi cả câu tiếng Trung là một token. Cần chiến lược lai `unicode61` + `trigram`. Chi phối cả Library search lẫn TM matching — phải quyết trước khi thiết kế schema.
- 🟢 **"AI mở" là phần dễ nhất, không phải khó nhất.** Ollama và LM Studio đều phơi API tương thích OpenAI; BYOK và local LLM dùng chung một đường code.
- 🔴 **Mọi nguồn từ điển Trung–Việt đều có khiếm khuyết riêng.** CVDICT dịch máy bằng GPT-4o; `zhm2vi` trộn nghĩa hiện đại với Hán-Nôm không phân biệt; VietPhrase không rõ xuất xứ nhưng đã qua kiểm chứng thực tế.
- 🟢 **Structural Index Mapping không phải bài toán mới** — đó là *segment alignment*, đã có lời giải và mẫu UX chuẩn trong ngành CAT tool.
- ⛔ **Stronghold đã bị khai tử.** Phần lớn hướng dẫn Tauri vẫn chỉ dùng nó để lưu API key; phải dùng `tauri-plugin-keyring`.
- 🟢 **Bundle từ điển chỉ vài chục MB**, không cần cơ chế tải về sau khi cài.

**Năm khuyến nghị hành động:**

1. **Chọn GPLv3, không phải GPLv2** — tương thích với crate Apache-2.0, phủ toàn bộ hệ sinh thái Rust mà không cần kiểm tra từng gói.
2. **Chuyển mọi từ điển sang SQLite ở bước build**, không đọc định dạng gốc lúc chạy.
3. **Chạy ba mũi thăm dò ở Giai đoạn 0** trước khi viết code sản phẩm: đo độ trễ Auto-Lookup, dựng công cụ chuyển từ điển, kiểm chứng tokenizer lai trên dữ liệu thật.
4. **Tách VietPhrase thành lớp có thể tháo rời**, đặt trên nền các nguồn có giấy phép sạch.
5. **Bổ sung xuất TMX** để đóng khoảng trống nguyên tắc local-first số 5 — dữ liệu phải sống lâu hơn phần mềm.

_Source: [Tauri vs Electron 2026 (PkgPulse)](https://www.pkgpulse.com/guides/electron-vs-tauri-2026) · [Tauri v2 vs Electron 2026](https://www.buildmvpfast.com/blog/tauri-v2-vs-electron-desktop-apps-2026) · [Best Desktop App Frameworks 2026](https://www.pkgpulse.com/guides/best-desktop-app-frameworks-2026)_

## Mục lục

| # | Mục | Nội dung chính |
|---|---|---|
| 1 | Technical Research Scope Confirmation | Phạm vi, mục tiêu, giới hạn đã thống nhất |
| 2 | Technology Stack Analysis | Ngôn ngữ, framework, SQLite/FTS5, thư viện NLP, diff, LLM, tương thích GPL |
| 3 | Integration Patterns Analysis | Tauri IPC, giao thức LLM, `.docx`/`.md`, định dạng từ điển |
| 4 | Architectural Patterns and Design | Bảo mật Tauri v2, SQLite dưới auto-save, mô hình TM, đối chiếu local-first |
| 5 | Implementation Approaches | Xuất xứ Thiều Chửu và VietPhrase, nguồn có giấy phép, kích thước bundle, phát hành |
| 6 | **Tổng hợp** (mục này) | Kết luận, nhận định xuyên suốt, nguồn, bước tiếp theo |

## Nhận định xuyên suốt — thứ từng bước riêng lẻ không nhìn thấy

### ① Trực giác kỹ thuật ban đầu sai một cách có hệ thống

Ở bước 1 mình dự đoán ba rủi ro cao nhất. Kết quả:

| Dự đoán bước 1 | Thực tế | |
|---|---|---|
| Bundle phình hàng trăm MB | ~15 MB cho CC-CEDICT + Unihan | ❌ Sai |
| Hệ sinh thái tách từ tiếng Trung trong Rust mỏng | `jieba-rs`: MIT, 330k tải/tháng, nhanh hơn cppjieba 33% | ❌ Sai |
| VietPhrase không rõ xuất xứ | Không có LICENSE, kho đóng băng 2020 | ✅ Đúng |

Trong khi đó **không dự đoán nào chạm tới bốn cái bẫy thật**: FTS5 mù tiếng Trung, Stronghold bị khai tử, SSE auto-reconnect tính phí hai lần, và chất lượng dữ liệu từ điển.

> **Bài học cho các quyết định kỹ thuật còn lại của dự án:** rủi ro không nằm ở chỗ trông có vẻ khó, mà ở chỗ **mặc định của công cụ không khớp với ca sử dụng của bạn**. FTS5 *có* chạy với tiếng Trung — chỉ là ra kết quả vô nghĩa. Stronghold *có* trong tài liệu — chỉ là sắp bị xoá. Đây là loại lỗi chỉ lộ ra khi đo, không lộ ra khi đọc.

### ② Một quyết định gỡ được năm vấn đề

**Chuyển từ điển sang SQLite ở bước build** — quyết định có bán kính ảnh hưởng rộng nhất trong toàn bộ nghiên cứu:

| Giải quyết | Cách |
|---|---|
| Truy vấn thống nhất | Cùng tầng SQL với TM, Glossary, Library |
| Full-text search | FTS5 dùng được ngay |
| Gộp nhiều từ điển | Một bảng, nhiều nguồn |
| **Ghi nguồn định nghĩa** | Cột `source` — yêu cầu bắt buộc của brief |
| Rủi ro giấy phép parser | Parser chỉ nằm trong build tool, không vào bản phát hành |

### ③ Nguyên tắc sản phẩm hoá ra là yêu cầu kỹ thuật

Brief đặt nguyên tắc *"panel Lookup luôn hiển thị nguồn của mỗi định nghĩa"* như một lựa chọn triết lý — công cụ cung cấp bằng chứng, người dịch phán xét.

Nghiên cứu cho thấy **đó còn là điều bắt buộc về mặt kỹ thuật**. Mỗi nguồn Trung–Việt có giấy phép đều sai theo một kiểu khác nhau: một bộ dịch bằng máy, một bộ trộn tầng nghĩa, một bộ không rõ nguồn gốc nhưng đã qua thực chiến. Không có nguồn nào đúng để chọn làm "câu trả lời duy nhất".

> Mối lo Ice nêu ở phiên brief — *"từ điển từ Trung Quốc có thể giải nghĩa sai"* — đã được xác nhận, và còn rộng hơn dự đoán ban đầu.

### ④ Chuỗi ràng buộc nối tiếp

```
Chọn GPL  →  dùng được FVDP (GPL v2+)
          →  nhưng phải là GPLv3 để dùng crate Apache-2.0
Không kinh phí  →  không ký số
                →  niềm tin phải đến từ nơi khác
                →  build công khai + checksum SHA-256
```

Không quyết định nào trong ba cái trên là độc lập. Đây là điều cần ghi vào tài liệu Architecture để người đọc sau không vô tình gỡ một mắt xích.

## Phương pháp và kiểm chứng nguồn

**Phạm vi:** sáu mục tiêu do Ice đặt, phủ qua năm bước có cấu trúc, mỗi bước dùng tìm kiếm web song song.

**Loại nguồn đã dùng:** tài liệu chính thức (Tauri, SQLite, Unicode Consortium, Apache Software Foundation), kho mã nguồn và trang crate (GitHub, crates.io, lib.rs, docs.rs), bài phân tích kỹ thuật 2026, nguồn pháp lý Việt Nam về thời hạn bản quyền, và **kiểm tra trực tiếp kho mã nguồn** khi tìm kiếm không đủ (QuickTranslator, CVDICT, published_dicts).

**Thang tin cậy đã áp dụng xuyên suốt:** 🟢 Cao (nguồn phát biểu tường minh) · 🟡 Trung bình (suy luận) · 🔴 Cần kiểm chứng thêm.

**Giới hạn đã tuyên bố:**

- Phần bản quyền là **suy luận từ dữ kiện đã xác minh, không phải ý kiến pháp lý**. Với bản phát hành công khai, nên có luật sư xác nhận tình trạng các bản tái bản và bộ dữ liệu số hoá cụ thể.
- Chỉ số hiệu năng của `tauri-wire` do chính dự án công bố, chưa có kiểm chứng độc lập.
- Kích thước StarDict của CC-CEDICT và Unihan là số liệu 2014, có thể đã tăng.
- **Chưa xác nhận giấy phép** của: `similar`, `tauri-plugin-keyring`, `reqwest-sse`, `sseer`, `rdocx`, `ollama-rs`. Cần kiểm tra trước khi đưa vào dự án.
- Chiến lược kiểm thử và CI chưa được đào sâu — nằm ngoài sáu mục tiêu, thuộc phạm vi `bmad-testarch-framework`.

## Kết luận

**Không có rào cản kỹ thuật nào chặn AuraTranslate.** Mọi thành phần đều có lời giải, phần lớn có giấy phép tương thích GPL, và stack Tauri/Rust/SQLite đã được kiểm chứng trong sản phẩm thật.

Rủi ro còn lại **không nằm ở kỹ thuật**:

| Rủi ro | Bản chất |
|---|---|
| Phạm vi v1 gồm toàn bộ, một người làm | Sức bền và thời gian, không phải năng lực kỹ thuật |
| Phát hành không ký số | Rào cản đón nhận với người dùng phổ thông |
| VietPhrase không rõ xuất xứ | Pháp lý, đã giảm nhẹ bằng kiến trúc lớp tháo rời |
| Chất lượng dữ liệu từ điển | Đã chuyển thành tính năng: hiển thị nguồn |

**Bước tiếp theo được khuyến nghị:** chạy **Giai đoạn 0** — ba mũi thăm dò — trước khi đưa nghiên cứu này vào PRD. Ba con số đó (độ trễ Auto-Lookup, kích thước database sau khi chuyển đổi, chất lượng khớp của tokenizer lai) sẽ biến phần lớn phỏng đoán còn lại trong tài liệu này thành dữ kiện.

---

**Ngày hoàn thành nghiên cứu:** 2026-08-02
**Kiểm chứng nguồn:** mọi khẳng định kỹ thuật đều có trích dẫn nguồn hiện hành
**Mức tin cậy tổng thể:** Cao đối với các kết luận kỹ thuật; Trung bình đối với suy luận bản quyền (đã tuyên bố rõ giới hạn)

---

# Phụ lục A — Dữ liệu từ loại và ví dụ cách dùng (bổ sung 2026-08-02)

> **Bối cảnh:** Ice cung cấp ảnh chụp ứng dụng *Từ điển Hán Việt v7.3.2* và đặt câu hỏi: bộ từ điển này có tra được **từ loại** (động từ, tính từ, phó từ), **cách dùng theo từng từ loại**, và **ví dụ** hay không.
>
> **Câu hỏi này phát hiện một khoảng trống trong khuyến nghị ở Bước 5.** Phụ lục được bổ sung sau khi nghiên cứu chính đã hoàn tất.

## A.1 Khoảng trống bị bỏ sót

Brief đặc tả Panel 2 (Lookup) là *"hiển thị chi tiết giải nghĩa, **ngữ cảnh**, **ví dụ cách dùng**"*. Stack từ điển khuyến nghị ở Bước 5 — Thiều Chửu + Unihan + CVDICT + CC-CEDICT — **không nguồn nào trong số đó có hệ thống từ loại hay ví dụ cách dùng có cấu trúc**. Khuyến nghị cũ đủ để tra *nghĩa*, nhưng không đủ để thực hiện lời hứa của Panel 2.

## A.2 Đối chiếu sáu nguồn trong ảnh

| Nguồn | Từ loại | Cách dùng | Ví dụ |
|---|---|---|---|
| **Hán Việt Từ Điển Trích Dẫn** (Đặng Thế Kiệt) | ✅ **Đầy đủ**: danh từ, động từ, hình dung từ (tính từ), phó từ, đại từ, liên từ, giới từ, trợ từ, thán từ, trạng thanh từ | ✅ | ✅ **Tách rõ ba phần: định nghĩa / ví dụ / trích dẫn**, kèm ghi chú khi cần 🟢 |
| **Thiều Chửu** (1942) | ❌ Không có hệ thống từ loại | Giảng nghĩa lối cổ điển | Hạn chế |
| **MDBG Chinese-English** = **CC-CEDICT** | ❌ **Cố ý không dùng POS tag** — tự mô tả là *"human readable descriptive dictionary, not a resource intended for machine processing"*; thông tin ngữ pháp nhúng trong định nghĩa tiếng Anh 🟢 | Gián tiếp | Ít |
| **Unicode Consortium** = **Unihan** | ❌ Chỉ âm đọc và nghĩa ngắn | ❌ | ❌ |
| **Tự Điển Chữ Nôm** | Chữ Nôm — khác phạm vi | — | — |
| **Cổ hán văn** (Tam tự kinh, Thiên tự văn, Bách gia tính) | Ngữ liệu kinh điển, không phải từ điển | — | ✅ Làm nguồn trích dẫn |

> **Kết luận: chỉ duy nhất Hán Việt Từ Điển Trích Dẫn đáp ứng yêu cầu của Ice.** Cấu trúc *định nghĩa / ví dụ / trích dẫn* của nó gần như là schema có sẵn cho Panel 2.

## A.3 ⚠️ Rào cản: HVTĐTD còn bản quyền

| Dữ kiện | |
|---|---|
| Tác giả | **Đặng Thế Kiệt** |
| Bản quyền | **© DTK, Paris 2006–2009** — tác phẩm đương đại 🟢 |
| Tình trạng | **KHÔNG thuộc phạm vi công cộng.** Khác hẳn Thiều Chửu (1942) |
| Liên hệ công khai | `dang.thekiet2022@yahoo.com` (nêu trong ứng dụng) |

**Nhưng có tiền lệ rõ ràng:** chính ứng dụng trong ảnh ghi ở mục Cổ hán văn — *"được tham khảo **với sự cho phép của tác giả**"*. Nghĩa là **xin phép là con đường khả thi và đã có người đi**.

> ### 🔑 Khuyến nghị hành động có đòn bẩy cao nhất cho Panel 2
>
> **Liên hệ trực tiếp Đặng Thế Kiệt xin phép sử dụng dữ liệu.** Lý lẽ để trình bày:
> - AuraTranslate là dự án **mã nguồn mở, phi thương mại**, phát hành theo GPL
> - Phục vụ **cộng đồng dịch giả Việt Nam**, đúng đối tượng mà tác phẩm hướng tới
> - Cam kết **ghi công đầy đủ và hiển thị nguồn cho từng định nghĩa** — vốn đã là nguyên tắc nền của sản phẩm
>
> Đây là việc nên làm **sớm**, vì kết quả quyết định chất lượng Panel 2 và có thể mất thời gian chờ hồi âm.

## A.4 Phương án dự phòng có giấy phép: kaikki.org / Wiktextract

Nếu không xin được phép, đây là **nguồn duy nhất có giấy phép mở vừa có từ loại vừa có ví dụ**:

| Đặc điểm | |
|---|---|
| Nguồn | Wiktionary, trích xuất bằng **Wiktextract** |
| Cấu trúc | JSON máy đọc được — **mỗi dòng mô tả một part of speech** của một từ 🟢 |
| Nội dung | Lemma, dạng biến cách, bản dịch, từ nguyên, **usage examples**, phát âm, quan hệ từ vựng–ngữ nghĩa, chú giải hình thái/cú pháp/ngữ nghĩa/chủ đề/phương ngữ 🟢 |
| Bao phủ | Trích xuất từ Wiktionary tiếng Anh; độ phủ cho ngôn ngữ khác **thường ngang hoặc vượt** bản Wiktionary riêng của ngôn ngữ đó 🟢 |
| Giấy phép | **CC-BY-SA + GFDL** ✅ |
| Ghi công học thuật | Trích dẫn Tatu Ylonen, *Wiktextract: Wiktionary as Machine-Readable Structured Data*, LREC 2022 |

> Lưu ý: `published_dicts` — nguồn đã tìm được ở Bước 5 — lấy bộ Việt–Anh **chính từ kaikki.org**, cập nhật 01/2026. Nghĩa là đường dẫn dữ liệu này đã được kiểm chứng trong thực tế.

## A.5 Kiến trúc từ điển sau cập nhật

Cấu trúc bốn lớp, mỗi lớp một vai trò khác nhau:

| Lớp | Nguồn | Vai trò | Giấy phép |
|---|---|---|---|
| **Từ loại + ví dụ** | HVTĐTD *(nếu xin được phép)* | ⭐ Nội dung Panel 2 | Cần xin phép |
| **Từ loại + ví dụ** *(dự phòng)* | kaikki.org / Wiktextract | Nền có giấy phép | CC-BY-SA ✅ |
| **Ký tự / âm Hán Việt** | Thiều Chửu + Unihan | Tra ký tự, tab Hán Việt | PD / Unicode ✅ |
| **Từ và cụm từ** | CVDICT + `zhm2vi` + VietPhrase | Tra nhanh khi dịch | CC-BY-SA / cộng đồng |
| **Đối chiếu chéo** | CC-CEDICT | Ý kiến thứ ba | CC-BY-SA ✅ |

> **Hệ quả với mô hình dữ liệu:** schema SQLite cho từ điển **không thể chỉ là `(từ, nghĩa, nguồn)`**. Phải chứa được **từ loại**, **ví dụ** và **trích dẫn** như các trường riêng biệt — vì HVTĐTD tách chúng ra, và kaikki.org cũng vậy. Đây là thay đổi đáng kể so với giả định ngầm ở Bước 3, cần phản ánh vào PRD và Architecture.

## A.6 Xác nhận ngoài lề

Ứng dụng trong ảnh chạy trên đúng ba nguồn đã khuyến nghị ở Bước 2 và Bước 5 — **MDBG chính là CC-CEDICT**, **Unicode Consortium chính là Unihan**, cộng **Thiều Chửu**. Một sản phẩm thật đang phát hành xác nhận stack dữ liệu này khả thi.

_Source: [Hán Việt Từ Điển Trích Dẫn — cách sử dụng](http://hv-ebook-thuquan.blogspot.com/2015/03/han-viet-tu-ien-trich-dan-chi-dan-su.html) · [vietnamtudien.org/hanviet](http://vietnamtudien.org/hanviet/hv_logo.html) · [Giới thiệu HVTĐTD](https://giaotrinh.edu.vn/detail/han-viet-tu-dien-trich-dan-han-yue-ci-dian-zhai-yin) · [CC-CEDICT V1 Syntax](http://cc-cedict.org/wiki/format:syntax) · [kaikki.org](https://kaikki.org/) · [Wiktextract raw data](https://kaikki.org/dictionary/rawdata.html) · [Wiktextract LREC 2022 paper](http://www.lrec-conf.org/proceedings/lrec2022/pdf/2022.lrec-1.140.pdf)_
