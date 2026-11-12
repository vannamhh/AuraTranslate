import json,sys,collections,os
def load(p):
    d=json.load(open(p))
    return d['traceEvents'] if isinstance(d,dict) else d
for p in sys.argv[1:]:
    ev=load(p)
    names={}
    for e in ev:
        if e.get('ph')=='M' and e.get('name')=='thread_name': names[(e['pid'],e['tid'])]=e['args']['name']
    disp=collections.Counter()
    for e in ev:
        if e.get('name')=='EventDispatch': disp[(e['pid'],e['tid'])]+=1
    main=max(disp,key=disp.get) if disp else None
    m=[e for e in ev if (e.get('pid'),e.get('tid'))==main and e.get('ph') in ('X','B','E')]
    ts=[e['ts'] for e in m if 'ts' in e]; span=(max(ts)-min(ts))/1e3 if ts else 0
    types=collections.Counter(e['args']['data'].get('type') for e in m if e.get('name')=='EventDispatch' and e.get('ph')=='X')
    tot=collections.Counter(); cnt=collections.Counter()
    for e in m:
        if e.get('ph')=='X' and 'dur' in e: tot[e['name']]+=e['dur']/1e3; cnt[e['name']]+=1
    long=[e['dur']/1e3 for e in m if e.get('name')=='RunTask' and e.get('ph')=='X' and e.get('dur',0)>50e3]
    lay=[e for e in m if e.get('name')=='Layout' and e.get('ph')=='X']
    totobj=max((e.get('args',{}).get('beginData',{}).get('totalObjects',0) for e in lay),default=0)
    print(f"\n=== {os.path.basename(p)}  main={names.get(main)} span={span:.0f}ms  layoutTotalObjects(max)={totobj}")
    print(' events:',dict((k,v) for k,v in types.most_common(12)))
    print(' long tasks >50ms: n=%d sum=%.0fms max=%.0fms'%(len(long),sum(long),max(long,default=0)))
    for k in ['RunTask','FunctionCall','EventDispatch','TimerFire','UpdateLayoutTree','Layout','PrePaint','Paint','Layerize','Commit','MajorGC','MinorGC','V8.GCScavenger','ParseHTML','HitTest','IntersectionObserverController::computeIntersections','UpdateLayer','ScheduleStyleRecalculation']:
        if k in tot: print(f'  {k:28s} n={cnt[k]:6d} total={tot[k]:9.0f}ms')
