import json, sys, statistics as st
for p in sys.argv[1:]:
    d = json.load(open(p)); k = d['keys']; c = d['confirm']
    t1, t300 = d['sent']; o = d['origin'] / 1000
    n = len(k)
    sent = [t1 + (t300 - t1) * i / 299 for i in range(300)]
    lat = [(o + x['f'] / 1000 - sent[i]) * 1000 for i, x in enumerate(k) if x['f'] is not None]
    last = (o + k[-1]['f'] / 1000 - t300) * 1000 if n == 300 else None
    back = sum(1 for x in k if o + x['h'] / 1000 > t300)
    conf = None
    if c and c['f'] and c['f'] > 0:
        conf = (o + c['f'] / 1000 - (t300 + 0.02)) * 1000
    print(d['label'], 'n=%d' % d['sentences'], 'cells=%d' % d['nCells'], 'keys=%d/300' % n,
          'sendrate=%.2f/s' % (299 / (t300 - t1)),
          'lastkey_ms=%s' % (None if last is None else round(last)), 'backlog_at_last_sent=%d' % back,
          'lat_med=%.0f max=%.0f' % (st.median(lat), max(lat)), 'confirm_ms=%s' % (None if conf is None else round(conf)),
          'load=%s' % d['load_before'])
