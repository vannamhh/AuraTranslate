import json, sys
for p in sys.argv[1:]:
    d = json.load(open(p))
    print(d['label'], 'rows=%s' % d['ready'], 'load=%s' % d['load_before'])
    for s in d['sweeps']:
        st = s['stat']
        print('  %-8s rows=%d selected=%s  total med %.1f max %.1f >50:%d/%d | js med %.1f max %.1f | after-microtask med %.1f max %.1f' % (
            s['shape'], s['rows'], s['selected'], st['total']['median'], st['total']['max'], st['total']['over50'], st['total']['n'],
            st['js']['median'], st['js']['max'], st['flush']['median'], st['flush']['max']))
