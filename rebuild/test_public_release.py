"""Public release regression: stable identities and honest approval."""
import json
from pathlib import Path
import tempfile
import unittest
import zipfile
import pipeline
import site_output

class PublicReleaseTests(unittest.TestCase):
    def test_public_catalog_keeps_legacy_alias(self):
        rows=json.loads((pipeline.ROOT/'rebuild/sources.json').read_text())
        infos=[json.loads((pipeline.ROOT/r['path']/'res/source.json').read_text())['info'] for r in rows]
        self.assertEqual(len(infos),6)
        installed={}
        for path in (pipeline.ROOT/'rebuild/published-packages').glob('*.aix'):
            with zipfile.ZipFile(path) as archive:
                previous=json.loads(archive.read('Payload/source.json'))['info']
                if previous['id'] not in installed or previous['version']>installed[previous['id']]['version']: installed[previous['id']]=previous
        for info in infos:
            previous=installed[info['id']]
            self.assertIn(info['version'],(previous['version'],previous['version']+1))
            self.assertEqual(info['name'],previous['name'].replace(' (LUC1D)',' [PN]'))
            for key in set(previous)-{'name','version'}: self.assertEqual(info.get(key),previous[key],key)
        for info in infos:
            self.assertTrue(info['name'].endswith(' [PN]'),info['name'])
        self.assertEqual({i['id'] for i in infos},{'en.luc1d-asurascans','en.luc1d-weebcentral','multi.luc1d-nhentai','multi.luc1d-webtoon','multi.luc1d-imhentai','multi.luc1d-hentaifox'})
        self.assertEqual(len(pipeline.validate_manifest(rows,release=True)),6)
        self.assertTrue(all(not r['runtime_tested'] and not r['device_tested'] for r in rows))
        with tempfile.TemporaryDirectory() as td:
            root=Path(td)
            (root/'index.json').write_text(json.dumps({'name':'PanelNest','sources':infos}))
            (root/'build-report.json').write_text('{"release":true}')
            site_output.prepare(root)
            catalog=json.loads((root/'index.json').read_text())
            self.assertEqual(len(catalog['sources']),6)
            self.assertEqual(catalog,json.loads((root/'experimental/index.json').read_text()))
            self.assertEqual(catalog,json.loads((root/'index.min.json').read_text()))

    def test_user_authorization_requires_explicit_approval(self):
        rows=json.loads((pipeline.ROOT/'rebuild/sources.json').read_text())
        for field in ('release_authorized','publish','package_verified'):
            changed=json.loads(json.dumps(rows)); changed[0][field]=False
            with self.assertRaises(ValueError): pipeline.validate_manifest(changed,release=True)
