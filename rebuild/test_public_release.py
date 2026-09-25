"""Public release regression: stable identities and honest approval."""
import json
import hashlib
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
        readme=(pipeline.ROOT/'README.md').read_text(encoding='utf-8')
        self.assertEqual(len(infos),6)
        installed={}
        for path in (pipeline.ROOT/'rebuild/published-packages').glob('*.aix'):
            with zipfile.ZipFile(path) as archive:
                previous=json.loads(archive.read('Payload/source.json'))['info']
                if previous['id'] not in installed or previous['version']>installed[previous['id']]['version']: installed[previous['id']]=previous
        for info in infos:
            self.assertIn(f"`{info['id']}` | {info['version']} |",readme,f"{info['id']} README version")
            previous=installed[info['id']]
            row=next(r for r in rows if r['id']==info['id'])
            verification_path=pipeline.ROOT/row['path']/'verification.json'
            self.assertTrue(verification_path.is_file(),f"{info['id']} verification record")
            verification=json.loads(verification_path.read_text())
            self.assertEqual(verification['version'],info['version'],f"{info['id']} verification version")
            self.assertEqual(verification['fixture_tests_passed'],row['fixture_tests_passed'],f"{info['id']} fixture count")
            self.assertEqual(verification['runtime_tested'],row['runtime_tested'],f"{info['id']} runtime evidence")
            self.assertEqual(verification['device_tested'],row['device_tested'],f"{info['id']} device evidence")
            self.assertEqual(verification.get('device_test_evidence'),row.get('device_test_evidence'),f"{info['id']} device evidence details")
            self.assertEqual(set(verification['implemented']),set(row['implemented']),f"{info['id']} implemented features")
            package_name=f"{info['id']}-v{info['version']}.aix"
            package=pipeline.ROOT/'rebuild/published-packages'/package_name
            if info['version']==previous['version']:
                self.assertTrue(package.is_file(),f"{info['id']} current published archive")
            if package.is_file():
                digest=hashlib.sha256(package.read_bytes()).hexdigest()
                self.assertEqual(verification['sha256'],digest,f"{info['id']} verified package hash")
            self.assertIn(info['version'],(previous['version'],previous['version']+1))
            self.assertEqual(info['name'],previous['name'].replace(' (LUC1D)',' [PN]'))
            for key in set(previous)-{'name','version','languages'}: self.assertEqual(info.get(key),previous[key],key)
            # A new source version may add site languages while retaining all
            # previously advertised locales. Existing installs keep their
            # immutable package metadata and checksum.
            self.assertTrue(set(previous['languages']) <= set(info['languages']))
        for info in infos:
            self.assertTrue(info['name'].endswith(' [PN]'),info['name'])
        self.assertEqual({i['id'] for i in infos},{'en.luc1d-asurascans','en.luc1d-weebcentral','multi.luc1d-nhentai','multi.luc1d-webtoon','multi.luc1d-imhentai','multi.luc1d-hentaifox'})
        self.assertEqual(len(pipeline.validate_manifest(rows,release=True)),6)
        for row in rows:
            self.assertIsInstance(row['runtime_tested'], bool)
            self.assertIsInstance(row['device_tested'], bool)
            if row['device_tested']:
                self.assertTrue(row.get('device_test_evidence'), row['id'])
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
