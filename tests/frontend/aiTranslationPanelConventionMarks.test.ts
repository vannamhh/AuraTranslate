import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { mount } from "@vue/test-utils";
import { flushPromises } from "./support/flushMicrotasks";
import { computed, ref } from "vue";
import type { Component } from "vue";
import type { ConventionSpanWire } from "../../src/config/convention";
import type { ChapterSegment } from "../../src/config/segment";
import { FIXTURE_SEGMENTS } from "./support/segmentFixture";
import type { CommandDeps } from "../../src/commands";

const runMock = vi.fn();
const batchMock = vi.fn();
const scanMock = vi.fn();
const TEXT = "Đảm bảo an toàn.";

vi.mock("../../src/config/aitranslate", () => ({
  runAiTranslateSegment: (...args: unknown[]) => runMock(...args),
  cancelAiTranslateCall: vi.fn(),
  runAiTranslateBatchCall: (...args: unknown[]) => batchMock(...args),
  resendAiCall: vi.fn(),
}));
vi.mock("../../src/config/convention", () => ({
  scanConventionSuggestions: (...args: unknown[]) => scanMock(...args),
}));
vi.mock("../../src/conventionState", () => ({ conventionGeneration: ref(1) }));

function span(
  start: number,
  end: number,
  replacement: string | null = "bảo đảm",
): ConventionSpanWire {
  return {
    start,
    end,
    rule_key: "đảm bảo",
    avoid: "đảm bảo",
    replacement_cased: replacement,
    tier: "global",
  };
}

type BatchEvent = {
  kind: string;
  call_index?: number;
  segment_id?: number;
  [k: string]: unknown;
};

/** Holds the batch run open so the test can emit events, then settle it when it wants `done`. */
function pendingBatchRun(): {
  emit: (e: BatchEvent) => void;
  settle: () => void;
} {
  let captured: (e: BatchEvent) => void = () => {
    throw new Error("runAiTranslateBatchCall chưa được gọi");
  };
  let resolveRun: (r: unknown) => void = () => {
    throw new Error("runAiTranslateBatchCall chưa được gọi");
  };
  batchMock.mockImplementation(
    (_a: unknown, _b: unknown, onEvent: (e: BatchEvent) => void) => {
      captured = onEvent;
      return new Promise((res) => {
        resolveRun = res;
      });
    },
  );
  return {
    emit: (e) => captured(e),
    settle: () =>
      resolveRun({
        value: { state: "done", usage: null, maybe_truncated: false },
        error: null,
      }),
  };
}

function receivedEvent(id: number, text: string): BatchEvent {
  return {
    kind: "received",
    call_index: 1,
    segment_id: id,
    text,
    has_han: false,
    source_hash: `h${id}`,
  };
}

/** Fixture sentences cut to the ids a case needs, with the source text that case wants to see. */
function segmentsWithSource(sources: Map<number, string>): ChapterSegment[] {
  return FIXTURE_SEGMENTS.flatMap((s) => {
    const source = sources.get(s.id);
    return source === undefined ? [] : [{ ...s, source_text: source }];
  });
}

async function freshPanel(segments: ChapterSegment[] = []) {
  vi.resetModules();
  runMock.mockReset();
  batchMock.mockReset();
  scanMock.mockReset();
  const caret = ref<number | null>(5);
  vi.doMock("../../src/panels/editorPanelState", () => ({
    editorCaretSegmentId: caret,
    editorActiveSegmentId: computed(() => caret.value),
    promoteAiTranslationToEditor: vi.fn(),
    editorPendingPromote: ref(null),
    editorPendingPromoteBatch: ref(null),
    editorPromoteBatchReport: ref(null),
    editorSegments: ref(segments),
    editorEditedShape: ref(new Map()),
    editorPromoteAiTranslationError: ref(null),
  }));
  const commands = await import("../../src/commands");
  const state = await import("../../src/aiTranslateState");
  const batchState = await import("../../src/aiTranslateBatchState");
  const AiTranslationPanel = (
    await import("../../src/panels/AiTranslationPanel.vue")
  ).default;
  const { aiTranslateHandlers } = await import("../../src/aiTranslateHandlers");
  commands.installCommands({ ...aiTranslateHandlers } as CommandDeps);
  return { state, batchState, AiTranslationPanel };
}

function mountPanel(panel: Component) {
  return mount(panel, {
    props: { params: { params: {} } },
    attachTo: document.body,
  });
}

beforeEach(() => {
  document.body.innerHTML = "";
});

afterEach(() => {
  vi.restoreAllMocks();
});

describe("AiTranslationPanel convention marks", () => {
  it("a finished suggestion shows a titled mark and offers no strip and no replace button", async () => {
    const { AiTranslationPanel } = await freshPanel();
    scanMock.mockResolvedValue({
      scan: {
        generation: 1,
        chapter_id: 1,
        items: [{ segment_id: 5, scanned_text: TEXT, spans: [span(0, 7)] }],
        ignored: 0,
        glossary_suppressed: 0,
        overlapped: 0,
      },
      error: null,
    });
    runMock.mockImplementation(
      async (
        _id: number,
        _set: string | null,
        onToken: (t: string) => void,
      ) => {
        onToken(TEXT);
        return {
          value: { state: "done", maybe_truncated: false, usage: null },
          error: null,
        };
      },
    );
    const wrapper = mountPanel(AiTranslationPanel);
    await wrapper.vm.$nextTick();
    await wrapper.get("[data-ai-translate-run]").trigger("click");
    await flushPromises();
    await wrapper.vm.$nextTick();
    const marked = wrapper.findAll("[data-convention-suggestion-mark]");
    expect(marked).toHaveLength(1);
    expect(marked[0].text()).toBe("Đảm bảo");
    expect(marked[0].classes()).toContain("convention-mark");
    expect(marked[0].attributes("title")).toBeTruthy();
    expect(wrapper.get("[data-ai-translate-text]").text()).toBe(TEXT);
    expect(wrapper.find("[data-convention-accept]").exists()).toBe(false);
    expect(wrapper.find("[data-convention-strip]").exists()).toBe(false);
    expect(wrapper.find("[data-convention-replace-chapter]").exists()).toBe(
      false,
    );
    wrapper.unmount();
  });

  it("a suggestion still streaming has no mark and triggers no scan", async () => {
    const { AiTranslationPanel } = await freshPanel();
    let tokenCb: (t: string) => void = () => undefined;
    runMock.mockImplementation(
      (_id: number, _set: string | null, onToken: (t: string) => void) => {
        tokenCb = onToken;
        return new Promise(() => undefined);
      },
    );
    const wrapper = mountPanel(AiTranslationPanel);
    await wrapper.vm.$nextTick();
    await wrapper.get("[data-ai-translate-run]").trigger("click");
    await flushPromises();
    tokenCb(TEXT);
    await wrapper.vm.$nextTick();
    expect(wrapper.get("[data-ai-translate-text]").text()).toBe(TEXT);
    expect(wrapper.findAll("[data-convention-suggestion-mark]")).toHaveLength(
      0,
    );
    expect(scanMock).not.toHaveBeenCalled();
    wrapper.unmount();
  });

  it("a finished batch marks each received row from the one scan it ran", async () => {
    const { batchState, AiTranslationPanel } = await freshPanel(
      segmentsWithSource(
        new Map([
          [11, "Nguồn một."],
          [12, "Nguồn hai."],
        ]),
      ),
    );
    scanMock.mockResolvedValue({
      scan: {
        generation: 1,
        chapter_id: 1,
        items: [{ segment_id: 11, scanned_text: TEXT, spans: [span(0, 7)] }],
        ignored: 0,
        glossary_suppressed: 0,
        overlapped: 0,
      },
      error: null,
    });
    const wrapper = mountPanel(AiTranslationPanel);
    const fake = pendingBatchRun();
    void batchState.runAiTranslateBatch(null, [11, 12]);
    await flushPromises();
    fake.emit({
      kind: "planned",
      calls: [{ call_index: 1, segment_ids: [11, 12] }],
    });
    fake.emit({ kind: "call_started", call_index: 1 });
    fake.emit(receivedEvent(11, TEXT));
    fake.emit(receivedEvent(12, "Bình thường."));
    fake.settle();
    await flushPromises();
    await wrapper.vm.$nextTick();

    expect(scanMock).toHaveBeenCalledTimes(1);
    expect((scanMock.mock.calls[0] as unknown[])[1]).toEqual([
      { segment_id: 11, text: TEXT },
      { segment_id: 12, text: "Bình thường." },
    ]);
    const rowTexts = wrapper.findAll("[data-ai-translate-batch-row-text]");
    expect(rowTexts).toHaveLength(2);
    expect(
      rowTexts[0]
        .findAll("[data-convention-suggestion-mark]")
        .map((m) => m.text()),
    ).toEqual(["Đảm bảo"]);
    expect(
      rowTexts[1].findAll("[data-convention-suggestion-mark]"),
    ).toHaveLength(0);
    wrapper.unmount();
  });

  it("a row that never arrived shows its source sentence with no mark, even when the scan answers for it", async () => {
    const { batchState, AiTranslationPanel } = await freshPanel(
      segmentsWithSource(
        new Map([
          [11, "Nguồn một."],
          [12, TEXT],
        ]),
      ),
    );
    scanMock.mockResolvedValue({
      scan: {
        generation: 1,
        chapter_id: 1,
        items: [
          { segment_id: 11, scanned_text: TEXT, spans: [span(0, 7)] },
          { segment_id: 12, scanned_text: TEXT, spans: [span(0, 7)] },
        ],
        ignored: 0,
        glossary_suppressed: 0,
        overlapped: 0,
      },
      error: null,
    });
    const wrapper = mountPanel(AiTranslationPanel);
    const fake = pendingBatchRun();
    void batchState.runAiTranslateBatch(null, [11, 12]);
    await flushPromises();
    fake.emit({
      kind: "planned",
      calls: [{ call_index: 1, segment_ids: [11, 12] }],
    });
    fake.emit({ kind: "call_started", call_index: 1 });
    fake.emit(receivedEvent(11, TEXT));
    fake.settle();
    await flushPromises();
    await wrapper.vm.$nextTick();

    expect(batchState.aiTranslateBatchRows.value[1]?.status).not.toBe(
      "received",
    );
    const rowTexts = wrapper.findAll("[data-ai-translate-batch-row-text]");
    expect(rowTexts[1].text()).toBe(TEXT);
    expect(
      rowTexts[1].findAll("[data-convention-suggestion-mark]"),
    ).toHaveLength(0);
    expect(
      rowTexts[0].findAll("[data-convention-suggestion-mark]"),
    ).toHaveLength(1);
    wrapper.unmount();
  });
});
