import { beforeEach, describe, expect, it, vi } from "vitest";
import { ref } from "vue";
import type {
  ConventionScanWire,
  ConventionSpanWire,
} from "../../src/config/convention";

const scanMock = vi.fn();
const acceptMock = vi.fn();
const revertMock = vi.fn();
const ignoreMock = vi.fn();
const flushMock = vi.fn();
const moveMock = vi.fn();
const replaceMock = vi.fn();
const scanSegmentsMock = vi.fn();
const flushListeners: ((ids: readonly number[]) => void)[] = [];
const pendingIds = new Set<number>();
const segments = ref<{ id: number; target_text: string; status: string }[]>([]);
const caret = ref<number | null>(null);

vi.mock("../../src/config/convention", () => ({
  scanConventionChapter: (...args: unknown[]) => scanMock(...args),
  scanConventionSegments: (...args: unknown[]) => scanSegmentsMock(...args),
  acceptConventionSpan: (...args: unknown[]) => acceptMock(...args),
  revertConventionSpan: (...args: unknown[]) => revertMock(...args),
  addConventionIgnore: (...args: unknown[]) => ignoreMock(...args),
}));
vi.mock("../../src/panels/editorPanelState", () => ({
  editorCaretSegmentId: caret,
  editorSegmentIsPending: (id: number) => pendingIds.has(id),
  editorSegments: segments,
  flushEditorBeforeDiscreteWrite: () => flushMock(),
  moveEditorCaretTo: (id: number) => {
    moveMock(id);
    caret.value = id;
    return true;
  },
  onEditorFlushed: (listener: (ids: readonly number[]) => void) => {
    flushListeners.push(listener);
    return () => undefined;
  },
  replaceEditorSegment: (...args: unknown[]) => replaceMock(...args),
}));
vi.mock("../../src/proofreadState", () => ({
  focusProofreadFinding: (segmentId: number, index: number) => {
    proofreadCursor = { segmentId, index };
  },
  proofreadCurrent: {
    get value() {
      return proofreadCursor;
    },
  },
  proofreadMarkPositions: () => proofreadMarks,
  suppressProofreadStripFor: vi.fn(),
}));

let proofreadMarks: {
  segmentId: number;
  index: number;
  start: number;
  end: number;
}[] = [];
let proofreadCursor: { segmentId: number; index: number } | null = null;

const state = await import("../../src/conventionState");

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

function scanOf(
  generation: number,
  entries: Record<number, [string, ConventionSpanWire[]]>,
): ConventionScanWire {
  return {
    generation,
    chapter_id: 1,
    segments: Object.entries(entries).map(([id, [text, spans]]) => ({
      segment_id: Number(id),
      scanned_text: text,
      spans,
    })),
    ignored: 0,
    glossary_suppressed: 0,
    overlapped: 0,
    not_scanned: 0,
  };
}

async function loadScan(
  entries: Record<number, [string, ConventionSpanWire[]]>,
): Promise<void> {
  scanMock.mockImplementation((generation: number) =>
    Promise.resolve({ scan: scanOf(generation, entries), error: null }),
  );
  await state.scanConventionNow();
}

beforeEach(() => {
  state.resetConvention();
  scanMock.mockReset();
  scanSegmentsMock.mockReset();
  pendingIds.clear();
  acceptMock.mockReset();
  revertMock.mockReset();
  ignoreMock.mockReset().mockResolvedValue({ ok: true, error: null });
  flushMock.mockReset().mockResolvedValue("clean");
  moveMock.mockReset();
  replaceMock
    .mockReset()
    .mockImplementation((id: number, patch: { target_text?: string }) => {
      segments.value = segments.value.map((s) =>
        s.id === id ? { ...s, ...patch } : s,
      );
    });
  proofreadMarks = [];
  proofreadCursor = null;
  segments.value = [
    { id: 1, target_text: "Đảm bảo an toàn.", status: "draft" },
    { id: 2, target_text: "không có gì", status: "draft" },
    { id: 3, target_text: "Cần đảm bảo. Đảm bảo nữa.", status: "confirmed" },
  ];
  caret.value = null;
});

describe("scan results", () => {
  it("drops a result older than the newest request", async () => {
    let release: (value: unknown) => void = () => undefined;
    scanMock.mockImplementationOnce(
      (generation: number) =>
        new Promise((resolve) => {
          release = () =>
            resolve({
              scan: scanOf(generation, {
                1: ["Đảm bảo an toàn.", [span(0, 7)]],
              }),
              error: null,
            });
        }),
    );
    const stale = state.scanConventionNow();
    state.resetConvention();
    release(null);
    await stale;
    expect(state.conventionResults.value.size).toBe(0);
  });

  it("keeps marks and raises the failure notice when a scan fails", async () => {
    await loadScan({ 1: ["Đảm bảo an toàn.", [span(0, 7)]] });
    scanMock.mockResolvedValueOnce({
      scan: null,
      error: {
        code: "x",
        message_key: "err.unknown",
        params: {},
        retryable: true,
      },
    });
    await state.scanConventionNow();
    expect(state.conventionResults.value.size).toBe(1);
    expect(state.conventionNotice.value?.key).toBe(
      "convention.status.scan_failed",
    );
  });

  it("a typed edit kills the marks of that sentence only", async () => {
    await loadScan({
      1: ["Đảm bảo an toàn.", [span(0, 7)]],
      3: ["Cần đảm bảo. Đảm bảo nữa.", [span(4, 11), span(13, 20)]],
    });
    state.clearConventionFor(1);
    expect([...state.conventionResults.value.keys()]).toEqual([3]);
  });

  it("a sentence whose text changed since the scan has no current spot", async () => {
    await loadScan({ 1: ["Đảm bảo an toàn.", [span(0, 7)]] });
    caret.value = 1;
    state.openConventionAt(1, 0);
    expect(state.conventionCurrent.value?.span.start).toBe(0);
    segments.value = [
      { id: 1, target_text: "Đảm bảo an toàn!", status: "draft" },
      ...segments.value.slice(1),
    ];
    expect(state.conventionCurrent.value).toBeNull();
  });
});

describe("F8 order across both families", () => {
  it("visits Proofreader findings and convention spots in text order and stops at the end", async () => {
    await loadScan({
      1: ["Đảm bảo an toàn.", [span(0, 7)]],
      3: ["Cần đảm bảo. Đảm bảo nữa.", [span(4, 11), span(13, 20)]],
    });
    proofreadMarks = [
      { segmentId: 1, index: 0, start: 8, end: 9 },
      { segmentId: 3, index: 0, start: 0, end: 3 },
    ];
    caret.value = 1;
    const visited: string[] = [];
    for (let i = 0; i < 6; i += 1) {
      state.stepMarks(1);
      const spot = state.conventionCurrent.value;
      visited.push(
        spot === null
          ? `${String(caret.value)}:other`
          : `${String(spot.segmentId)}:${String(spot.span.start)}`,
      );
      if (spot !== null) proofreadCursor = null;
      if (state.conventionNotice.value?.key === "convention.status.no_next")
        break;
    }
    expect(visited).toEqual([
      "1:0",
      "1:other",
      "3:other",
      "3:4",
      "3:13",
      "3:13",
    ]);
    expect(state.conventionNotice.value?.key).toBe("convention.status.no_next");
  });

  it("reports no_prev before the first mark and no_marks on an empty Chapter", async () => {
    state.stepMarks(1);
    expect(state.conventionNotice.value?.key).toBe(
      "convention.status.no_marks",
    );
    await loadScan({ 1: ["Đảm bảo an toàn.", [span(0, 7)]] });
    caret.value = 1;
    state.stepMarks(1);
    state.stepMarks(-1);
    expect(state.conventionNotice.value?.key).toBe("convention.status.no_prev");
  });
});

describe("F8 after the strip was hidden", () => {
  it("continues from the caret offset instead of returning to the first mark of the sentence", async () => {
    await loadScan({
      3: ["Cần đảm bảo. Đảm bảo nữa.", [span(4, 11), span(13, 20)]],
    });
    caret.value = 3;
    const cell = document.createElement("div");
    cell.setAttribute("data-col", "tgt");
    cell.setAttribute("data-segment-id", "3");
    cell.textContent = "Cần đảm bảo. Đảm bảo nữa.";
    document.body.appendChild(cell);
    try {
      state.openConventionAt(3, 13);
      state.hideConventionStrip();
      const text = cell.firstChild as Text;
      document.getSelection()?.setBaseAndExtent(text, 15, text, 15);
      state.stepMarks(1);
      expect(state.conventionNotice.value?.key).toBe(
        "convention.status.no_next",
      );
      expect(state.conventionCurrent.value).toBeNull();
      state.stepMarks(-1);
      expect(state.conventionCurrent.value?.span.start).toBe(4);
    } finally {
      cell.remove();
    }
  });
});

describe("accept and revert", () => {
  it("shifts the later spots of the sentence and opens a revert tray", async () => {
    await loadScan({
      3: ["Cần đảm bảo. Đảm bảo nữa.", [span(4, 11), span(13, 20)]],
    });
    caret.value = 3;
    state.openConventionAt(3, 4);
    acceptMock.mockResolvedValue({
      outcome: {
        segment_id: 3,
        target_text: "Cần bảo đảm. Đảm bảo nữa.",
        translation_origin: "other",
        status: "confirmed",
        chapter_started: false,
        start: 4,
        end: 11,
        phrase: "bảo đảm",
        version_created: true,
      },
      error: null,
    });
    await state.acceptConventionSpot();
    expect(acceptMock).toHaveBeenCalledWith(
      expect.objectContaining({
        segmentId: 3,
        start: 4,
        end: 11,
        expectedPhrase: "đảm bảo",
        ruleKey: "đảm bảo",
      }),
    );
    expect(
      state.conventionResults.value.get(3)?.spans.map((s) => [s.start, s.end]),
    ).toEqual([[13, 20]]);
    expect(state.conventionRevertible.value).toBe(true);
    expect(state.conventionRevertConfirmed.value).toBe(true);
  });

  it("a rule without a replacement offers no accept and sends nothing", async () => {
    await loadScan({ 1: ["Đảm bảo an toàn.", [span(0, 7, null)]] });
    caret.value = 1;
    state.openConventionAt(1, 0);
    await state.acceptConventionSpot();
    expect(acceptMock).not.toHaveBeenCalled();
  });

  it("keeps the sentence unchanged and shows the error when Rust refuses", async () => {
    await loadScan({ 1: ["Đảm bảo an toàn.", [span(0, 7)]] });
    caret.value = 1;
    state.openConventionAt(1, 0);
    acceptMock.mockResolvedValue({
      outcome: null,
      error: {
        code: "convention.text_changed",
        message_key: "convention.text_changed",
        params: { segment_id: "1" },
        retryable: false,
      },
    });
    await state.acceptConventionSpot();
    expect(replaceMock).not.toHaveBeenCalled();
    expect(state.conventionActionError.value?.code).toBe(
      "convention.text_changed",
    );
    expect(state.conventionResults.value.get(1)?.spans).toHaveLength(1);
  });

  it("revert restores the span and drops the tray", async () => {
    await loadScan({ 1: ["Đảm bảo an toàn.", [span(0, 7)]] });
    caret.value = 1;
    state.openConventionAt(1, 0);
    acceptMock.mockResolvedValue({
      outcome: {
        segment_id: 1,
        target_text: "Bảo đảm an toàn.",
        translation_origin: "other",
        status: "draft",
        chapter_started: false,
        start: 0,
        end: 7,
        phrase: "Bảo đảm",
        version_created: false,
      },
      error: null,
    });
    await state.acceptConventionSpot();
    revertMock.mockResolvedValue({
      outcome: {
        segment_id: 1,
        target_text: "Đảm bảo an toàn.",
        translation_origin: "other",
        status: "draft",
        chapter_started: false,
        start: 0,
        end: 7,
        phrase: "Đảm bảo",
        version_created: false,
      },
      error: null,
    });
    await state.revertConventionSpot();
    expect(revertMock).toHaveBeenCalledWith(
      expect.objectContaining({
        expectedText: "Bảo đảm an toàn.",
        expectedPhrase: "Bảo đảm",
        originalPhrase: "Đảm bảo",
      }),
    );
    expect(
      state.conventionResults.value.get(1)?.spans.map((s) => s.start),
    ).toEqual([0]);
    expect(state.conventionRevertible.value).toBe(false);
  });
});

describe("Chapter replace hooks", () => {
  it("the viewed rule count is the Chapter-wide count of spots with a replacement, and null without one", async () => {
    await loadScan({
      1: ["Đảm bảo an toàn.", [span(0, 7)]],
      3: [
        "Cần đảm bảo. Đảm bảo nữa.",
        [span(4, 11), span(13, 20), span(21, 24, null)],
      ],
    });
    expect(state.conventionViewedRuleCount.value).toBeNull();
    caret.value = 1;
    state.openConventionAt(1, 0);
    expect(state.conventionViewedRuleCount.value).toBe(3);
    expect(state.viewedConventionRuleKey()).toBe("đảm bảo");
  });

  it("a committed replace mirrors each sentence, drops its marks and tray, keeps other marks, and says what happened", async () => {
    await loadScan({
      1: ["Đảm bảo an toàn.", [span(0, 7)]],
      3: ["Cần đảm bảo. Đảm bảo nữa.", [span(4, 11), span(13, 20)]],
    });
    scanMock.mockClear();
    state.applyConventionReplaced({
      outcomes: [
        {
          segment_id: 1,
          target_text: "Bảo đảm an toàn.",
          translation_origin: "other",
          status: "draft",
          version_created: false,
        },
      ],
      replaced: 1,
      sentences: 1,
      confirmed: 0,
      drafts: 1,
      chapter_started: false,
    });
    expect(replaceMock).toHaveBeenCalledWith(1, {
      target_text: "Bảo đảm an toàn.",
      translation_origin: "other",
      status: "draft",
    });
    expect(state.conventionResults.value.has(1)).toBe(false);
    expect(state.conventionResults.value.has(3)).toBe(true);
    expect(state.conventionNotice.value).toEqual({
      key: "convention.status.replaced",
      params: {
        replaced_count: "1",
        sentence_count: "1",
        confirmed_count: "0",
        draft_count: "1",
      },
    });
    expect(scanMock).toHaveBeenCalledTimes(1);
  });
});

function segmentsScanOf(
  generation: number,
  entries: Record<number, [string, ConventionSpanWire[]]>,
  counts: Partial<
    Pick<
      ConventionScanWire,
      "ignored" | "glossary_suppressed" | "overlapped" | "not_scanned"
    >
  > = {},
): ConventionScanWire {
  return { ...scanOf(generation, entries), ...counts };
}

async function flushed(ids: number[]): Promise<void> {
  expect(flushListeners).toHaveLength(1);
  flushListeners[0](ids);
  await Promise.resolve();
  await Promise.resolve();
}

function deferred<T>(): { promise: Promise<T>; resolve: (value: T) => void } {
  let resolve: (value: T) => void = () => undefined;
  const promise = new Promise<T>((res) => {
    resolve = res;
  });
  return { promise, resolve };
}

describe("rescan after flush", () => {
  const ONE = "Đảm bảo an toàn.";
  const THREE = "Cần đảm bảo. Đảm bảo nữa.";
  const FIXED = "Bảo đảm rồi. Đảm bảo nữa.";

  it("typing alone never reaches the scan adapter", async () => {
    await loadScan({ 1: [ONE, [span(0, 7)]] });
    state.clearConventionFor(1);
    state.clearConventionFor(1);
    expect(scanSegmentsMock).not.toHaveBeenCalled();
  });

  it("a flush asks for exactly the flushed ids, once, under the current generation, and merges only those", async () => {
    await loadScan({
      1: [ONE, [span(0, 7)]],
      3: [THREE, [span(4, 11), span(13, 20)]],
    });
    const untouched = state.conventionResults.value.get(3);
    scanSegmentsMock.mockImplementation((generation: number) =>
      Promise.resolve({
        scan: segmentsScanOf(
          generation,
          { 1: ["Đảm bảo nhé.", [span(0, 7)]] },
          { overlapped: 1 },
        ),
        error: null,
      }),
    );
    await flushed([1]);
    expect(scanSegmentsMock).toHaveBeenCalledTimes(1);
    const [generation, ids] = scanSegmentsMock.mock.calls[0] as [
      number,
      number[],
    ];
    expect(ids).toEqual([1]);
    expect(generation).toBe(state.conventionGeneration.value);
    expect(state.conventionResults.value.get(1)?.scannedText).toBe(
      "Đảm bảo nhé.",
    );
    expect(state.conventionResults.value.get(3)).toBe(untouched);
    expect(state.conventionCounts.value.overlapped).toBe(0);
  });

  it("a flushed sentence that no longer has a mark loses its entry, and a repeated id is sent once", async () => {
    await loadScan({ 1: [ONE, [span(0, 7)]], 3: [THREE, [span(4, 11)]] });
    scanSegmentsMock.mockImplementation((generation: number) =>
      Promise.resolve({
        scan: segmentsScanOf(generation, {}, { not_scanned: 1 }),
        error: null,
      }),
    );
    await flushed([1, 1]);
    expect((scanSegmentsMock.mock.calls[0] as [number, number[]])[1]).toEqual([
      1,
    ]);
    expect([...state.conventionResults.value.keys()]).toEqual([3]);
    expect(state.conventionCounts.value.notScanned).toBe(0);
  });

  it("leaves the Chapter-wide counts alone, however many rescans run", async () => {
    scanMock.mockImplementation((generation: number) =>
      Promise.resolve({
        scan: segmentsScanOf(
          generation,
          { 1: [ONE, [span(0, 7)]] },
          { overlapped: 3 },
        ),
        error: null,
      }),
    );
    await state.scanConventionNow();
    scanSegmentsMock.mockImplementation((generation: number) =>
      Promise.resolve({
        scan: segmentsScanOf(
          generation,
          {},
          { glossary_suppressed: 2, not_scanned: 1 },
        ),
        error: null,
      }),
    );
    await flushed([1]);
    await flushed([1]);
    expect(state.conventionCounts.value).toEqual({
      ignored: 0,
      glossarySuppressed: 0,
      overlapped: 3,
      notScanned: 0,
    });
  });

  it("a whole-Chapter scan still in flight does not overwrite what a flush rescan wrote", async () => {
    await loadScan({ 1: [ONE, [span(0, 7)]] });
    const pending = deferred<{
      scan: ConventionScanWire | null;
      error: null;
    }>();
    let asked = 0;
    scanMock.mockImplementation((generation: number) => {
      asked = generation;
      return pending.promise;
    });
    const chapterScan = state.scanConventionNow();
    await Promise.resolve();
    scanSegmentsMock.mockImplementation((generation: number) =>
      Promise.resolve({
        scan: segmentsScanOf(generation, { 1: [FIXED, [span(13, 20)]] }),
        error: null,
      }),
    );
    await flushed([1]);
    expect(state.conventionResults.value.get(1)?.scannedText).toBe(FIXED);

    // The Chapter scan read the disk before the flush wrote, so it still reports the pre-flush text.
    pending.resolve({
      scan: scanOf(asked, { 1: [ONE, [span(0, 7)]] }),
      error: null,
    });
    await chapterScan;
    expect(state.conventionResults.value.get(1)?.scannedText).toBe(FIXED);
    expect(state.conventionResults.value.get(1)?.spans).toEqual([span(13, 20)]);
  });

  it("a flush rescan that cleared a sentence is not undone by the scan already in flight", async () => {
    await loadScan({ 1: [ONE, [span(0, 7)]] });
    const pending = deferred<{
      scan: ConventionScanWire | null;
      error: null;
    }>();
    let asked = 0;
    scanMock.mockImplementation((generation: number) => {
      asked = generation;
      return pending.promise;
    });
    const chapterScan = state.scanConventionNow();
    await Promise.resolve();
    scanSegmentsMock.mockImplementation((generation: number) =>
      Promise.resolve({ scan: segmentsScanOf(generation, {}), error: null }),
    );
    await flushed([1]);
    pending.resolve({
      scan: scanOf(asked, { 1: [ONE, [span(0, 7)]] }),
      error: null,
    });
    await chapterScan;
    expect(state.conventionResults.value.has(1)).toBe(false);
  });

  it("a result from an older generation is dropped", async () => {
    await loadScan({ 1: [ONE, [span(0, 7)]], 3: [THREE, [span(4, 11)]] });
    let release: () => void = () => undefined;
    scanSegmentsMock.mockImplementation(
      (generation: number) =>
        new Promise((resolve) => {
          release = () =>
            resolve({ scan: segmentsScanOf(generation, {}), error: null });
        }),
    );
    const before = state.conventionResults.value;
    await flushed([1]);
    state.resetConvention();
    release();
    await Promise.resolve();
    await Promise.resolve();
    expect(state.conventionResults.value).not.toBe(before);
    expect(state.conventionResults.value.size).toBe(0);
    state.resetConvention();
  });

  it("a failed rescan keeps the entries, says scan_failed and does not raise the Chapter failure flag", async () => {
    await loadScan({ 1: [ONE, [span(0, 7)]] });
    scanSegmentsMock.mockResolvedValue({
      scan: null,
      error: {
        code: "x",
        message_key: "err.unknown",
        params: {},
        retryable: true,
      },
    });
    await flushed([1]);
    expect(state.conventionResults.value.size).toBe(1);
    expect(state.conventionNotice.value?.key).toBe(
      "convention.status.scan_failed",
    );
    expect(state.conventionScanFailed.value).toBe(false);
  });
});

describe("F8 while the caret sentence is still pending", () => {
  it("says sentence_pending instead of no_marks when nothing is marked", () => {
    caret.value = 2;
    pendingIds.add(2);
    state.stepMarks(1);
    expect(state.conventionNotice.value?.key).toBe(
      "convention.status.sentence_pending",
    );
  });

  it("says sentence_pending instead of no_next / no_prev when no mark is left in that direction", async () => {
    await loadScan({ 1: ["Đảm bảo an toàn.", [span(0, 7)]] });
    caret.value = 2;
    pendingIds.add(2);
    state.stepMarks(1);
    expect(state.conventionNotice.value?.key).toBe(
      "convention.status.sentence_pending",
    );
    state.stepMarks(-1);
    expect(state.conventionCurrent.value?.span.start).toBe(0);
  });

  it("keeps no_marks when the caret sentence is not pending", () => {
    caret.value = 2;
    state.stepMarks(1);
    expect(state.conventionNotice.value?.key).toBe(
      "convention.status.no_marks",
    );
  });

  it("still walks to a mark that exists even while the caret sentence is pending", async () => {
    await loadScan({ 3: ["Cần đảm bảo. Đảm bảo nữa.", [span(4, 11)]] });
    caret.value = 2;
    pendingIds.add(2);
    state.stepMarks(1);
    expect(state.conventionCurrent.value?.segmentId).toBe(3);
    expect(state.conventionNotice.value).toBeNull();
  });
});

describe("Bỏ qua and Không phải lỗi", () => {
  const TWO = {
    1: ["Đảm bảo an toàn.", [span(0, 7)]],
    3: ["Cần đảm bảo. Đảm bảo nữa.", [span(4, 11), span(13, 20)]],
  } as Record<number, [string, ConventionSpanWire[]]>;

  it("Bỏ qua calls no Rust command, keeps every span and moves to the next mark", async () => {
    await loadScan(TWO);
    caret.value = 1;
    state.openConventionAt(1, 0);
    state.skipConventionSpot();
    expect(ignoreMock).not.toHaveBeenCalled();
    expect(acceptMock).not.toHaveBeenCalled();
    expect(revertMock).not.toHaveBeenCalled();
    expect(state.conventionResults.value.get(1)?.spans).toHaveLength(1);
    expect(state.conventionResults.value.get(3)?.spans).toHaveLength(2);
    expect(state.conventionCounts.value.ignored).toBe(0);
    expect(state.conventionCurrent.value?.segmentId).toBe(3);
  });

  it("Không phải lỗi calls the adapter exactly once with the viewed place, drops exactly one span and lands on the next", async () => {
    await loadScan(TWO);
    caret.value = 3;
    state.openConventionAt(3, 4);
    await state.ignoreConventionSpot();
    expect(ignoreMock).toHaveBeenCalledTimes(1);
    expect(ignoreMock).toHaveBeenCalledWith({
      segmentId: 3,
      expectedText: "Cần đảm bảo. Đảm bảo nữa.",
      start: 4,
      end: 11,
      expectedPhrase: "đảm bảo",
      ruleKey: "đảm bảo",
    });
    expect(
      state.conventionResults.value.get(3)?.spans.map((x) => [x.start, x.end]),
    ).toEqual([[13, 20]]);
    expect(state.conventionResults.value.get(1)?.spans).toHaveLength(1);
    // `counts` is a whole-Chapter total that only a whole-Chapter scan may restate — the same rule
    // the per-sentence rescan obeys — so the click itself must NOT move it.
    expect(state.conventionCounts.value.ignored).toBe(0);
    const here = state.conventionCurrent.value;
    expect([here?.segmentId, here?.span.start]).toEqual([3, 13]);
    expect(state.conventionNotice.value?.key).not.toBe("convention.status.no_next");
  });

  it("only a whole-Chapter scan restates `ignored`, and it does", async () => {
    await loadScan(TWO);
    caret.value = 3;
    state.openConventionAt(3, 4);
    await state.ignoreConventionSpot();
    expect(state.conventionCounts.value.ignored).toBe(0);
    scanMock.mockImplementation((generation: number) =>
      Promise.resolve({
        scan: { ...scanOf(generation, TWO), ignored: 1 },
        error: null,
      }),
    );
    await state.scanConventionNow();
    expect(state.conventionCounts.value.ignored).toBe(1);
  });

  it("a whole-Chapter scan still in flight does NOT bring back the place just remembered", async () => {
    await loadScan(TWO);
    caret.value = 3;
    state.openConventionAt(3, 4);
    // This scan read the disk BEFORE the write, so its entry for sentence 3 is the older one.
    let land: () => void = () => undefined;
    scanMock.mockImplementation(
      (generation: number) =>
        new Promise((resolve) => {
          land = () =>
            resolve({ scan: scanOf(generation, TWO), error: null });
        }),
    );
    const inFlight = state.scanConventionNow();
    await state.ignoreConventionSpot();
    land();
    await inFlight;
    expect(
      state.conventionResults.value.get(3)?.spans.map((x) => [x.start, x.end]),
    ).toEqual([[13, 20]]);
  });

  it("a whole-Chapter scan still in flight does NOT undo the replacement just accepted", async () => {
    await loadScan(TWO);
    caret.value = 3;
    state.openConventionAt(3, 4);
    acceptMock.mockResolvedValue({
      outcome: {
        segment_id: 3,
        target_text: "Cần bảo đảm. Đảm bảo nữa.",
        translation_origin: "other",
        status: "confirmed",
        chapter_started: false,
        start: 4,
        end: 11,
        phrase: "bảo đảm",
        version_created: true,
      },
      error: null,
    });
    let land: () => void = () => undefined;
    scanMock.mockImplementation(
      (generation: number) =>
        new Promise((resolve) => {
          land = () =>
            resolve({ scan: scanOf(generation, TWO), error: null });
        }),
    );
    const inFlight = state.scanConventionNow();
    await state.acceptConventionSpot();
    land();
    await inFlight;
    expect(state.conventionResults.value.get(3)?.scannedText).toBe(
      "Cần bảo đảm. Đảm bảo nữa.",
    );
  });

  it("after a forget and a fresh whole-Chapter scan the strip opens again", async () => {
    await loadScan(TWO);
    caret.value = 3;
    state.openConventionAt(3, 4);
    await state.ignoreConventionSpot();
    state.hideConventionStrip();
    expect(state.conventionCurrent.value).toBeNull();
    // What the card's Bỏ ghi nhớ does: the row goes in Rust and the Chapter is rescanned.
    await loadScan(TWO);
    state.openConventionAt(3, 4);
    expect(state.conventionCurrent.value?.span.start).toBe(4);
    state.hideConventionStrip();
    caret.value = 1;
    state.stepMarks(1);
    expect(state.conventionCurrent.value?.segmentId).toBe(1);
  });

  it("removing the last span of a sentence deletes its entry and ends on the remembered notice", async () => {
    await loadScan({ 1: ["Đảm bảo an toàn.", [span(0, 7)]] });
    caret.value = 1;
    state.openConventionAt(1, 0);
    await state.ignoreConventionSpot();
    expect(state.conventionResults.value.has(1)).toBe(false);
    expect(state.conventionNotice.value?.key).toBe("convention.status.ignored");
  });

  it("a refused write drops nothing and shows the error", async () => {
    await loadScan(TWO);
    caret.value = 1;
    state.openConventionAt(1, 0);
    ignoreMock.mockResolvedValue({
      ok: false,
      error: {
        code: "convention.text_changed",
        message_key: "err.convention.text_changed",
        params: {},
        retryable: false,
      },
    });
    await state.ignoreConventionSpot();
    expect(state.conventionResults.value.get(1)?.spans).toHaveLength(1);
    expect(state.conventionCounts.value.ignored).toBe(0);
    expect(state.conventionActionError.value?.code).toBe("convention.text_changed");
  });

  it("sends nothing when no spot is viewed", async () => {
    await state.ignoreConventionSpot();
    expect(ignoreMock).not.toHaveBeenCalled();
  });
});
