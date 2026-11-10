---
title: 'Quản lý luật hai tầng Global và Tác phẩm'
type: 'feature'
ticket: '3'
created: '2026-11-09'
status: 'built'
baseline_revision: '99381b00144cdaedead2169e1e03cf4497a41907'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md
  - _bmad-output/initiative-auratranslate/ux-auratranslate/EXPERIENCE.md
  - _bmad-output/initiative-auratranslate/ux-auratranslate/DESIGN.md
  - _bmad-output/initiative-auratranslate/ux-auratranslate/mockups/quy-uoc-chu-viet.html
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Luật quy ước chỉ có một tầng và chỉ thêm được: `RuleTier` có đúng biến thể `Global`, `convention_rule` chưa có trong `project.db`, `core/convention/store.rs` không có hàm sửa, xoá hay đổi cờ bật, và `ConventionMatcherState` là một ô duy nhất không khoá theo Tác phẩm. Người dịch không tắt được một luật Global cho riêng một Tác phẩm (R2) và không sửa được một luật đã thêm sai.

**Approach:** Đưa cùng hằng `CONVENTION_RULE_DDL` sang `project.db`, phân giải hai tầng theo khoá luật (hàng tầng Tác phẩm thắng; hàng tầng Tác phẩm có cờ tắt là cách duy nhất tắt một luật Global), khoá bộ khớp theo Tác phẩm đang mở, thêm năm lệnh quản lý theo khuôn `commands/cleanup.rs`, và mở rộng lớp phủ Quản lý quy ước thành thanh tầng, sửa tại chỗ, xoá hai bước, tắt và bật lại. Mỗi lượt ghi danh sách dựng lại bộ khớp và quét lại Chương đang mở.

## Boundaries & Constraints

**Always:**
- AD-67 mục 2: một hằng DDL cho cả hai kho, `UNIQUE` trên khoá trong mỗi tầng, ghi đè theo khoá, **không cơ chế thứ hai** để tắt một luật.
- AD-67 mục 3: bộ khớp dựng từ danh sách **đã phân giải**, giữ theo Tác phẩm đang mở, dựng lại khi danh sách đổi. Phân giải xảy ra trước `Matcher::build`, nên `build` không bao giờ thấy hai luật cùng khoá.
- AD-67 mục 4: mỗi kết quả quét mang thế hệ; kết quả cũ hơn một lượt đổi danh sách luật bị bỏ.
- AD-49 mục 3: luật là nội dung người dùng lớp (iii) — xoá hỏi hai bước kèm chữ không hoàn tác được.
- R11: mọi thao tác làm được bằng bàn phím và bằng chuột.
- Lệnh quản lý luật giữ dạng đồng bộ (không `#[tauri::command(async)]`), vì chính điều đó giữ chúng không xen vào `convention_accept` cũng đồng bộ.

**Never:**
- Thẻ *Không phải lỗi* (R8, Story 16.5) và xuất nhập CSV/TSV (R9, Story 16.6) không thuộc story này.
- Không thêm cột hay bảng riêng để tắt luật; không đổi `Matcher::scan`, `casing.rs`, `splice.rs`.
- Không đổi tên trường của `ConventionSpanWire`, `ConventionSegmentWire`, `ConventionScanWire` (16.1 và 16.2 đọc chúng).
- Không chạm chữ ký ghi nhớ hay lượt ghi của Proofreader (AD-53).
- Không phơi cờ bật của một hàng tầng Tác phẩm ra giao diện: luật Tác phẩm bỏ bằng **Xoá**, không bằng tắt.

**Decision (Ice, 2026-11-09) — phép kiểm luật còn hiệu lực.** Làm theo hình thi hành được: phân giải cả hai tầng trong closure ghi, nửa tầng Tác phẩm nằm trong giao dịch `project.db`, nửa tầng Global là một lượt đọc `global.db` ngay trước lượt ghi. Không chạm spine trong story này. Mục nợ `Chủ: Story 16.3` đóng bằng `→ 🟡` nêu đúng khoảng còn hở: lượt đọc `global.db` không nằm trong giao dịch `project.db`, và chỉ tới được qua lệnh `(async)` duy nhất là `convention_replace_chapter`. Câu chữ của AD-67 mục 7 để Winston chữa về sau.

**Decision (Ice, 2026-11-10) — một hàng mỗi cụm mỗi tầng, ba thao tác cùng dùng nó.** Hàng tầng Tác phẩm có cờ tắt chiếm đúng cái chỗ `UNIQUE` của cụm ấy, nên thêm, tắt và bật lại đều tác động lên MỘT hàng: thêm một luật tầng Tác phẩm cho cụm đang bị tắt thì **nhận và bật lên** — dựng cờ, đặt cụm thay vừa gõ — chứ không từ chối vì trùng. Hàng có cờ tắt không bao giờ là một luật của người dùng, nên nó không bao giờ được liệt kê; điều đó cũng dọn luôn hàng bỏ lại sau khi một luật Toàn cục đổi cụm. Luật Toàn cục sống lại ở Tác phẩm đó sau lượt đổi cụm là ĐÚNG, vì AD-67 mục 2 lấy cụm làm danh tính.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Tắt luật Global ở một Tác phẩm | luật Global `đảm bảo → bảo đảm`; Tác phẩm A mở; bấm *Tắt ở Tác phẩm này* | hàng tầng Tác phẩm cùng khoá, `replacement_phrase` NULL, `enabled = 0`; quét A trả 0 chỗ; quét ở Tác phẩm B vẫn trả chỗ | chưa mở Tác phẩm ⇒ `err.convention.work_tier_unavailable`, không ghi gì |
| Bật lại | hàng khoá nói trên đang có | hàng khoá bị xoá; lượt quét kế trả lại chỗ | hàng đã biến mất ⇒ `err.convention.rule_missing`, nạp lại danh sách |
| Luật Tác phẩm che luật Global | Global `bằng hữu → bạn hữu`, Work `bằng hữu → bạn bè` | phân giải trả luật Work; hàng Global vẫn liệt kê, thụt vào, nhãn *Đang bị che* | không có |
| Sửa thành cụm đã có | sửa cụm cần tránh của một luật thành cụm của một luật khác **cùng tầng** | từ chối, không ghi gì | `err.convention.duplicate_avoid_phrase` |
| Sửa hoặc xoá luật đã biến mất | luật bị xoá ở nơi khác giữa hai lượt | từ chối, danh sách nạp lại | `err.convention.rule_missing` |
| Xoá hai bước | bấm *Xoá* lần một | hàng vào trạng thái chờ, không ghi gì; lần hai mới xoá | Esc hoặc rời hàng ⇒ huỷ trạng thái chờ |
| Luật đổi giữa lượt thay cả Chương | `convention_replace_chapter` (`async`) đang chạy, một luật bị xoá | cả lượt bị từ chối, 0 hàng đổi | `err.convention.rule_changed` |
| Danh sách đổi | bất kỳ lượt thêm, sửa, xoá, tắt, bật lại thành công | bộ khớp của Tác phẩm đang mở bị vô hiệu, thế hệ mới, quét lại Chương đang mở | kết quả thế hệ cũ bị bỏ |
| Tầng Tác phẩm khi chưa mở Tác phẩm | thanh tầng ở *Tác phẩm* hoặc *Cả hai* | mục *Tác phẩm* khoá, bảng nói rõ cần mở một Tác phẩm | không lệnh nào gửi đi |
| Thêm luật Tác phẩm cho cụm đang bị tắt | cụm X đang tắt ở Tác phẩm này; người dùng thêm luật Tác phẩm cho X | hàng có cờ tắt được nhận lại: cờ dựng lên, cụm thay là chữ vừa gõ; hàng Toàn cục chuyển từ *Đã tắt ở Tác phẩm này* sang *Đang bị che*; quét lại ra chỗ khớp với cụm thay mới | không lỗi; lỗi trùng cụm chỉ dành cho một luật đang BẬT cùng tầng |

</frozen-after-approval>

## Code Map

Rust (`src-tauri/src/`):
- `core/convention/rule.rs:3` `RuleTier` chỉ có `Global` (chú thích nói tầng Work tới cùng màn quản lý) — thêm `Work`; `Rule` :18 mang `key/avoid/replacement/tier`, không mang cờ bật.
- `core/convention/store.rs` — `RuleRow` :11, `into_rule` :20 **đóng cứng** `RuleTier::Global` ở :25, `list_rules(&Store)` :89, `find_active_rule` :98, `add_rule` :112; `ConventionStoreError` :31. Thiếu `edit_rule`, `delete_rule`, `set_enabled`, và mọi hàm đều không có tham số tầng.
- `core/convention/matcher.rs:30` `Matcher::build` sắp theo khoá rồi `dedup_by` trên khoá bằng nhau — giữ hàng **đầu sau khi sắp**, không biết tầng. Phân giải phải xong trước khi gọi nó.
- `core/cleanup/store.rs` — khuôn API hai tầng để chép: `resolve_two_tiers` :109, `list_tier` :180, `add_rule` :189, `edit_rule` :214, `set_enabled` :238, `delete_rule` :254 (xoá hàng không có là vô hại), `validate_pattern` :144; lỗi `RuleMissing`, `WorkTierUnavailable` :264. `core/cleanup/mod.rs:50` `CleanupRuleTier` với serde `"global"`/`"work"`. ⚠️ Hai tầng của cleanup **gộp** (`ScopeKind::ImportCleanupRule => Merge`, `core/scope/kinds.rs:214`) và không có đường tắt một luật Global ở tầng Work — phép phân giải ghi đè theo khoá là mã MỚI, chỉ hình dáng hàm là chép.
- `core/store/schema.rs:893` `CONVENTION_RULE_DDL` (chú thích :890 “One const so the Work tier can reuse it”), đã là bước 17 của `GLOBAL_MIGRATIONS` :800 (câu đầu mục :768). `PROJECT_MIGRATIONS` :2346 đang dừng ở bước 43, `len()` 42 (số 4 bị đốt), câu đầu mục :2176 — thêm bước 44 dùng lại đúng hằng đó, như `IMPORT_CLEANUP_RULE_DDL` :1305 dùng ở global bước 7 và project bước 19.
- `commands/convention.rs` — `ConventionMatcherState` :28 là **một ô không khoá**, `invalidate` :31, `matcher(&self, &Store)` :38 dựng từ `list_rules(global)` lọc cờ bật; `ConventionRuleWire` :58 đã có `enabled` và `tier`; `convention_list_rules` :67 (chỉ Global), `convention_active_rule` :83, `convention_add_rule` :93 (chỗ duy nhất gọi `invalidate`), `convention_scan_chapter` :284 và `_with_progress` :295 lấy bộ khớp từ `cache.matcher(global)`; `pub mod wire` :337.
- `commands/cleanup.rs:36` `store_for_tier(tier, global, open)` và năm hàm thuần :78/:110/:127/:142/:159 + vỏ `pub mod wire` :176 — khuôn để chép cho `convention_*`.
- `commands/segment/wire.rs:604` `convention_accept` là lệnh **đồng bộ**, gọi `convention_active_rule` ở :616 **trước** khoá `OpenWorkState`; `:647` `convention_replace_chapter` là `(async)` và gọi `resolve_convention_rules` ở :654 trước khoá. `commands/segment/convention_write.rs:311` `resolve_convention_rules`, `rule_changed()` :300, `plan_sentence` :338, `convention_replace_chapter` :444, `splice_convention_range` :120 — `RuleRow` được truyền vào, trong giao dịch không ai chạm `global.db`.
- `lib.rs:1176-1179` đăng ký bốn lệnh `convention::wire::`, `:1236-1238` ba lệnh `segment::wire::`, `:1742` `app.manage(ConventionMatcherState::default())`. `core/i18n/mod.rs:944-949` các khoá `err.convention.*`.

Webview (`src/`):
- `ConventionManageOverlay.vue` (282 dòng) — `<p class="cvm-tier">` là một dòng chữ tĩnh, **chưa có thanh tầng**; bảng `.cvm-table` chỉ hai cột, hàng `tr[data-convention-rule-row]` không con trỏ, không cột thao tác; `rows` :43 bỏ `enabled` và `tier`; `focusableWithin` :58 chỉ nhận `button`/`input`; `trapTab` :62, `onEscape` :54.
- `conventionManageState.ts` (100 dòng) — `load(mine)` :26, `openConventionManage` :37, `resetConventionManage` :56, `closeConventionManage` :67, `submitConventionRule` :75 (kết thúc bằng `flushEditorBeforeDiscreteWrite` rồi `scanConventionNow`) — đường quét lại duy nhất hiện có.
- `config/convention.ts` — `ConventionTier = 'global' | 'work'` đã có :7; `ConventionRuleWire` :35 đã có `enabled`/`tier`; `listConventionRules()` :174 không tham số, `addConventionRule` :187 không tham số tầng; `isRuleWire` :117, `failure` :146, `badShape` :156. Chưa có adapter sửa, xoá, đổi cờ.
- `conventionState.ts` — `generation` module-level, `scanConventionNow` :99, `clearConventionFor` :131, `resetConvention` :85.
- `GlossaryManageOverlay.vue` — khuôn cần theo: thanh tầng `.gm-seg.ui-seg` `role="radiogroup"` :439 (radio ẩn, `:disabled` khi `!manageWorkTierAvailable`, số đếm qua `manageTierCounts`), hàng listbox và `manageCursor` :576-700, sửa tại chỗ trong hàng con trỏ :652-700, `@dblclick` :636, nhãn xoá chờ :645; `glossaryManageState.ts` — `deletePendingKey` :98, `manageDeletePending` :262, `cancelGlossaryManageDeleteConfirm` :306, `deleteGlossaryManageEntry` :561 (nhịp một gài, nhịp hai xoá, rồi nạp lại và `refreshGlossaryMarks`).
- `ImportPreviewOverlay.vue` + `importPreviewState.ts` — tiền lệ bật tắt và sửa luật cleanup theo cặp `(tier, id)`: `onToggleCleanupRule` ~:237, `onDeleteCleanupRule` :271, `onStartEditCleanupRule` :285, state `editImportPreviewCleanupRule` :2202, `deleteImportPreviewCleanupRule` :2232, `cleanupRuleKey` `${tier}:${id}`.
- `commands/index.ts` — cổng :1223-1234, khai báo `convention.manage.open` :5423 (`Mod+Alt+A`), `.close` :5434, `.add` :5445; chưa lệnh nào của họ `convention.*` có khai báo `pointer`. `conventionHandlers.ts` (ở `src/`, không phải `src/commands/`) giữ bản đồ handler. `i18n/vi.json:963-973` chỉ có 11 khoá `convention.manage.*`.

Cổng và test:
- `scripts/check-commands.mjs` — `VUE_FLOOR = 62` :235, `POINTER_FLOOR = 273` :2968, `POINTER_DEBT = []` :2917, bảng điểm danh handler của lớp phủ này ở :2463-2464; `scripts/check-i18n.mjs:264` cũng giữ `VUE_FLOOR`.
- `src-tauri/tests/convention_contract.rs` (864 dòng) — `rule()` :39 đóng cứng `RuleTier::Global`; `fixture` :140, `seed` :166, `add_rule` :196, `scan` :200, `dump` :205, `versions` :232, `pairs_with` :239, `accept` :246, `replace` :267; ca `a_rule_disabled_after_the_scan_is_refused_and_writes_nothing` :501 lật cờ bằng SQL thẳng trên kho Global và phải tiếp tục xanh.
- `src-tauri/tests/convention_boundary.rs:11` `CONVENTION_FLOOR = 6` (đếm tệp trong `core/convention/`).
- `src-tauri/tests/config_invariants.rs` — `blocking_wire_cases` :1449 (ba mục quy ước :1499/:1504/:1509), hàng điểm danh `src/commands/convention.rs` :2247-2253 đang ghi 2 async / 2 plain kèm chú thích.
- `src-tauri/tests/ipc_argument_contract.rs:630` `NO_FRONTEND_CALLER` không có mục quy ước — mọi lệnh mới phải có chỗ gọi thật ở `src/`.
- `src-tauri/tests/pinned_contract.rs` — ghim global 17 ở :86-100 và project `len() == 42` quanh :260. Các tệp khác ghim phiên bản project 43: `prompt_set_contract.rs:1006`, `tm_contract.rs:823/:1234/:1269`, `review_harvest_contract.rs:701`, `prompt_set_kind_migration_contract.rs:152`.
- `src-tauri/tests/cleanup_contract.rs` — khuôn fixture hai tầng: `open_global` :57, `open_work_real` :64, ca `a_global_rule_number_one_and_a_work_rule_number_one_coexist_and_toggle_independently` :361, `choosing_the_work_tier_with_no_work_open_is_refused` :967.
- Vitest ở `tests/frontend/`: `conventionManage.test.ts` (4 ca) phải mở rộng; `libraryImportResetsConventionManage.test.ts` gọi `resetConventionManage()`.
- E2E: `e2e/specs/story-16-1-convention-mark-accept.e2e.mjs:56-64` thêm luật **qua giao diện** (`[data-tools-item="convention.manage.open"]`, `[data-convention-add-avoid]`, `[data-convention-add]`, `[data-convention-rule-row]`) — đổi lớp phủ là đổi các bộ chọn này.

## Tasks & Acceptance

**Execution:**
- [ ] `core/store/schema.rs` — `PROJECT_MIGRATIONS` bước 44 dùng lại đúng hằng `CONVENTION_RULE_DDL`; sửa câu đầu mục của cả hai danh sách (câu ở :768 và :2176).
- [ ] `core/convention/rule.rs`, `store.rs` — thêm `RuleTier::Work` (serde `"work"`); mọi hàm kho nhận tầng và `into_rule` lấy tầng từ tham số thay vì đóng cứng; thêm `edit_rule`, `delete_rule`, `set_enabled`, `find_row_by_key`; lỗi `RuleMissing`, `WorkTierUnavailable` theo khuôn `cleanup`.
- [ ] `core/convention/resolve.rs` (mới) — hàm thuần `resolve_two_tiers(global_rows, work_rows) -> Vec<Rule>`: gom theo khoá, hàng tầng Tác phẩm thắng bất kể cờ bật, rồi lọc cờ bật; kèm hàm trả trạng thái hiển thị từng hàng cho lớp phủ (*đang dùng* · *đang bị che* · *đã tắt ở Tác phẩm này*) để Rust là nơi duy nhất biết luật ghi đè.
- [ ] `commands/convention.rs` — `ConventionMatcherState` khoá theo Tác phẩm đang mở (đường dẫn hay id của `OpenWork`), `invalidate` nhận khoá đó; `matcher` dựng từ `resolve_two_tiers`; `convention_list_rules(tier)` trả cả hai tầng kèm trạng thái; năm lệnh thuần `convention_add_rule`/`edit_rule`/`delete_rule`/`set_enabled`/`list_rules` + vỏ `wire` **đồng bộ** theo khuôn `commands/cleanup.rs:36-176`; mỗi lượt ghi thành công vô hiệu bộ khớp.
- [ ] `commands/segment/wire.rs`, `convention_write.rs` — phân giải luật hai tầng chuyển vào trong khoá `OpenWork`; ở `convention_replace_chapter` phần phân giải nằm **trong closure ghi** để một luật đổi giữa lượt làm cả lượt bị từ chối.
- [ ] `lib.rs`, `core/i18n/mod.rs`, `tests/config_invariants.rs` — đăng ký lệnh mới, khoá `err.convention.rule_missing`/`work_tier_unavailable`, cập nhật hàng điểm danh `convention.rs` và chú thích của nó.
- [ ] `src/config/convention.ts` — adapter cho năm lệnh (không bao giờ ném, kiểm kiểu lúc chạy), `listConventionRules(tier)` trả thêm trạng thái hàng; `addConventionRule` nhận tầng.
- [ ] `src/conventionManageState.ts` — thanh tầng với số đếm, ô tìm (lọc phía webview), con trỏ hàng, sửa tại chỗ, xoá hai bước theo khuôn `glossaryManageState.ts:561`, tắt và bật lại; mỗi lượt ghi thành công nạp lại danh sách rồi `flushEditorBeforeDiscreteWrite` và `scanConventionNow` khi có Chương mở; mục *Tác phẩm* khoá khi chưa mở Tác phẩm.
- [ ] `src/ConventionManageOverlay.vue` — thanh tầng `.ui-seg` `role="radiogroup"`, bảng năm cột (*Cụm cần tránh → Cụm thay thế · Tầng · Trạng thái · thao tác*), vạch trái 2px `confirmed` cho hàng tầng Tác phẩm, hàng Global bị che thụt `calc(var(--space-unit) * 5)` kèm nhãn viền đứt, nút xoá bậc Nguy hiểm hai bước, hàng thêm luật có chọn tầng (mặc định Tác phẩm khi có Tác phẩm mở); `focusableWithin` nhận thêm `select`/`[tabindex]`; Esc huỷ trạng thái chờ trước, rồi mới đóng.
- [ ] `src/commands/index.ts`, `src/conventionHandlers.ts`, `src/i18n/vi.json` — lệnh mới cho sửa, xoá, tắt, bật lại, đổi tầng (kèm khai báo `pointer`, `POINTER_DEBT` vẫn rỗng), nhãn `command.convention.manage.*`, các khoá `convention.manage.*` của EXPERIENCE dùng trong story này.
- [ ] `scripts/check-commands.mjs` — hàng điểm danh handler mới của lớp phủ ở :2463; nâng `VUE_FLOOR`/`POINTER_FLOOR` chỉ khi cổng đòi.
- [ ] `src-tauri/tests/convention_contract.rs` — `rule()` nhận tầng; ca cho từng hàng của bảng I/O, trong đó AD-67 12 (g) đối chứng bằng phép **gỡ** nhánh ưu tiên tầng Tác phẩm trong `resolve_two_tiers`; ca phân giải trong closure ghi đối chứng bằng phép gỡ chính nó; ca `:501` giữ nguyên; nâng `CONVENTION_FLOOR` ở `convention_boundary.rs:11` vì `resolve.rs` là tệp thứ bảy của `core/convention/`.
- [ ] `src-tauri/tests/pinned_contract.rs` và sáu tệp ghim phiên bản project 43 — nâng lên 44 (`len()` 42 → 43).
- [ ] `tests/frontend/conventionManage.test.ts` — đổi tầng giữ bộ lọc, xoá hai bước, bật lại xoá hàng khoá, mục *Tác phẩm* khoá khi chưa mở Tác phẩm, mỗi lượt ghi gọi đúng một lượt quét lại.
- [ ] `e2e/specs/story-16-1-convention-mark-accept.e2e.mjs` — cập nhật bộ chọn theo lớp phủ mới.
- [ ] `e2e/specs/story-16-3-convention-two-tier.e2e.mjs` — `verify` của vé bằng chuột thật và bằng bàn phím: tắt một luật Global ở Tác phẩm A thì dấu biến, một luật Tác phẩm cùng cụm thay luật Global.
- [ ] `deferred-work.md` — đóng mục `Chủ: Story 16.3` bằng một dòng `→ 🟡` theo Decision trong khối đã chốt.

**Acceptance Criteria:**
- Given một luật Global và hai Tác phẩm, when người dùng tắt luật đó ở Tác phẩm A, then quét A trả 0 chỗ khớp, quét B vẫn trả chỗ, và luật Global vẫn còn trong danh sách tầng Global (R2, AD-67 12 (g)).
- Given phép thử trên, when nhánh ưu tiên tầng Tác phẩm bị gỡ thật khỏi `resolve_two_tiers`, then ca đó đỏ vì đúng lý do đang đo.
- Given lớp phủ Quản lý quy ước, when làm trọn thêm, sửa, xoá, tắt, bật lại **chỉ bằng bàn phím**, rồi lặp lại **chỉ bằng chuột**, then cả hai lượt hoàn tất và tiêu điểm trở về nút mở sau Esc (R11).
- Given một Chương đang mở có dấu, when danh sách luật đổi bằng bất kỳ thao tác nào, then bộ khớp được dựng lại và dấu trong Chương khớp danh sách mới, không còn dấu của luật vừa tắt hay vừa xoá.
- Given `convention_replace_chapter` đang chạy, when một luật trong lượt đó bị xoá trước lúc ghi, then cả lượt bị từ chối với `err.convention.rule_changed` và không hàng `segment` nào đổi.
- Given một cơ sở dữ liệu Tác phẩm cũ (phiên bản 43), when mở nó, then nó di trú lên 44 và bảng `convention_rule` có mặt với đúng các ràng buộc của hằng dùng chung.

## Implementation Notes

- `convention_set_enabled(rule_key, enabled)` thay cặp `(tier, id)`: tắt = chèn hàng tầng Tác phẩm cùng khoá (`enabled = 0`, cụm thay NULL), bật lại = xoá hàng đó; chỉ nhận luật Global và cần Tác phẩm đang mở. Webview ẩn hàng tắt này khỏi tầng Tác phẩm, nên cờ bật của hàng Tác phẩm không lộ ra giao diện.
- `resolve_two_tiers` và `effective_row` (`core/convention/resolve.rs`) dùng chung một luật ưu tiên; `convention_replace_chapter` nhận danh sách Global đọc ngay trước lượt ghi, nửa tầng Tác phẩm đọc trong giao dịch. Bản `resolve_convention_rules` đọc trước khoá đã bị gỡ.
- `convention_accept` nay lấy khoá `OpenWorkState` rồi mới phân giải luật; `convention_active_rule` nhận thêm `Option<&OpenWork>`.
- `ConventionMatcherState` giữ một ô `(đường dẫn Tác phẩm, Matcher)`; mọi lượt ghi luật xoá ô, đổi Tác phẩm tự trượt khoá.
- `convention_list_rules(tier)` luôn tính trạng thái hàng Global theo hàng Tác phẩm đang mở; webview gọi hai lần (`global`, `work`) và coi `err.convention.work_tier_unavailable` là "chưa mở Tác phẩm".
- Bảng ghim di trú: `PROJECT_MIGRATIONS` 43 bước, đích 44; mọi ca hạ phiên bản bằng SQL phải `DROP TABLE convention_rule` và xoá dòng log 44.
- Sàn `CLICK_FLOOR`/`COMMAND_FLOOR`/`HANDLER_ATTR_FLOOR` của `check-commands.mjs` được nâng vì cổng đòi sau khi thêm chín lệnh và các nút của lớp phủ; `POINTER_DEBT` vẫn rỗng.
- Đối chứng: gỡ nhánh Tác phẩm thắng khỏi `resolve_two_tiers` làm bốn ca đỏ đúng lý do (khớp ở A, ưu tiên hàng Tác phẩm, trạng thái che).
- Đối chứng thứ hai: gỡ `list_rules_in(tx)` khỏi `resolve_convention_rules` (phân giải chỉ còn danh sách Global chụp trước lượt ghi) làm đúng `a_rule_switched_off_in_the_work_during_a_chapter_replace_refuses_the_whole_batch` đỏ, và đỏ vì mẻ ghi ĐƯỢC ÁP (`replaced: 1`, ô thành `Bảo đảm an toàn.`) thay vì bị từ chối — đúng khe đang đo.

## Plan Change Log

- Pass 1, hai phát hiện cùng gốc (hàng khoá tầng Tác phẩm vô hình với giao diện nhưng thật với `UNIQUE` của kho và với danh sách). Khoảng trống thuộc intent: plan dựng cơ chế hàng khoá nhưng không nói lượt THÊM và lượt ĐỔI CỤM gặp nó thì sao. Ice chọn ngày 2026-11-10: thêm thì nhận và bật lên, và hàng có cờ tắt không bao giờ được liệt kê. Đã thêm một Decision và một hàng bảng I/O; mã được vá tại chỗ theo lựa chọn của Ice thay vì hoàn nguyên rồi dựng lại, nên `review_loop_iteration` giữ 0. Trạng thái xấu tránh được: người dùng xin một luật Tác phẩm cho cụm mình vừa tắt và nhận lỗi "trùng cụm" trỏ vào một hàng bị ẩn. GIỮ khi dựng lại: phép phân giải hai tầng ở `core/convention/resolve.rs`, phép đọc hàng tầng Tác phẩm TRONG giao dịch ghi của `convention_replace_chapter` (đã đối chứng bằng phép gỡ thật), và `conventionManageSummary` đếm cả hai tầng chưa lọc (cũng đã đối chứng).

## Review Triage Log

Pass 1 (quick): 0 high · 5 medium · 2 low · 0 false · 0 maybe-false. Hai mục nhóm thành một gốc, định tuyến `intent_gap`; các mục còn lại chờ, vì một lượt lặp lại sẽ dựng lại mã.

- Nút **Đóng** không đóng khi đang có lượt xoá chờ hay một ô sửa đang mở — medium · patch. Xác minh: `ConventionManageOverlay.vue:208` phát đúng `convention.manage.close`, và `closeConventionManage` đóng theo bậc (huỷ chờ → huỷ sửa → đóng), nên cần tới ba cú bấm. Hành vi theo bậc chỉ dành cho Esc.
- AC3 (R11) chưa được spec e2e phủ — medium · patch. Xác minh: `story-16-3-convention-two-tier.e2e.mjs` chạy thêm luật bằng chuột, tắt và bật lại bằng cả chuột lẫn Space, và một lượt xoá được gài rồi huỷ bằng Esc; không lượt nào **hoàn tất** một lượt sửa hay một lượt xoá, không lượt nào thêm luật chỉ bằng bàn phím, và không chỗ nào khẳng định tiêu điểm trở về `[data-convention-manage-open]`.
- Hàng Global bị che mất cả bốn nút, nhưng bàn phím vẫn sửa và xoá được nó — medium · patch. Xác minh: `ConventionManageOverlay.vue:412` `v-else-if="rule.status !== 'shadowed'"` ẩn Sửa, Tắt, Bật lại, Xoá; `onKeydown` vẫn phát `edit` và `delete`; `beginConventionManageEdit` và `deleteConventionManageRule` không đọc `status`. Mockup `quy-uoc-chu-viet.html` dòng 311 vẽ rõ `[Sửa] [Xoá]` trên đúng hàng `tr.sh`, nên chỗ sai là template, không phải bàn phím — và hai đường vào lệch nhau là lỗi R11.
- Bộ khớp nay được dựng trong lúc đang giữ khoá `OpenWorkState` — low · patch chỉ phần chú thích. Xác minh: `cache.matcher(global, open)` nằm trong khối `guard` của `convention_scan_chapter_with_progress`, nên chú thích "the Work lock is held only to copy its text" ở `commands/convention.rs` hết đúng và phải sửa. Phần giữ khoá thì bác: AD-67 mục 4 đòi nhả khoá trước khi **khớp**, và lượt khớp vẫn ở ngoài khoá; lượt dựng đo được 4,6 ms p95 ở 2.000 luật (16.1) và chỉ chạy khi ô đệm trống, nên không ai gặp nó trong lúc dùng, còn phép chữa thì phải chép hàng luật ra rồi dựng ngoài khoá — hơn một phép sửa trực tiếp.
- Sửa cụm cần tránh của một luật Global bỏ lại hàng khoá của từng Tác phẩm — medium · intent_gap (gốc chung với mục dưới). Xác minh: `edit_rule` ghi lại `rule_key`; hàng khoá ở `project.db` giữ khoá CŨ, nên `switchedOff` trong `conventionManageState.ts` không còn khớp nó và nó hiện ra như một luật Tác phẩm `inactive` người dùng chưa từng thêm. Lượt luật sống lại thì KHÔNG phải lỗi: AD-67 mục 2 lấy khoá làm danh tính, nên đổi cụm là một luật khác, và một luật mới thì đang bật. Hàng bỏ lại mới là lỗi.
- Thêm một luật Tác phẩm cho cụm đang bị tắt bị từ chối vì trùng, trong khi hàng gây trùng bị ẩn — medium · intent_gap (cùng gốc). Xác minh: `disable_in_work` chèn hàng mang đúng khoá đó, `add_rule` gặp `UNIQUE` và trả `convention.duplicate_avoid_phrase`, còn danh sách tầng Tác phẩm đã lọc hàng ấy đi. Gốc chung của hai mục: AD-67 mục 2 chỉ cho MỘT hàng mỗi khoá mỗi tầng, nhưng ba thao tác thêm, tắt, bật lại không coi nó là cùng một hàng.
- AC6 chưa có ca nào khẳng định ràng buộc phía `project.db` — low · patch. Xác minh: nửa di trú ĐÃ có người canh — `prompt_set_kind_migration_contract.rs:162` hạ kho Tác phẩm về 42, mở lại và khẳng định `schema_version() == 44`, nên bước 44 thật sự chạy trên một kho chưa có bảng. Chưa ca nào thêm hai luật Tác phẩm cùng khoá để chạm `UNIQUE(rule_key)` ở `project.db`; một ca mười dòng là đủ.

## Design Notes

- **Phân giải hai tầng.** `BTreeMap<rule_key, (tier, RuleRow)>`: nạp hàng Global trước, hàng Work ghi đè, rồi lọc `enabled`. Hàng Work thắng **bất kể** cờ bật của nó — đó chính là cách tắt một luật Global, nên không có cơ chế thứ hai (AD-67 mục 2). Phân giải phải chạy trước `Matcher::build`, vì `dedup_by` ở `matcher.rs:30` chỉ giữ hàng đầu sau khi sắp theo khoá và không biết tầng.
- **Trạng thái hàng là dẫn xuất, không phải cột mới.** Với mỗi hàng Global: có hàng Work cùng khoá và `enabled = 1` ⇒ *Đang bị che*; có hàng Work cùng khoá và `enabled = 0` ⇒ *Đã tắt ở Tác phẩm này*; không có ⇒ *Đang dùng*. Vì giao diện không bao giờ ghi cờ bật lên một hàng Global, một hàng Global `enabled = 0` chỉ tới được bằng SQL thẳng (ca test `:501`): bộ phân giải vẫn loại nó, và nó hiện ở nhóm *Đang dùng*. Một hàng Work `enabled = 0` mà **không** có hàng Global cùng khoá là luật Tác phẩm người dùng đã tắt — giao diện story này không sinh ra được trạng thái đó, nên nó chỉ là một hàng không hiệu lực trong danh sách tầng Tác phẩm.
- **Phép kiểm luật còn hiệu lực.** `convention_accept` là lệnh đồng bộ và các lệnh quản lý luật cũng đồng bộ, nên chúng chạy trên cùng main thread và **không xen được vào nhau**: cửa sổ đua ở đường nhận một chỗ là rỗng, và điều đó chỉ đúng khi lệnh quản lý không mang `(async)`. Chỉ `convention_replace_chapter` là `(async)`, nên chỉ đường đó cần phân giải trong closure ghi. Sau story này nửa tầng Tác phẩm của phép kiểm nằm **trong** giao dịch `project.db`; nửa tầng Global thì không, vì hai kho không có giao dịch chung; khoảng còn hở đó là mục nợ 🟡 của Decision trong khối đã chốt.
- **Phases** (AGENTS.md): Rust · Webview · Tests that move (e2e + cập nhật ghim di trú), một agent mới mỗi phase, giao nhau qua tệp handoff cạnh plan này.
- Ô tìm của EXPERIENCE được làm trong story này (lọc phía webview trên danh sách đã nạp): với 2.000 luật, một bảng không có ô tìm là không dùng được, và nó không thêm lệnh Rust nào.

## Verification

**Commands:**
- `npm run build` rồi `cargo test --manifest-path src-tauri/Cargo.toml --test convention_contract --test convention_boundary --test pinned_contract` — expected: xanh.
- `cargo test` đầy đủ + vitest một lượt (đổi `PROJECT_MIGRATIONS`, đăng ký lệnh ở `lib.rs`, và bảy tệp ghim phiên bản) — expected: xanh.
- `npm run check:commands` và `npm run check:i18n` — expected: xanh, `POINTER_DEBT` rỗng.
- `npm run check:debt-owner` — expected: xanh sau khi đóng mục nợ `Chủ: Story 16.3`.
- `npm run test:e2e -- --spec e2e/specs/story-16-3-convention-two-tier.e2e.mjs` và `--spec e2e/specs/story-16-1-convention-mark-accept.e2e.mjs` (ngoài sandbox, máy không khoá) — expected: xanh trên WKWebView.
