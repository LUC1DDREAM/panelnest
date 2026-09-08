import hashlib
from pathlib import Path
import tempfile
import unittest
import zipfile
import pipeline

class ImmutablePackageTests(unittest.TestCase):
    def test_original_archive_reused_for_timestamp_only_rebuild(self):
        baseline=Path(__file__).parent/'published-packages'
        for original in baseline.glob('*.aix'):
            with self.subTest(package=original.name), tempfile.TemporaryDirectory() as d:
                rebuilt=Path(d)/'package.aix'
                with zipfile.ZipFile(original) as src, zipfile.ZipFile(rebuilt,'w',compression=zipfile.ZIP_DEFLATED) as dst:
                    for name in src.namelist():
                        info=zipfile.ZipInfo(name,date_time=(2026,1,1,0,0,0))
                        dst.writestr(info,src.read(name))
                self.assertNotEqual(original.read_bytes(),rebuilt.read_bytes())
                pipeline.preserve_published_package(rebuilt,original.name,baseline)
                self.assertEqual(original.read_bytes(),rebuilt.read_bytes())
        self.assertEqual(len(list(baseline.glob('*.aix'))),6)

    def test_content_change_is_rejected_without_modifying_rebuild(self):
        baseline=Path(__file__).parent/'published-packages'
        original=next(baseline.glob('*.aix'))
        with tempfile.TemporaryDirectory() as d:
            rebuilt=Path(d)/'package.aix'
            with zipfile.ZipFile(original) as src, zipfile.ZipFile(rebuilt,'w') as dst:
                for name in src.namelist():
                    dst.writestr(name,b'changed' if name=='Payload/main.wasm' else src.read(name))
            before=rebuilt.read_bytes()
            with self.assertRaisesRegex(ValueError,'content changed'):
                pipeline.preserve_published_package(rebuilt,original.name,baseline)
            self.assertEqual(before,rebuilt.read_bytes())

    def test_missing_pinned_archive_fails_closed(self):
        import shutil
        baseline=Path(__file__).parent/'published-packages'
        with tempfile.TemporaryDirectory() as d:
            root=Path(d); copy=root/'baseline'; shutil.copytree(baseline,copy)
            original=next(copy.glob('*.aix')); rebuilt=root/'package.aix'
            shutil.copyfile(original,rebuilt); original.unlink()
            with self.assertRaisesRegex(ValueError,'Missing published archive'):
                pipeline.preserve_published_package(rebuilt,original.name,copy)

    def test_new_version_is_allowed(self):
        baseline=Path(__file__).parent/'published-packages'
        original=next(baseline.glob('*.aix'))
        with tempfile.TemporaryDirectory() as d:
            rebuilt=Path(d)/'package.aix'; rebuilt.write_bytes(original.read_bytes())
            pipeline.preserve_published_package(rebuilt,'new-version-v99.aix',baseline)
            self.assertEqual(original.read_bytes(),rebuilt.read_bytes())

    def test_unsafe_duplicate_and_drift_rejected(self):
        baseline=Path(__file__).parent/'published-packages'
        original=next(baseline.glob('*.aix'))
        for extra in ['../escape', '/absolute', 'Payload/extra', 'Payload/source.json']:
            with self.subTest(extra=extra), tempfile.TemporaryDirectory() as d:
                rebuilt=Path(d)/'package.aix'
                with zipfile.ZipFile(original) as src, zipfile.ZipFile(rebuilt,'w') as dst:
                    for name in src.namelist(): dst.writestr(name,src.read(name))
                    dst.writestr(extra,b'changed')
                before=rebuilt.read_bytes()
                with self.assertRaises(ValueError):
                    pipeline.preserve_published_package(rebuilt,original.name,baseline)
                self.assertEqual(before,rebuilt.read_bytes())

    def test_corrupt_or_missing_pin_rejected(self):
        import shutil, json
        baseline=Path(__file__).parent/'published-packages'
        for pins in [{}, {p.name:'0'*64 for p in baseline.glob('*.aix')}]:
            with tempfile.TemporaryDirectory() as d:
                copy=Path(d)/'baseline'; shutil.copytree(baseline,copy)
                (copy/'SHA256.json').write_text(json.dumps(pins))
                original=next(copy.glob('*.aix')); rebuilt=Path(d)/'package.aix'
                rebuilt.write_bytes(original.read_bytes())
                with self.assertRaises(ValueError):
                    pipeline.preserve_published_package(rebuilt,original.name,copy)

if __name__=='__main__': unittest.main()
