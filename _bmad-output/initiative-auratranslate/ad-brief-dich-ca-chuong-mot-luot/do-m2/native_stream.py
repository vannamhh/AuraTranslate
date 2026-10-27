import json,sys,time,urllib.request
sys.path.insert(0, sys.argv[1]); import measure as m
segs=json.load(open(sys.argv[2],encoding='utf-8'))
body={"systemInstruction":{"parts":[{"text":m.SYSTEM}]},"contents":[{"role":"user","parts":[{"text":m.prompt_for(segs)}]}]}
req=urllib.request.Request(f"https://generativelanguage.googleapis.com/v1beta/models/{sys.argv[3]}:streamGenerateContent?alt=sse",data=json.dumps(body).encode(),headers={"x-goog-api-key":m.api_key(),"Content-Type":"application/json"})
t0=time.monotonic(); txt=""; reasons=[]; last=None; usage=None
with urllib.request.urlopen(req,timeout=180) as r:
    buf=b""
    for ch in iter(lambda:r.read(4096),b""):
        buf+=ch
        while b"\n\n" in buf:
            ev,buf=buf.split(b"\n\n",1)
            d="".join(l[5:].strip() for l in ev.decode("utf-8","replace").splitlines() if l.startswith("data:"))
            if not d: continue
            j=json.loads(d); last=d[:500]; usage=j.get("usageMetadata",usage)
            for c in j.get("candidates",[]):
                txt+="".join(p.get("text","") for p in c.get("content",{}).get("parts",[]))
                if c.get("finishReason"): reasons.append(c["finishReason"])
print(json.dumps({"finishReasons":reasons,"blocks":txt.count("SEGMENT>>>"),"usage":usage,"seconds":time.monotonic()-t0,"last_event":last},ensure_ascii=False))
