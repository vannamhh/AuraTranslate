import json,sys,collections,os,statistics as st
for p in sys.argv[1:]:
    d=json.load(open(p)); ev=d['traceEvents'] if isinstance(d,dict) else d
    disp=collections.Counter((e['pid'],e['tid']) for e in ev if e.get('name')=='EventDispatch')
    main=max(disp,key=disp.get)
    m=sorted([e for e in ev if (e.get('pid'),e.get('tid'))==main and e.get('ph')=='X' and 'dur' in e],key=lambda e:e['ts'])
    dispatches=[e for e in m if e['name']=='EventDispatch']
    lay=[e for e in m if e['name']=='Layout']
    par=collections.Counter(); dirty=[]; durs=[]; partial=collections.Counter(); stacks=collections.Counter(); roots=collections.Counter()
    for L in lay:
        enc=[x for x in dispatches if x['ts']<=L['ts']<=x['ts']+x['dur']]
        t=enc[-1]['args']['data'].get('type') if enc else '(none)'
        par[t]+=1
        b=L.get('args',{}).get('beginData',{})
        dirty.append(b.get('dirtyObjects',0)); durs.append(L['dur']/1e3); partial[b.get('partialLayout')]+=1
        s=b.get('stackTrace') or []
        if s: stacks[' < '.join(f"{f.get('functionName') or '?'}" for f in s[:4])]+=1
        ed=L.get('args',{}).get('endData',{})
        r=ed.get('layoutRoots') or []
        roots[len(r)]+=1
    print(f"\n=== {os.path.basename(p)} layouts={len(lay)} dur med={st.median(durs):.1f} p90={sorted(durs)[int(.9*len(durs))]:.1f} max={max(durs):.0f}ms dirtyObjects med={st.median(dirty)} max={max(dirty)} partial={dict(partial)} roots={dict(roots)}")
    print('  inside event:',dict(par.most_common(6)))
    for k,v in stacks.most_common(5): print('  stack',v,k)
