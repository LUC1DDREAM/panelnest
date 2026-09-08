"""Build only local Rust; fail closed on incomplete release or missing artifacts."""
import argparse
import hashlib
import io
import json
from PIL import Image
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import zipfile

ROOT = Path(__file__).resolve().parents[1]
FEATURES = {'search', 'details', 'chapters', 'pages'}
SLUGS = {'asurascans', 'weebcentral', 'nhentai', 'webtoon', 'imhentai', 'hentaifox'}

def validate_manifest(rows, release=False):
    ids = [r['id'] for r in rows]
    if len(ids) != 6 or len(set(ids)) != 6:
        raise ValueError('Manifest must track exactly six unique requested sources')
    if {s.split('luc1d-')[-1] for s in ids} != SLUGS:
        raise ValueError('Unexpected source set')
    selected = []
    for row in rows:
        if not re.fullmatch(r'(en|multi)\.luc1d-[a-z]+',row['id']):
            raise ValueError('Invalid independent source ID')
        path = row.get('path')
        if path is not None and (not path.startswith('rust/') or '..' in PurePosixPath(path).parts or ':' in path or '\\' in path):
            raise ValueError('Only repository-local rust/ paths are accepted')
        complete = FEATURES <= set(row.get('implemented',[])) and path is not None
        if release and not (complete and row.get('publish') and row.get('package_verified') and row.get('runtime_tested')):
            raise ValueError(f"Release blocked: {row['id']} is not verified and approved")
        if complete:
            selected.append(row)
    return selected

def inspect_package(path, expected_id):
    with zipfile.ZipFile(path) as archive:
        info = json.loads(archive.read('Payload/source.json'))['info']
        if info['id'] != expected_id:
            raise ValueError('Package ID mismatch')
        if not archive.read('Payload/main.wasm').startswith(b'\0asm\x01\0\0\0'):
            raise ValueError('Invalid wasm header')
        with Image.open(io.BytesIO(archive.read('Payload/icon.png'))) as icon:
            if icon.format != 'PNG' or icon.size != (128, 128):
                raise ValueError('Icon must be 128x128 PNG')
            if icon.convert('RGBA').getchannel('A').getextrema() != (255, 255):
                raise ValueError('Icon must be fully opaque')
        return info

def validate_catalog_metadata(actual, expected):
    for field in ('id', 'name', 'version', 'languages', 'contentRating'):
        if actual.get(field) != expected.get(field):
            raise ValueError(f'Catalog/package/source {field} mismatch for {expected["id"]}')

def run(*args, cwd=ROOT):
    subprocess.run(args, cwd=cwd, check=True)

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--release',action='store_true',help='Require all six verified sources; default is non-deployable staging')
    args = parser.parse_args()
    rows = json.loads((ROOT/'rebuild/sources.json').read_text())
    selected = validate_manifest(rows, args.release)
    if not selected:
        raise ValueError('No implemented sources')
    out = ROOT/'dist'
    if out.exists(): shutil.rmtree(out)
    packages=[]
    results=[]
    source_infos={}
    for row in selected:
        directory = ROOT/row['path']
        # The CLI takes the first WASM in target/release. Isolate per source,
        # and discard stale WASM/Payload when a crate name or resources change.
        release_dir = directory/'target/wasm32-unknown-unknown/release'
        if (release_dir/'Payload').exists(): shutil.rmtree(release_dir/'Payload')
        for old in release_dir.glob('*.wasm'): old.unlink()
        package=directory/'package.aix'
        package.unlink(missing_ok=True)
        run('cargo','test','--locked',cwd=directory)
        run('cargo','build','--release','--locked','--target','wasm32-unknown-unknown',cwd=directory)
        run('aidoku','package',str(directory))
        run('aidoku','verify',str(package))
        info=inspect_package(package,row['id'])
        source_info=json.loads((directory/'res/source.json').read_text())['info']
        validate_catalog_metadata(info, source_info)
        source_infos[row['id']]=source_info
        packages.append(str(package))
        results.append(dict(id=row['id'],version=info['version'],sha256=hashlib.sha256(package.read_bytes()).hexdigest(),package_verified=True,runtime_tested=row.get('runtime_tested',False)))
    run('aidoku','build','-o',str(out),'-n','LUC1D Independent Sources'+('' if args.release else ' — EXPERIMENTAL (not device-tested)'),*packages)
    index=json.loads((out/'index.json').read_text())
    actual=[r['id'] for r in index['sources']]
    expected={r['id'] for r in selected}
    if len(actual)!=len(expected) or set(actual)!=expected:
        raise ValueError('CLI omitted or duplicated a source')
    for item in index['sources']:
        package=out/item['downloadURL']
        info=inspect_package(package,item['id'])
        validate_catalog_metadata(info, source_infos[item['id']])
        validate_catalog_metadata(item, info)
        icon_bytes = (out/item['iconURL']).read_bytes()
        with zipfile.ZipFile(package) as archive:
            if icon_bytes != archive.read('Payload/icon.png'):
                raise ValueError('Catalog/package icon mismatch')
        source_row = next(row for row in selected if row['id'] == item['id'])
        if icon_bytes != (ROOT/source_row['path']/'res/icon.png').read_bytes():
            raise ValueError('Catalog/source icon mismatch')
    (out/'.nojekyll').touch()
    (out/'build-report.json').write_text(json.dumps(dict(release=args.release,requested=6,built=len(results),sources=results),indent=2)+'\n')
    checksums=[]
    for p in sorted(out.rglob('*')):
        if p.is_file(): checksums.append(f'{hashlib.sha256(p.read_bytes()).hexdigest()}  {p.relative_to(out).as_posix()}')
    (out/'CHECKSUMS.sha256').write_text('\n'.join(checksums)+'\n')
    print(json.dumps(dict(requested=6,built=len(actual),release=args.release,ids=actual)))

if __name__=='__main__': main()
