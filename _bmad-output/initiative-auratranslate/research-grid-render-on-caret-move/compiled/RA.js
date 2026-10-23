import { createCommentVNode as _createCommentVNode, toDisplayString as _toDisplayString, openBlock as _openBlock, createElementBlock as _createElementBlock, withModifiers as _withModifiers, withKeys as _withKeys, normalizeClass as _normalizeClass, createElementVNode as _createElementVNode, renderList as _renderList, Fragment as _Fragment, isMemoSame as _isMemoSame, withMemo as _withMemo, createTextVNode as _createTextVNode, resolveComponent as _resolveComponent, createVNode as _createVNode, createBlock as _createBlock, normalizeStyle as _normalizeStyle, withCtx as _withCtx } from "vue"

const _hoisted_1 = {
  key: 0,
  class: "load-error"
}
const _hoisted_2 = {
  key: 1,
  class: "load-error"
}
const _hoisted_3 = {
  key: 2,
  class: "load-error"
}
const _hoisted_4 = {
  key: 3,
  class: "tabs",
  role: "tablist"
}
const _hoisted_5 = ["aria-selected", "tabindex"]
const _hoisted_6 = ["aria-selected", "tabindex"]
const _hoisted_7 = ["data-chapter-id"]
const _hoisted_8 = {
  class: "col col-rule",
  "aria-hidden": "true"
}
const _hoisted_9 = {
  key: 0,
  class: "strip-pad",
  "aria-hidden": "true"
}
const _hoisted_10 = {
  class: "col col-num",
  "aria-hidden": "true"
}
const _hoisted_11 = {
  key: 0,
  class: "strip-pad",
  "aria-hidden": "true"
}
const _hoisted_12 = ["data-segment-id", "data-cut-count"]
const _hoisted_13 = ["data-role"]
const _hoisted_14 = {
  key: 0,
  class: "cut-mark",
  "aria-hidden": "true"
}
const _hoisted_15 = ["data-src-start", "onMouseenter"]
const _hoisted_16 = {
  key: 0,
  class: "strip-pad",
  "aria-hidden": "true"
}
const _hoisted_17 = ["data-segment-id", "data-unlinked", "aria-label", "data-row-error", "aria-details"]
const _hoisted_18 = {
  key: 0,
  class: "strip-pad",
  "aria-hidden": "true"
}

export function render(_ctx, _cache) {
  const _component_ChapterImage = _resolveComponent("ChapterImage")
  const _component_SourceHanViet = _resolveComponent("SourceHanViet")
  const _component_TmFuzzyStrip = _resolveComponent("TmFuzzyStrip")
  const _component_GlossaryConfirmStrip = _resolveComponent("GlossaryConfirmStrip")
  const _component_ProofreaderStrip = _resolveComponent("ProofreaderStrip")
  const _component_PanelFrame = _resolveComponent("PanelFrame")

  return (_openBlock(), _createBlock(_component_PanelFrame, {
    owner: "panel.grid",
    "status-key": "panel.grid.status",
    "show-status": _ctx.showFrameStatus
  }, {
    default: _withCtx(() => [
      _createCommentVNode(" Lỗi nạp nói ra bằng chuỗi CỦA NÓ, không im — cùng khuôn `SourcePanel.vue`. "),
      (_ctx.loadErrorKey !== null)
        ? (_openBlock(), _createElementBlock("p", _hoisted_1, _toDisplayString(_ctx.t(_ctx.loadErrorKey)), 1 /* TEXT */))
        : (_ctx.showEmptyChapter)
          ? (_openBlock(), _createElementBlock("p", _hoisted_2, _toDisplayString(_ctx.t('panel.grid.empty_chapter')), 1 /* TEXT */))
          : (_ctx.showNoSegments)
            ? (_openBlock(), _createElementBlock("p", _hoisted_3, _toDisplayString(_ctx.t('panel.grid.no_segments')), 1 /* TEXT */))
            : _createCommentVNode("v-if", true),
      _createCommentVNode("\n      🔴 DẢI TAB HÁN VIỆT — AC8 vế *\"người dùng tự bật tắt\"*, và nó phải có mặt bằng CHUỘT.\n\n      Ba command đã tồn tại từ Story 1.16 và có phím tắt, nên vế bàn phím của AC8 vốn đã chạy.\n      Nhưng dải tab này sống ở `SourcePanel.vue` — tệp bị gỡ ở story này — nên bỏ nó đi là để\n      lại một tính năng **chỉ tới được bằng phím tắt người dùng chưa chắc biết. Đó là một lượt\n      rút tính năng im lặng, không một lượt gọn gàng.\n\n      ⚠️ Mỗi `@click` là ĐÚNG MỘT lời gọi `dispatch('<id>')` — `check-commands.mjs` Kiểm A\n      cưỡng chế AD-34 §1. Không hàm khác, không mã nội tuyến.\n      🔴 `tabindex` roving: đúng MỘT tab vào được vòng Tab, mũi tên đi giữa hai tab — một hợp\n      đồng `tablist` khai một nửa còn tệ hơn không khai, vì nó hứa một mô hình tương tác không\n      tồn tại (bắt ở code review 2026-08-06).\n    "),
      (_ctx.hasSegments && _ctx.isChinese)
        ? (_openBlock(), _createElementBlock("div", _hoisted_4, [
            _createElementVNode("button", {
              id: "grid-tab-original",
              ref: "tabOriginalBtn",
              type: "button",
              class: _normalizeClass(["tab", { active: _ctx.activeTab === 'original' }]),
              role: "tab",
              "aria-controls": "grid-tabpanel",
              "aria-selected": _ctx.activeTab === 'original',
              tabindex: _ctx.activeTab === 'original' ? 0 : -1,
              onClick: _cache[0] || (_cache[0] = $event => (_ctx.dispatch('source.select_tab_original'))),
              onKeydown: [
                _cache[1] || (_cache[1] = _withKeys(_withModifiers($event => (_ctx.selectTabViaArrow('han_viet')), ["prevent"]), ["right"])),
                _cache[2] || (_cache[2] = _withKeys(_withModifiers($event => (_ctx.selectTabViaArrow('han_viet')), ["prevent"]), ["left"]))
              ]
            }, _toDisplayString(_ctx.t('panel.source.tab_original')), 43 /* TEXT, CLASS, PROPS, NEED_HYDRATION */, _hoisted_5),
            _createElementVNode("button", {
              id: "grid-tab-han-viet",
              ref: "tabHanVietBtn",
              type: "button",
              class: _normalizeClass(["tab", { active: _ctx.activeTab === 'han_viet' }]),
              role: "tab",
              "aria-controls": "grid-tabpanel",
              "aria-selected": _ctx.activeTab === 'han_viet',
              tabindex: _ctx.activeTab === 'han_viet' ? 0 : -1,
              onClick: _cache[3] || (_cache[3] = $event => (_ctx.dispatch('source.select_tab_han_viet'))),
              onKeydown: [
                _cache[4] || (_cache[4] = _withKeys(_withModifiers($event => (_ctx.selectTabViaArrow('original')), ["prevent"]), ["right"])),
                _cache[5] || (_cache[5] = _withKeys(_withModifiers($event => (_ctx.selectTabViaArrow('original')), ["prevent"]), ["left"]))
              ]
            }, _toDisplayString(_ctx.t('panel.source.tab_han_viet')), 43 /* TEXT, CLASS, PROPS, NEED_HYDRATION */, _hoisted_6),
            _createCommentVNode(" aura-allow-text: cả hai nhánh đi qua t(), không chuỗi viết thẳng nào — Kiểm A2\n           không đọc tĩnh được toán tử ba ngôi (cùng khuôn `LibraryMode.vue:143`). "),
            (_ctx.activeTab === 'han_viet' && (_ctx.viewMode === 'parallel' || _ctx.canUseParallelView))
              ? (_openBlock(), _createElementBlock("button", {
                  key: 0,
                  type: "button",
                  class: "view-toggle",
                  onClick: _cache[6] || (_cache[6] = $event => (_ctx.dispatch('source.toggle_han_viet_view')))
                }, _toDisplayString(_ctx.viewMode === 'switch' ? _ctx.t('panel.source.view_mode_parallel') : _ctx.t('panel.source.view_mode_switch')), 1 /* TEXT */))
              : _createCommentVNode("v-if", true)
          ]))
        : _createCommentVNode("v-if", true),
      (_ctx.hasSegments)
        ? (_openBlock(), _createElementBlock("div", {
            key: 4,
            id: "grid-tabpanel",
            class: "grid-scroll",
            "data-chapter-id": _ctx.chapterId
          }, [
            _createCommentVNode("\n        🔴 LƯỚI CHỦ-CỘT — Quyết định #1(b). Cha khai **tập track hàng**; năm con là năm CỘT.\n        `grid-template-rows` đặt qua `:style` vì số hàng là dữ liệu, không phải hằng CSS.\n\n        ⚠️ Đây là chỗ DUY NHẤT của tệp này bind một giá trị hình học qua `:style`. Màu thì\n        **không bao giờ** — Kiểm B của `check-tokens.mjs` đọc CSS, không đọc TypeScript, nên\n        một màu bind qua `:style` đi qua cổng mà không một dòng nào được soi.\n      "),
            _createElementVNode("div", {
              class: "grid",
              style: _normalizeStyle({ gridTemplateRows: `repeat(${_ctx.editorSegments.length + (_ctx.stripAnchorIndex < 0 ? 0 : 1)}, auto)` })
            }, [
              _createCommentVNode("\n          Cột ① — VẠCH TRẠNG THÁI (UX-DR19, sáu giá trị).\n\n          `aria-hidden`: vạch là kênh **thị giác** của trạng thái; trình đọc màn hình nghe\n          trạng thái qua tên truy cập (`aria-label`) của ô bản dịch.\n        "),
              _createElementVNode("div", _hoisted_8, [
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_ctx.editorSegments, (s, __, ___, _cached) => {
                  const _memo = ([s, _ctx.ruleClassForRow(s.id), _ctx.selectedRowClassById.get(s.id), s.id === _ctx.stripAnchorSegmentId])
                  if (_cached && _cached.el && _cached.key === s.id && _isMemoSame(_cached, _memo)) return _cached
                  const _item = (_openBlock(), _createElementBlock(_Fragment, {
                    key: s.id
                  }, [
                    _createElementVNode("div", {
                      class: _normalizeClass(["cell cell-rule", [{ 'para-end': s.is_paragraph_end, 'cell-unlinked': _ctx.unlinkedIds.has(s.id) }, _ctx.selectedRowClassById.get(s.id)]])
                    }, [
                      _createElementVNode("div", {
                        class: _normalizeClass(["margin-slot", _ctx.ruleSlotClass(_ctx.ruleClassForRow(s.id))])
                      }, null, 2 /* CLASS */)
                    ], 2 /* CLASS */),
                    (s.id === _ctx.stripAnchorSegmentId)
                      ? (_openBlock(), _createElementBlock("div", _hoisted_9))
                      : _createCommentVNode("v-if", true)
                  ], 64 /* STABLE_FRAGMENT */))
                  _item.memo = _memo
                  return _item
                }, _cache, 7), 128 /* KEYED_FRAGMENT */))
              ]),
              _createCommentVNode("\n          Cột ② — SỐ CÂU. `on-surface-variant`, KHÔNG `ornament` (Quyết định #9(a)): đây là\n          **chữ thật**, không phải nét, và `ornament` đo được 2,44 (sáng) / 2,64 (tối) trên\n          `surface` — trượt AA.\n          aura-allow-text: số thứ tự hàng là DỮ LIỆU, không một chuỗi giao diện.\n        "),
              _createElementVNode("div", _hoisted_10, [
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_ctx.editorSegments, (s, i, ___, _cached) => {
                  const _memo = ([s, i, _ctx.selectedRowClassById.get(s.id), s.id === _ctx.stripAnchorSegmentId])
                  if (_cached && _cached.el && _cached.key === s.id && _isMemoSame(_cached, _memo)) return _cached
                  const _item = (_openBlock(), _createElementBlock(_Fragment, {
                    key: s.id
                  }, [
                    _createElementVNode("div", {
                      class: _normalizeClass(["cell cell-num", [{ 'para-end': s.is_paragraph_end, omitted: s.is_omitted }, _ctx.selectedRowClassById.get(s.id)]])
                    }, [
                      _createCommentVNode(" aura-allow-text: số thứ tự hàng — một con số dựng tại chỗ hiển thị, đúng luật \"định dạng số và ngày giờ CHỈ ở frontend\" (§Consistency Conventions), không một chuỗi giao diện dịch được. "),
                      _createTextVNode(_toDisplayString(i + 1), 1 /* TEXT */)
                    ], 2 /* CLASS */),
                    (s.id === _ctx.stripAnchorSegmentId)
                      ? (_openBlock(), _createElementBlock("div", _hoisted_11))
                      : _createCommentVNode("v-if", true)
                  ], 64 /* STABLE_FRAGMENT */))
                  _item.memo = _memo
                  return _item
                }, _cache, 9), 128 /* KEYED_FRAGMENT */))
              ]),
              _createCommentVNode(" Cột ③ — NGUYÊN VĂN. Một bề mặt vùng chọn duy nhất, vai `'source'` (AC7). "),
              _createElementVNode("div", {
                ref: "colSrc",
                class: _normalizeClass(["col col-src", _ctx.sourceTokenClass])
              }, [
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_ctx.editorSegments, (s, __, ___, _cached) => {
                  const _memo = ([
              s,
              _ctx.selectedRowClassById.get(s.id),
              s.id === _ctx.stripAnchorSegmentId,
              _ctx.cutOffsetsOf(s.id),
              _ctx.glossaryTermsOf(s.id),
              _ctx.concordanceSourceHighlight?.segmentId === s.id ? _ctx.concordanceSourceHighlight : null,
              _ctx.trailingImagesOf(s.id),
              s.id === _ctx.firstSegmentId ? _ctx.editorChapterAssets : null,
              _ctx.editorAssetsDir,
              _ctx.showHanViet,
              _ctx.viewMode,
            ])
                  if (_cached && _cached.el && _cached.key === s.id && _isMemoSame(_cached, _memo)) return _cached
                  const _item = (_openBlock(), _createElementBlock(_Fragment, {
                    key: s.id
                  }, [
                    _createElementVNode("div", {
                      class: _normalizeClass(["cell cell-src", [
              {
                'para-end': s.is_paragraph_end,
                omitted: s.is_omitted,
              },
              _ctx.selectedRowClassById.get(s.id),
            ]]),
                      "data-segment-id": s.id,
                      "data-cut-count": _ctx.cutCountOf(s.id),
                      "data-col": "src",
                      onMouseup: _cache[12] || (_cache[12] = (...args) => (_ctx.onSourceCellMouseUp && _ctx.onSourceCellMouseUp(...args)))
                    }, [
                      _createCommentVNode("\n              🔵 Story 6.14 (nợ ③) — NHÃN VAI, trước mọi nội dung khác của ô. `aria-hidden`\n              KHÔNG đặt: đây là chữ THẬT phân biệt hàng này với văn xuôi, không một hoạ tiết.\n              `s.role` không có `data-src-start` ⇒ vô hình với `sourceCutOffsetOf` (nó chỉ đọc\n              neo, không đếm mù toàn bộ ô — xem doc-comment `editorSegments.ts:282`).\n            "),
                      (s.role !== null && _ctx.ROLE_LABEL_KEYS[s.role] !== undefined)
                        ? (_openBlock(), _createElementBlock("span", {
                            key: 0,
                            class: "role-label",
                            "data-role": s.role
                          }, _toDisplayString(_ctx.t(_ctx.ROLE_LABEL_KEYS[s.role]!)), 9 /* TEXT, PROPS */, _hoisted_13))
                        : _createCommentVNode("v-if", true),
                      _createCommentVNode("\n              🔵 Story 6.14 (FR42) — ảnh neo `0` đứng Ở ĐẦU ô của câu ĐẦU TIÊN, TRƯỚC chữ.\n              `<ChapterImage>` không có `data-src-start` ⇒ cùng lý do vô hình với cắt/Hán Việt.\n            "),
                      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_ctx.leadingImagesOf(s.id), (asset) => {
                        return (_openBlock(), _createElementBlock("figure", {
                          key: 'lead-' + asset.asset_id,
                          class: "grid-image"
                        }, [
                          _createVNode(_component_ChapterImage, {
                            "file-name": asset.file_name,
                            "assets-dir": _ctx.editorAssetsDir,
                            "source-url": asset.source_url,
                            "alt-text": asset.alt_text ?? ''
                          }, null, 8 /* PROPS */, ["file-name", "assets-dir", "source-url", "alt-text"])
                        ]))
                      }), 128 /* KEYED_FRAGMENT */)),
                      _createCommentVNode("\n              AC8 — Hán Việt sống TRONG ô nguyên văn, hai chế độ FR19, người dùng tự bật tắt.\n              `surface-role=\"cell\"` nhượng lượt đăng ký cho CỘT — xem `hanVietSurfaces.ts`.\n            "),
                      (_ctx.showHanViet)
                        ? (_openBlock(), _createBlock(_component_SourceHanViet, {
                            key: 1,
                            "source-text": s.source_text,
                            "view-mode": _ctx.viewMode,
                            cuts: _ctx.cutOffsetsOf(s.id),
                            "glossary-terms": _ctx.glossaryTermsOf(s.id).spans,
                            "surface-role": "cell"
                          }, null, 8 /* PROPS */, ["source-text", "view-mode", "cuts", "glossary-terms"]))
                        : (_openBlock(), _createElementBlock(_Fragment, { key: 2 }, [
                            _createCommentVNode("\n              🔵 2026-08-17, AC7 — nguyên văn chia thành mảnh theo TẬP ĐIỂM CẮT đang chờ CỘNG\n              (Story 3.4b) biên thuật ngữ Glossary, mỗi ranh giới một mảnh. Không điểm nào ⇒\n              `sourcePiecesOf` trả đúng MỘT mảnh, tức đường thường giữ nguyên hình dạng cây.\n\n              ⚠️ Dấu cắt RỖNG và mang `aria-hidden` — nó không phải chữ, và một ký tự thật ở\n              đây sẽ đi vào `Selection.toString()` của Auto-Lookup rồi vào mọi lượt sao chép.\n\n              🔴 Story 3.4b — `cut-mark` vẽ CHỈ khi `piece.isPendingCut`: một mảnh sinh RIÊNG\n              bởi biên thuật ngữ (không phải một điểm cắt người dùng) không được vẽ dấu ngắt\n              đoạn giả — xem `sourcePieceStartsOf` và Design Notes \"ba tập điểm, đừng gộp\" của\n              `3-4b-…md`.\n            "),
                            (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_ctx.sourcePieceInfoOf(s.id, s.source_text), (piece, i) => {
                              return (_openBlock(), _createElementBlock(_Fragment, { key: i }, [
                                (i > 0 && piece.isPendingCut)
                                  ? (_openBlock(), _createElementBlock("span", _hoisted_14))
                                  : _createCommentVNode("v-if", true),
                                _createCommentVNode(" 🔵 2026-08-17 (AC9) — mỗi mảnh nay là một `<span>` mang NEO\n                     `data-src-start`. Trước lượt này mảnh là text node trần và\n                     `sourceCutOffsetOf` phải **đếm mù** mọi text node trong ô — phép đếm đó\n                     sai hẳn ở tab Hán Việt *(đo: 17 và 19 trên một câu 5 chữ)*.\n                     ⚠️ `<span>` KHÔNG thêm một ký tự nào vào `Selection.toString()`, nên\n                     Auto-Lookup (FR21) không bị chạm.\n                     🔵 Story 3.4b — `glossary-confirmed`/`glossary-pending` tô dấu thuật ngữ\n                     đã khớp. `@mouseenter`/`@mouseleave` là vế CHUỘT; vế BÀN PHÍM/TIÊU ĐIỂM\n                     đi qua `onSourceSelectionChange()` ở khối kịch bản phía trên — KHÔNG\n                     `tabindex` mới ở đây: caret di chuyển bằng `Selection.modify()` (AC11)\n                     không cần phần tử này tự focus được. Không đi qua `dispatch()` — Kiểm A\n                     của `check:commands` chỉ canh `@click`.\n                     aura-allow-text: nguyên văn của Tác phẩm — DỮ LIỆU, không chuỗi giao\n                     diện (NFR16). Không `v-html` (AD-16). "),
                                _createElementVNode("span", {
                                  class: _normalizeClass(["src-piece", {
                    'glossary-confirmed': piece.glossary?.isConfirmed === true,
                    'glossary-pending': piece.glossary?.isConfirmed === false,
                    'concordance-source': piece.concordance,
                  }]),
                                  "data-src-start": piece.start,
                                  onMouseenter: $event => (_ctx.onGlossaryPieceEnter(piece.glossary)),
                                  onMouseleave: _cache[11] || (_cache[11] = $event => (_ctx.onGlossaryPieceLeave()))
                                }, _toDisplayString(piece.text), 43 /* TEXT, CLASS, PROPS, NEED_HYDRATION */, _hoisted_15)
                              ], 64 /* STABLE_FRAGMENT */))
                            }), 128 /* KEYED_FRAGMENT */))
                          ], 64 /* STABLE_FRAGMENT */)),
                      _createCommentVNode(" 🔵 Story 6.14 (FR42) — ảnh SAU câu này, ở CUỐI ô, đúng neo hai ảnh cùng neo giữ nguyên thứ tự `asset.id`. "),
                      (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_ctx.trailingImagesOf(s.id), (asset) => {
                        return (_openBlock(), _createElementBlock("figure", {
                          key: asset.asset_id,
                          class: "grid-image"
                        }, [
                          _createVNode(_component_ChapterImage, {
                            "file-name": asset.file_name,
                            "assets-dir": _ctx.editorAssetsDir,
                            "source-url": asset.source_url,
                            "alt-text": asset.alt_text ?? ''
                          }, null, 8 /* PROPS */, ["file-name", "assets-dir", "source-url", "alt-text"])
                        ]))
                      }), 128 /* KEYED_FRAGMENT */))
                    ], 42 /* CLASS, PROPS, NEED_HYDRATION */, _hoisted_12),
                    (s.id === _ctx.stripAnchorSegmentId)
                      ? (_openBlock(), _createElementBlock("div", _hoisted_16))
                      : _createCommentVNode("v-if", true)
                  ], 64 /* STABLE_FRAGMENT */))
                  _item.memo = _memo
                  return _item
                }, _cache, 13), 128 /* KEYED_FRAGMENT */))
              ], 2 /* CLASS */),
              _createCommentVNode("\n          Cột ④ — BẢN DỊCH. Một bề mặt vùng chọn duy nhất, vai `'display'` (B1).\n\n          🔴 Ba handler sống ở **CỘT**, không ở từng ô: sự kiện nổi bọt, và N listener cho\n          9.850 hàng là N lượt đăng ký mà một lượt là đủ.\n        "),
              _createElementVNode("div", {
                ref: "colTgt",
                class: "col col-tgt tok-editor",
                onFocusout: _cache[17] || (_cache[17] = (...args) => (_ctx.onColumnFocusOut && _ctx.onColumnFocusOut(...args))),
                onMousedown: _cache[18] || (_cache[18] = (...args) => (_ctx.onCellMouseDown && _ctx.onCellMouseDown(...args))),
                onMouseup: _cache[19] || (_cache[19] = (...args) => (_ctx.onCellMouseUp && _ctx.onCellMouseUp(...args))),
                onBeforeinput: _cache[20] || (_cache[20] = (...args) => (_ctx.onBeforeInput && _ctx.onBeforeInput(...args))),
                onCompositionstart: _cache[21] || (_cache[21] = (...args) => (_ctx.onCompositionStart && _ctx.onCompositionStart(...args))),
                onCompositionend: _cache[22] || (_cache[22] = (...args) => (_ctx.onCompositionEnd && _ctx.onCompositionEnd(...args))),
                onInput: _cache[23] || (_cache[23] = (...args) => (_ctx.onEditInput && _ctx.onEditInput(...args))),
                onKeydown: _cache[24] || (_cache[24] = (...args) => (_ctx.onEditKeydown && _ctx.onEditKeydown(...args)))
              }, [
                _createCommentVNode("\n            aura-allow-text: BẢN DỊCH của người dùng — dữ liệu, không chuỗi giao diện.\n\n            🔴 `contenteditable` trên **MỌI** ô (Quyết định #3(b)). Không một binding động nào:\n            giá trị không đổi theo trạng thái, nên Vue không bao giờ vá thuộc tính này và engine\n            không bao giờ dựng lại editing host.\n\n            ⚠️ Văn bản render `s.target_text` — bản **LÚC NẠP**. Văn bản đang gõ đi đường\n            [`restoreEditedText`], KHÔNG qua một binding phản ứng. Xem khối *\"DOM SỞ HỮU VĂN\n            BẢN\"* ở script.\n\n            🔴 `tgt-para-end` — cờ kết đoạn của **BẢN DỊCH** (Story 2.5d, FR134/AD-46). Nó đọc\n            `s.is_target_paragraph_end`, tức **dữ liệu đã lưu** (AC4) — **không** suy từ\n            `is_paragraph_end` của cột nguyên văn, và **không** suy từ vị trí các `\\n` trong\n            `target_text`. Hai phép suy đó đều chạy ra kết quả trông đúng, và cả hai rẽ khỏi\n            đĩa đúng vào ngày người dùng đổi cờ đầu tiên.\n            ⚠️ Class đặt trên **từng ô**, vì một hàng KHÔNG phải một phần tử DOM — xem khối\n            đầu tệp. Cái giá hình học của lựa chọn này đã đo, ghi ở khối style cuối tệp.\n\n            🔵 CẬP NHẬT 2026-08-16 (code review) — `empty` đo bằng `.trim()`, KHÔNG `=== ''`.\n            Trước lượt vá này lưới hỏi `=== ''` còn Rust hỏi `target_text.trim().is_empty()`\n            (`commands/segment.rs`, đường `confirm_segment`). Hai định nghĩa \"rỗng\" chỉ trùng\n            nhau chừng nào ô KHÔNG chứa được khoảng trắng — và AC1 của Story 2.5d vừa cho nó\n            chứa `\\n`. Hệ quả đo được: bấm `Enter` trong một ô rỗng cho `\"\\n\"`, ô MẤT viền đứt\n            nên trông đã dịch, mà `confirm_segment` vẫn từ chối ký — người dùng không có cách\n            nào biết vì sao. 🔴 Rust là nguồn sự thật cho \"rỗng\"; lưới đi theo, không ngược lại.\n          "),
                (_openBlock(true), _createElementBlock(_Fragment, null, _renderList(_ctx.editorSegments, (s, i, ___, _cached) => {
                  const _memo = ([
              s,
              i,
              _ctx.selectedRowClassById.get(s.id),
              s.id === _ctx.stripAnchorSegmentId,
              _ctx.rowErrorTextById.get(s.id),
              _ctx.editorEditedShape.get(s.id),
              _ctx.ruleOf(s.id),
            ])
                  if (_cached && _cached.el && _cached.key === s.id && _isMemoSame(_cached, _memo)) return _cached
                  const _item = (_openBlock(), _createElementBlock(_Fragment, {
                    key: s.id
                  }, [
                    _createCommentVNode(" aura-allow-text: bản dịch của người dùng, dữ liệu. "),
                    _createElementVNode("div", {
                      class: _normalizeClass(["cell cell-tgt", [
              {
                'para-end': s.is_paragraph_end,
                refused: _ctx.rowErrorLabelById.has(s.id),
                empty: (_ctx.editorEditedShape.get(s.id) ?? _ctx.textShapeOf(s.target_text)) < 2,
                editing: _ctx.editorCaretSegmentId === s.id,
                omitted: s.is_omitted,
                'tgt-para-end': s.is_target_paragraph_end,
              },
              _ctx.selectedRowClassById.get(s.id),
            ]]),
                      "data-segment-id": s.id,
                      "data-col": "tgt",
                      "data-unlinked": _ctx.unlinkedIds.has(s.id) ? 'true' : undefined,
                      contenteditable: "true",
                      role: "textbox",
                      "aria-multiline": "true",
                      "aria-label": _ctx.cellLabelOf(s.id, i),
                      "data-row-error": _ctx.rowErrorTextById.get(s.id),
                      "aria-details": s.id === _ctx.stripAnchorSegmentId ? 'strip-track' : undefined
                    }, _toDisplayString(s.target_text), 11 /* TEXT, CLASS, PROPS */, _hoisted_17),
                    (s.id === _ctx.stripAnchorSegmentId)
                      ? (_openBlock(), _createElementBlock("div", _hoisted_18))
                      : _createCommentVNode("v-if", true)
                  ], 64 /* STABLE_FRAGMENT */))
                  _item.memo = _memo
                  return _item
                }, _cache, 15), 128 /* KEYED_FRAGMENT */))
              ], 544 /* NEED_HYDRATION, NEED_PATCH */),
              _createCommentVNode(" Must follow the four columns in DOM order: it paints over and takes clicks in col-tgt only by coming later. "),
              _createElementVNode("div", {
                id: "strip-track",
                ref: "stripTrack",
                class: _normalizeClass(["strip-track", { 'strip-track-off': _ctx.stripAnchorIndex < 0 }]),
                style: _normalizeStyle(_ctx.stripAnchorIndex < 0 ? undefined : { gridRow: _ctx.stripAnchorIndex + 2 }),
                onFocusout: _cache[25] || (_cache[25] = (...args) => (_ctx.onStripFocusOut && _ctx.onStripFocusOut(...args)))
              }, [
                _createVNode(_component_TmFuzzyStrip),
                _createVNode(_component_GlossaryConfirmStrip),
                _createVNode(_component_ProofreaderStrip, { place: "track" })
              ], 38 /* CLASS, STYLE, NEED_HYDRATION */)
            ], 4 /* STYLE */)
          ], 8 /* PROPS */, _hoisted_7))
        : _createCommentVNode("v-if", true)
    ]),
    _: 1 /* STABLE */
  }, 8 /* PROPS */, ["show-status"]))
}