import json,subprocess,sys,urllib.request
sys.path.insert(0, sys.argv[1]); import measure as m
segs=json.load(open(sys.argv[2],encoding='utf-8'))[int(sys.argv[3]):int(sys.argv[4])]
key=m.api_key()
body={"systemInstruction":{"parts":[{"text":m.SYSTEM}]},"contents":[{"role":"user","parts":[{"text":m.prompt_for(segs)}]}]}
req=urllib.request.Request(f"https://generativelanguage.googleapis.com/v1beta/models/{sys.argv[5]}:generateContent",data=json.dumps(body).encode(),headers={"x-goog-api-key":key,"Content-Type":"application/json"})
r=json.load(urllib.request.urlopen(req,timeout=180))
c=r["candidates"][0]
txt="".join(p.get("text","") for p in c.get("content",{}).get("parts",[]))
print(json.dumps({"finishReason":c.get("finishReason"),"citation":bool(c.get("citationMetadata")),"usage":r.get("usageMetadata"),"blocks":txt.count("SEGMENT>>>"),"tail":txt[-160:]},ensure_ascii=False))
