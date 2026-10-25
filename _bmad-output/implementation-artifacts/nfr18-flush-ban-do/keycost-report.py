import json,sys,glob
# usage: keycost-report.py <label> ; prints per-session stage table and [n] lag trend
for f in sorted(glob.glob(f'keys-{sys.argv[1]}-*.json')):
    d=json.load(open(f)); log=f.replace('.json','.typing.log')
    inj={}
    for l in open(log):
        if l.startswith('#'): continue
        p=l.split()
        if len(p)>=2: inj[int(p[0])]=float(p[1])
    print('==',f,'rows',d['nRows'],'keydowns',d['nKeydown'])
    for part in ('all','first_third','last_third'):
        s=d[part]
        print(' ',part,'n',s['n'])
        for k,v in s.items():
            if isinstance(v,dict): print(f'    {k:30s} p50 {v["p50"]} p95 {v["p95"]} max {v["max"]} (n={v["n"]})')
    lag=[(n,(e/1000.0)-inj[n]) for n,e,q in d['lag_n_epoch_ms_q'] if n in inj]
    if lag:
        k=len(lag); a=lag[:max(1,k//4)]; b=lag[-max(1,k//4):]
        print('  lag[n] -> noteEditorEdit: first quarter med %.2f s, last quarter med %.2f s, max %.2f s (n=%d)'%(sorted(x for _,x in a)[len(a)//2],sorted(x for _,x in b)[len(b)//2],max(x for _,x in lag),k))
