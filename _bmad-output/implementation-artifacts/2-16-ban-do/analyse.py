import json, sys, statistics as st
for p in sys.argv[1:]:
    d = json.load(open(p)); o = d['origin'] / 1000
    L = d['layout']
    mo = [x for x in d['mouse'] if x['kind'] == 'down']
    mu = [x for x in d['mouse'] if x['kind'] == 'up']
    ms, ks = d['mouse_sent'], d['key_sent']
    def lat(recs, sent, field):
        return [round((o + r['f'] / 1000 - s) * 1000) if field == 'sent' else round(r['f'] - r['h']) for r, s in zip(recs, sent)]
    print(d['label'], 'n=%d' % d['sentences'], 'cells=%d' % d['nCells'],
          'layout_ms=%d' % round(L['f'] - L['h0']), 'cell_present_ms=%d' % round(L['cell'] - L['h0']))
    print('  mouse down: sent->f %s  h->f %s  hit=%s' % (lat(mo, ms, 'sent'), lat(mo, ms, 'h'), [x['id'] in d['targets'] for x in mo]))
    print('  mouse up:   sent->f %s  h->f %s' % (lat(mu, ms, 'sent'), lat(mu, ms, 'h')))
    print('  key:        sent->f %s  h->f %s  moved=%s' % (lat(d['keys'], ks, 'sent'), lat(d['keys'], ks, 'h'), [(k['before'], k['after'], k['later']) for k in d['keys']]))
    print('  load=%s' % d['load_before'])
    for k in d['keys']:
        fs = [f for f in d.get('focus', []) if f['h'] >= k['h'] - 1 and f['h'] - k['h'] < 3000]
        print('  key h=%d focusin after: %s' % (k['h'], [(f['id'], round(f['h'] - k['h']), round(f['f'] - k['h'])) for f in fs]))

    for pr in d.get('prof', []):
        if pr.get('f') is None: continue
        print('    bubble@%.0f calls=%s' % ((pr.get('bubble') or 0)-pr['h'], [c for c in pr.get('calls',[]) if c[1] >= 1][:8]))
        print('  prof %s: h->task %.0f  style %.0f  layout %.0f  ->raf1 %.0f  ->f %.0f  muts=%d blocks=%d' % (pr['kind'], pr['task']-pr['h'], pr['style']-pr['task'], pr['layout']-pr['style'], (pr['raf1'] or pr['layout'])-pr['layout'], pr['f']-(pr['raf1'] or pr['layout']), pr['muts'], pr['blocks']))
    q = d.get('quiet')
    if q and q.get('q'): print('  quiet: forced_layout_ms=%.0f quiet_after=%.0f ms from layout start' % (q['forced_layout_ms'], q['q']-L['h0']))
