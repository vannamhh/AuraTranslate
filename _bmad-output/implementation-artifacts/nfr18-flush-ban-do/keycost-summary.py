import json,sys,glob
print('file                 rows  vErr  webkit p50/p95  in->raf p50/p95  in->sc p95  queue(last3rd) p50  lag[n] last-quarter med')
for pat in sys.argv[1:]:
  for f in sorted(glob.glob(f'keys-{pat}.json')):
    d=json.load(open(f)); a=d['all']; l=d['last_third']
    inj={}
    for ln in open(f.replace('.json','.typing.log')):
        if ln.startswith('#'): continue
        p=ln.split()
        if len(p)>=2: inj[int(p[0])]=float(p[1])
    lag=sorted(e/1000-inj[n] for n,e,q in d['lag_n_epoch_ms_q'] if n in inj)
    lg=[e/1000-inj[n] for n,e,q in d['lag_n_epoch_ms_q'] if n in inj]
    q=lg[-max(1,len(lg)//4):]; q.sort()
    w=a['webkit_edit_bi_to_input']; r=a['input_out_to_raf']; s=a['input_out_to_selectionchange']
    print(f"{f[5:-5]:16s} {d['nRows']:5d} {d.get('variantErr','')[:6]:5s} {w['p50']:5}/{w['p95']:<5} {r['p50']:5}/{r['p95']:<5} {s['p95']:6} {l['queue_ev_to_kdIn']['p50']:10} {q[len(q)//2] if q else None!s:>10}")
