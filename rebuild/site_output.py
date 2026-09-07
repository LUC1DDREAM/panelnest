"""Expose staging builds separately; never promote them to supported sources."""
import hashlib
import json
from pathlib import Path
import shutil


def prepare(root):
    report = json.loads((root/'build-report.json').read_text())
    if report.get('release') is not False:
        raise ValueError('Only explicitly experimental builds accepted')
    experimental = root/'experimental'
    if experimental.exists():
        raise ValueError('Already prepared')
    files = list(root.iterdir())
    experimental.mkdir()
    for path in files:
        shutil.move(str(path), str(experimental/path.name))
    supported = {'name':'LUC1D Independent Sources - no device-verified sources yet','sources':[]}
    for name in ('index.json','index.min.json'):
        (root/name).write_text(json.dumps(supported)+'\n', encoding='utf-8')
    (root/'.nojekyll').touch()
    (root/'index.html').write_text('''<!doctype html><html lang="en"><meta charset="utf-8"><title>LUC1D Aidoku sources</title>
<h1>Independent Aidoku sources: experimental</h1>
<p><a href="https://aidoku.app/add-source-list/?url=https://luc1ddream.github.io/my-aidoku-sources/experimental/">Add experimental list to Aidoku</a></p>
<p>Manual list URL: <code>https://luc1ddream.github.io/my-aidoku-sources/experimental/</code></p>
<p>The supported catalog is empty. All six packages are not device-tested and are not advertised as release-ready.</p>
<p><a href="experimental/index.json">Opt-in experimental catalog</a> | <a href="experimental/build-report.json">Build evidence and hashes</a> | <a href="https://github.com/LUC1DDREAM/my-aidoku-sources#verification">Feature matrix and limitations</a></p>
<p>Requires Aidoku 0.7.1 or newer. Independent IDs do not update old sources automatically; retain a library backup before migrating. IMHentai live access returned 403; no bypass was attempted. WEBTOON is English-first, first search page only; Canvas is unverified. No source has an iOS reading/rendering verification.</p></html>''', encoding='utf-8')
    checksums = [f'{hashlib.sha256(p.read_bytes()).hexdigest()}  {p.relative_to(root).as_posix()}' for p in sorted(root.rglob('*')) if p.is_file() and p != root/'CHECKSUMS.sha256']
    (root/'CHECKSUMS.sha256').write_text('\n'.join(checksums)+'\n')


if __name__ == '__main__':
    prepare(Path(__file__).resolve().parents[1]/'dist')
