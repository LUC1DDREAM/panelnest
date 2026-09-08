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

    def test_package_identity_and_wasm_validation(self):
        with tempfile.TemporaryDirectory() as td:
            p=Path(td)/'package.aix'
            def package(identifier, wasm):
                with zipfile.ZipFile(p,'w') as z:
                    z.writestr('Payload/source.json',json.dumps({'info':{'id':identifier,'version':1}}))
                    z.writestr('Payload/main.wasm',wasm)
                    z.writestr('Payload/icon.png',b'fixture')
            package('en.luc1d-asurascans',b'\0asm\x01\0\0\0')
            self.assertEqual(pipeline.inspect_package(p,'en.luc1d-asurascans')['version'],1)
            with self.assertRaises(ValueError): pipeline.inspect_package(p,'wrong.id')
            package('en.luc1d-asurascans',b'not wasm')
            with self.assertRaises(ValueError): pipeline.inspect_package(p,'en.luc1d-asurascans')

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
            self.assertIn('not device-tested', html)
            self.assertIn('https://aidoku.app/add-source-list/?url=https://luc1ddream.github.io/my-aidoku-sources/experimental/index.min.json', html)
            self.assertIn('<code>https://luc1ddream.github.io/my-aidoku-sources/experimental/index.min.json</code>', html)
            self.assertIn('Add experimental list to Aidoku', html)

if __name__=='__main__': unittest.main()
