#!/usr/bin/env python3
"""Genera Rust estático desde los JSON fijados; no requiere dependencias ni red."""
from _source import finish, load


def q(text):
    return '"' + ''.join('\\"' if c == '"' else '\\\\' if c == '\\' else
                         f'\\u{{{ord(c):x}}}' if ord(c) < 32 else c for c in text) + '"'


unicode, catalogs = load()
lines = ['// Generado por scripts/generate.py. No editar manualmente.',
         'use crate::{Entry, Metadata, Provenance, PageRange, Anomaly, Occurrence};',
         '#[derive(Debug)]', 'pub(crate) struct CatalogData {',
         'pub entries: &\'static [Entry], pub search_names: &\'static [&\'static str],',
         'pub by_code: &\'static [usize], pub metadata: &\'static Metadata,',
         'pub provenance: &\'static Provenance, pub anomalies: &\'static [Anomaly],',
         'pub entries_json: &\'static str, pub metadata_json: &\'static str,',
         'pub provenance_json: &\'static str, pub anomalies_json: &\'static str,', '}']
for d in catalogs:
    year = d['year']
    lines.append(f'pub(crate) static V{year}: CatalogData = CatalogData {{')
    lines.append('entries: &[')
    for e in d['entries']:
        parent = 'None' if e['parent_code'] is None else f'Some({q(e["parent_code"])})'
        lines.append('Entry { code: '+q(e['code'])+', name: '+q(e['name'])+', parent_code: '+parent+' },')
    lines.append('], search_names: &[' + ','.join(q(s) for s in d['search_names']) + '],')
    lines.append('by_code: &[' + ','.join(map(str, d['by_code'])) + '],')
    m = d['metadata']
    lines.append(f'metadata: &Metadata {{ pcge_version: {q(year)}, schema_version: {m["schema_version"]}, dataset_revision: {m["dataset_revision"]}, entry_count: {m["entry_count"]} }},')
    lines.append('provenance: &Provenance {')
    for key, value in d['source'].items():
        if isinstance(value, dict):
            lines.append(f'{key}: PageRange {{ first: {value["first"]}, last: {value["last"]} }},')
        else:
            lines.append(f'{key}: {q(value)},')
    lines.append('}, anomalies: &[')
    for a in d['anomalies']:
        lines.append('Anomaly {')
        for key in ('id', 'status', 'description', 'decision', 'confirmation_no_invented_code'):
            lines.append(f'{key}: {q(a[key])},')
        lines.append('kind: '+q(a['type'])+', codes: &['+','.join(q(c) for c in a.get('codes', [a.get('code')]))+'],')
        lines.append('occurrences: &[')
        for occurrence in a['occurrences']:
            lines.append('Occurrence { '+','.join(key+': '+(str(value) if isinstance(value,int) else q(value)) for key,value in occurrence.items())+' },')
        lines.append('], },')
    lines.append('],')
    for field, name in [('entries_json','entries'),('metadata_json','metadata'),('provenance_json','source'),('anomalies_json','anomalies')]:
        lines.append(f'{field}: include_str!("../data/{year}/{name}.json"),')
    lines.append('};')
u = ['// Generado desde Unicode '+unicode['unicode_version']+'. Véase licenses/LICENSE-UNICODE.',
     'pub const VERSION: &str = '+q(unicode['unicode_version'])+';',
     'pub const WHITESPACE: &[u32] = &['+','.join(map(str,unicode['whitespace_codepoints']))+'];',
     'pub const MAPPING: &[(u32, &str)] = &[']
u.extend(f'({code}, {q(replacement)}),' for code,replacement in unicode['mappings'])
u.append('];')
finish({'src/data.rs':'\n'.join(lines)+'\n', 'src/unicode_data.rs':'\n'.join(u)+'\n'})
