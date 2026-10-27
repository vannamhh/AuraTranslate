"""M2: one call per whole chapter, measure omission/merge/misalignment, tokens, timing.

usage: measure.py prep ZH_RAW EN_RAW OUTDIR
       measure.py run OUTDIR MODEL EFFORT(default|minimal) TEXT(zh|en) TAG
"""
import json, os, re, subprocess, sys, time, urllib.request

ENDPOINT = "https://generativelanguage.googleapis.com/v1beta/openai"

SYSTEM = (
    "Bạn là dịch giả chuyên nghiệp, dịch sang tiếng Việt.\n"
    "Đầu vào gồm nhiều khối, mỗi khối mở bằng một dòng `<<<SEGMENT n` và đóng bằng dòng `SEGMENT>>>`.\n"
    "Dịch từng khối riêng, trả về đúng từng ấy khối, cùng số n, cùng thứ tự, mỗi khối mở bằng `<<<SEGMENT n` và đóng bằng `SEGMENT>>>`.\n"
    "Không gộp, không tách, không bỏ khối nào, kể cả khối rất ngắn hay chỉ là thơ.\n"
    "Chỉ trả về các khối bản dịch: không giải thích, không ghi chú, không chép lại nguyên văn.\n"
    "Không dùng Markdown: không thêm tiêu đề, dấu ** hay ký hiệu định dạng."
)
GUIDANCE = (
    "Đây là văn học. Ưu tiên giọng văn, nhịp câu và hình ảnh của tác giả hơn việc dịch từng chữ.\n"
    "Ẩn dụ, biểu tượng, câu nhiều tầng nghĩa giữ nguyên tính mơ hồ, không giải thích cho rõ.\n"
    "Hán Việt dùng vừa phải, theo không khí tác phẩm.\n"
    "Lời thoại hợp với tuổi, nghề, hoàn cảnh nhân vật; tên người Trung Quốc theo Hán Việt."
)


def split_zh(raw):
    raw = re.sub(r"\{\{.*?\}\}", "", raw, flags=re.S)
    out = []
    for line in raw.splitlines():
        line = line.strip().lstrip(":").strip()
        if not line or line.startswith(("[[", "{{", "<")):
            continue
        line = re.sub(r"\[\[[^\]|]*\|([^\]]*)\]\]", r"\1", line)
        line = re.sub(r"<[^>]+>", "", line)
        parts = re.findall(r".+?(?:[。！？](?:[」』”’]+)?|$)", line)
        out += [p.strip() for p in parts if p.strip()]
    return out


ABBR = r"(?<!\bMr)(?<!\bMrs)(?<!\bDr)(?<!\bSt)(?<!\bMessrs)(?<!\bMme)"


def split_en(raw):
    start = [m.start() for m in re.finditer(r"^I\. A SCANDAL IN BOHEMIA", raw, re.M)][-1]
    end = [m.start() for m in re.finditer(r"^II\. THE RED-HEADED LEAGUE", raw, re.M)][-1]
    body = raw[start:end]
    paras = [re.sub(r"\s+", " ", p).strip() for p in re.split(r"\n\s*\n", body)]
    out = []
    for p in paras:
        if not p:
            continue
        pieces = re.split(r"(?:(?<=[.!?])|(?<=[.!?][\"’”']))\s+(?=[\"“‘'A-Z])", p)
        cur = ""
        for s in pieces:
            cur = f"{cur} {s}".strip() if cur else s.strip()
            if not re.search(r"\b(Mr|Mrs|Dr|St|Messrs|Mme)\.$", cur):
                out.append(cur)
                cur = ""
        if cur:
            out.append(cur)
    return out


def prompt_for(segs):
    blocks = "\n".join(f"<<<SEGMENT {i}\n{s}\nSEGMENT>>>" for i, s in enumerate(segs, 1))
    return f"{GUIDANCE}\n\nĐoạn nguồn (tiếng Trung hoặc tiếng Anh):\n{blocks}"


def api_key():
    return subprocess.run(
        ["security", "find-generic-password", "-s", "com.auratranslate.desktop", "-a", "ai_provider_api_key", "-w"],
        capture_output=True, text=True, check=True,
    ).stdout.strip()


BLOCK = re.compile(r"<<<SEGMENT (\d+)\s*\n(.*?)\n?SEGMENT>>>", re.S)
LENIENT = re.compile(r"(?:<<<)?SEGMENT (\d+)\s*\n(.*?)\n?SEGMENT>>>", re.S)


def run(outdir, model, effort, text, tag):
    segs = json.load(open(os.path.join(outdir, f"{text}.segments.json"), encoding="utf-8"))
    body = {
        "model": model,
        "stream": True,
        "stream_options": {"include_usage": True},
        "messages": [{"role": "system", "content": SYSTEM}, {"role": "user", "content": prompt_for(segs)}],
    }
    if effort != "default":
        body["reasoning_effort"] = effort
    req = urllib.request.Request(
        ENDPOINT + "/chat/completions",
        data=json.dumps(body).encode(),
        headers={"Authorization": "Bearer " + api_key(), "Content-Type": "application/json", "Accept": "text/event-stream"},
    )
    t0 = time.monotonic()
    first_token = first_seg = None
    text_out, usage, finish, status, err = "", None, None, None, None
    done, last_events = False, []
    try:
        with urllib.request.urlopen(req, timeout=180) as r:
            status = r.status
            buf = b""
            for chunk in iter(lambda: r.read(4096), b""):
                buf += chunk
                while b"\n\n" in buf:
                    ev, buf = buf.split(b"\n\n", 1)
                    data = "".join(l[5:].strip() for l in ev.decode("utf-8", "replace").splitlines() if l.startswith("data:"))
                    if data:
                        last_events = (last_events + [data[:600]])[-3:]
                    if data == "[DONE]":
                        done = True
                    if not data or data == "[DONE]":
                        continue
                    j = json.loads(data)
                    if j.get("usage"):
                        usage = j["usage"]
                    for ch in j.get("choices") or []:
                        d = (ch.get("delta") or {}).get("content")
                        if d:
                            if first_token is None:
                                first_token = time.monotonic() - t0
                            text_out += d
                            if first_seg is None and "SEGMENT>>>" in text_out:
                                first_seg = time.monotonic() - t0
                        if ch.get("finish_reason"):
                            finish = ch["finish_reason"]
    except urllib.error.HTTPError as e:
        status, err = e.code, e.read(2000).decode("utf-8", "replace")
    except Exception as e:  # noqa: BLE001
        err = repr(e)
    wall = time.monotonic() - t0

    strict = len(BLOCK.findall(text_out))
    got = [(int(n), t.strip()) for n, t in LENIENT.findall(text_out)]
    ids = [n for n, _ in got]
    n = len(segs)
    expected = set(range(1, n + 1))
    seen = set(ids)
    han = re.compile(r"[一-鿿]")
    res = {
        "tag": tag, "model": model, "effort": effort, "text": text, "status": status, "error": err,
        "segments_sent": n, "blocks_returned": len(got), "blocks_strict": strict,
        "missing": sorted(expected - seen), "duplicates": sorted({i for i in ids if ids.count(i) > 1}),
        "out_of_range": sorted(seen - expected), "out_of_order": ids != sorted(ids),
        "empty": [i for i, t in got if not t],
        "han_left": [i for i, t in got if han.search(t)] if text == "zh" else None,
        "same_as_source": [i for i, t in got if 0 < i <= n and t == segs[i - 1]],
        "finish_reason": finish, "done": done, "last_events": last_events, "usage": usage,
        "seconds_first_token": first_token, "seconds_first_segment": first_seg, "seconds_total": wall,
        "chars_out": len(text_out), "unparsed_tail": text_out[-300:] if len(got) < n else "",
    }
    with open(os.path.join(outdir, f"{tag}.out.txt"), "w", encoding="utf-8") as f:
        f.write(text_out)
    with open(os.path.join(outdir, f"{tag}.json"), "w", encoding="utf-8") as f:
        json.dump(res, f, ensure_ascii=False, indent=1)
    short = {k: (v if not isinstance(v, list) or len(v) < 15 else f"{len(v)} items") for k, v in res.items() if k != "unparsed_tail"}
    print(json.dumps(short, ensure_ascii=False))


if __name__ == "__main__":
    if sys.argv[1] == "prep":
        zh, en, outdir = sys.argv[2:5]
        os.makedirs(outdir, exist_ok=True)
        for name, segs in (("zh", split_zh(open(zh, encoding="utf-8").read())), ("en", split_en(open(en, encoding="utf-8").read()))):
            json.dump(segs, open(os.path.join(outdir, f"{name}.segments.json"), "w", encoding="utf-8"), ensure_ascii=False, indent=0)
            p = prompt_for(segs)
            try:
                import tiktoken
                tok = len(tiktoken.get_encoding("o200k_base").encode(SYSTEM + p))
            except ImportError:
                tok = None
            print(name, "segments", len(segs), "prompt chars", len(p), "o200k tokens", tok, "first", segs[:2], "last", segs[-1:])
    else:
        run(*sys.argv[2:7])
