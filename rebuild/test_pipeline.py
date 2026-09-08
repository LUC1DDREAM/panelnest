"""Publication contract tests. Fixtures are synthetic, never remote packages."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
import zipfile

spec = importlib.util.spec_from_file_location('pipeline', Path(__file__).with_name('pipeline.py'))
pipeline = importlib.util.module_from_spec(spec)
spec.loader.exec_module(pipeline)

class PipelineTests(unittest.TestCase):
    def rows(self):
        return [dict(id=f'en.luc1d-{s}', path=f'rust/en.luc1d-{s}', implemented=['search','details','chapters','pages'], package_verified=True, runtime_tested=True, publish=True) for s in ['asurascans','weebcentral','nhentai','webtoon','imhentai','hentaifox']]

    def test_requires_exactly_six_unique_sources(self):
        rows=self.rows()
        self.assertEqual(len(pipeline.validate_manifest(rows, release=True)),6)
        with self.assertRaises(ValueError): pipeline.validate_manifest(rows[:-1], release=True)
        rows[-1]['id']=rows[0]['id']
        with self.assertRaises(ValueError): pipeline.validate_manifest(rows, release=True)

    def test_pending_source_blocks_release_but_not_staging(self):
        rows=self.rows(); rows[-1].update(implemented=[], publish=False, runtime_tested=False)
        self.assertEqual(len(pipeline.validate_manifest(rows, release=False)),5)
        with self.assertRaises(ValueError): pipeline.validate_manifest(rows, release=True)

    def test_unverified_or_unimplemented_source_blocks_release(self):
        for field, value in [('package_verified',False),('runtime_tested',False),('implemented',['search'])]:
            rows=self.rows(); rows[0][field]=value
            with self.assertRaises(ValueError): pipeline.validate_manifest(rows, release=True)

    def test_remote_forwarding_and_path_escape_rejected(self):
        for path in ['https://example.org/source.aix','../other-repo','C:/other-repo']:
            rows=self.rows(); rows[0]['path']=path
            with self.assertRaises(ValueError): pipeline.validate_manifest(rows, release=False)

    def test_catalog_metadata_matches_package_and_source(self):
        info = dict(id='multi.luc1d-imhentai', name='IMHentai (LUC1D)',
                    version=2, languages=['multi'], contentRating=2)
        pipeline.validate_catalog_metadata(info, info)
        for field, value in [('languages', ['All']), ('version', 1), ('contentRating', 0),
                             ('id', 'different.id'), ('name', 'wrong')]:
            with self.subTest(field=field), self.assertRaises(ValueError):
                pipeline.validate_catalog_metadata(dict(info, **{field: value}), info)

    def test_package_rejects_non_square_or_transparent_icons(self):
        import io
        from PIL import Image
        with tempfile.TemporaryDirectory() as td:
            p = Path(td)/'package.aix'
            for size, color in [((64, 128), (0, 0, 0, 255)), ((128, 128), (0, 0, 0, 0))]:
                icon = io.BytesIO()
                Image.new('RGBA', size, color).save(icon, format='PNG')
                with zipfile.ZipFile(p, 'w') as z:
                    z.writestr('Payload/source.json', json.dumps({'info': {'id': 'test'}}))
                    z.writestr('Payload/main.wasm', b'\0asm\x01\0\0\0')
                    z.writestr('Payload/icon.png', icon.getvalue())
                with self.subTest(size=size, color=color), self.assertRaises(ValueError):
                    pipeline.inspect_package(p, 'test')

    def test_all_six_source_icons_match_provenance(self):
        import hashlib
        from PIL import Image
        records = json.loads((pipeline.ROOT/'rebuild/icon-provenance.json').read_text())
        rows = json.loads((pipeline.ROOT/'rebuild/sources.json').read_text())
        self.assertEqual(len(records), 6)
        self.assertEqual({r['id'] for r in records}, {r['id'] for r in rows})
        for row in rows:
            record = next(r for r in records if r['id'] == row['id'])
            icon = pipeline.ROOT/row['path']/'res/icon.png'
            info = json.loads(icon.with_name('source.json').read_text())['info']
            self.assertEqual(info['version'], record['version'])
            self.assertEqual(hashlib.sha256(icon.read_bytes()).hexdigest(), record['result_sha256'])
            with Image.open(icon) as image:
                self.assertEqual(image.format, 'PNG')
                self.assertEqual(image.size, (128, 128))
                self.assertEqual(image.convert('RGBA').getchannel('A').getextrema(), (255, 255))
            if row['id'] in ('multi.luc1d-imhentai', 'multi.luc1d-hentaifox'):
                self.assertEqual(info['languages'], ['multi'])
                self.assertEqual(info['contentRating'], 2)

    def test_package_identity_and_wasm_validation(self):
        with tempfile.TemporaryDirectory() as td:
            p=Path(td)/'package.aix'
            def package(identifier, wasm):
                with zipfile.ZipFile(p,'w') as z:
                    z.writestr('Payload/source.json',json.dumps({'info':{'id':identifier,'version':1}}))
                    z.writestr('Payload/main.wasm',wasm)
                    import io
                    from PIL import Image
                    icon = io.BytesIO()
                    Image.new('RGB', (128, 128), 'white').save(icon, format='PNG')
                    z.writestr('Payload/icon.png',icon.getvalue())
            package('en.luc1d-asurascans',b'\0asm\x01\0\0\0')
            self.assertEqual(pipeline.inspect_package(p,'en.luc1d-asurascans')['version'],1)
            with self.assertRaises(ValueError): pipeline.inspect_package(p,'wrong.id')
            package('en.luc1d-asurascans',b'not wasm')
            with self.assertRaises(ValueError): pipeline.inspect_package(p,'en.luc1d-asurascans')

class LanguageFilterTests(unittest.TestCase):
    # Aidoku v0.9 AddSourceView.filterExternalSources: exact membership, not
    # SourceLanguage.primaryCode/display normalization. See language-filter.md.
    @staticmethod
    def visible(info, selected):
        return any(code in info['languages'] if info.get('languages') is not None
                   else info.get('lang') == code for code in selected)

    def test_all_six_visible_with_multilingual_and_english(self):
        rows = json.loads((pipeline.ROOT/'rebuild/sources.json').read_text())
        infos = [json.loads((pipeline.ROOT/row['path']/'res/source.json').read_text())['info']
                 for row in rows]
        visible = [info['id'] for info in infos if self.visible(info, {'multi', 'en'})]
        self.assertEqual(set(visible), {row['id'] for row in rows})
        self.assertEqual(len(visible), 6)

    def test_exact_language_membership_controls(self):
        self.assertFalse(self.visible({'languages': ['All']}, {'multi', 'en'}))
        self.assertTrue(self.visible({'languages': ['multi']}, {'multi'}))
        self.assertFalse(self.visible({'languages': ['multi']}, {'en'}))
        self.assertFalse(self.visible({'languages': ['MULTI']}, {'multi'}))
        self.assertTrue(self.visible({'languages': ['en']}, {'en'}))
        self.assertTrue(self.visible({'lang': 'en'}, {'en'}))
        self.assertFalse(self.visible({'languages': [], 'lang': 'en'}, {'en'}))
        self.assertFalse(self.visible({'languages': ['en', 'ja']}, {'multi'}))

class SiteTests(unittest.TestCase):
    def test_experimental_site_has_empty_supported_catalog(self):
        import site_output
        with tempfile.TemporaryDirectory() as td:
            root=Path(td)
            (root/'index.json').write_text(json.dumps({'name':'STAGING','sources':[{'id':'test'}]}))
            (root/'index.min.json').write_text('{}')
            (root/'build-report.json').write_text('{"release":false}')
            site_output.prepare(root)
            self.assertEqual(json.loads((root/'index.json').read_text())['sources'],[])
            self.assertEqual(json.loads((root/'experimental/index.json').read_text())['sources'],[{'id':'test'}])
            html = (root/'index.html').read_text()
            self.assertIn('formal device verification is still pending', html)
            self.assertIn('https://aidoku.app/add-source-list/?url=https%3A%2F%2Fluc1ddream.github.io%2Fmy-aidoku-sources%2Fexperimental%2Findex.min.json', html)
            self.assertIn('https://luc1ddream.github.io/my-aidoku-sources/experimental/index.min.json</code>', html)
            self.assertIn('Add to Aidoku', html)

if __name__=='__main__': unittest.main()
