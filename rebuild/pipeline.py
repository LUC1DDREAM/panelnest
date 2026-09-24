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
CATALOG_FEATURES = {
    'home', 'listings', 'dynamic-listings', 'dynamic-filters', 'deep-links',
    'image-request', 'alternate-covers', 'page-descriptions', 'web-login', 'migration', 'notifications',
}
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
        if release and not (complete and row.get('publish') and row.get('package_verified') and (row.get('runtime_tested') or row.get('release_authorized') is True)):
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

def enrich_catalog(catalog, rows):
    """Attach verified, user-facing capabilities from the local source manifests."""
    by_id = {row['id']: row for row in rows}
    actual_ids = [item.get('id') for item in catalog.get('sources', [])]
    if len(actual_ids) != len(set(actual_ids)) or set(actual_ids) != set(by_id):
        raise ValueError('Cannot enrich a catalog with missing or duplicate sources')
    for item in catalog['sources']:
        row = by_id[item['id']]
        info = json.loads((ROOT/row['path']/'res/source.json').read_text(encoding='utf-8'))
        manifest = info
        declared = set(row.get('features', []))
        if not FEATURES <= declared:
            raise ValueError(f"Feature metadata incomplete for {item['id']}")
        # Only expose capabilities implemented and manually declared for this build.
        if not declared <= set(row.get('implemented', [])):
            raise ValueError(f"Feature metadata exceeds implementation record for {item['id']}")
        listings = [listing.get('name', listing.get('id', '')) for listing in manifest.get('listings', [])]
        for name in row.get('dynamic_listings', []):
            if name not in listings:
                listings.append(name)
        item['features'] = sorted(declared & (FEATURES | CATALOG_FEATURES))
        item['listings'] = listings
        if row.get('limitations'):
            item['limitations'] = row['limitations']
    return catalog

def preserve_published_package(package, name, baseline=ROOT/'rebuild/published-packages'):
    """Keep published ZIP bytes only when every rebuilt member is identical.

    Aidoku's packager timestamps ZIP entries, so equivalent builds otherwise
    mutate unchanged version URLs. Content changes require a version bump.
    """
    if PurePosixPath(name).name != name or '\\' in name or ':' in name:
        raise ValueError('Invalid published package name')
    with zipfile.ZipFile(package) as archive:
        names = archive.namelist()
        if len(names) != len(set(names)):
            raise ValueError('Duplicate package members')
        for member in names:
            path = PurePosixPath(member)
            if path.is_absolute() or '..' in path.parts or '\\' in member or ':' in member:
                raise ValueError('Unsafe package member')
    original = baseline/name
    hashes = json.loads((baseline/'SHA256.json').read_text())
    if name not in hashes:
        if original.exists():
            raise ValueError('Published archive missing checksum pin')
        return  # A genuinely new version is not pinned yet.
    if not original.is_file():
        raise ValueError('Missing published archive')
    if hashlib.sha256(original.read_bytes()).hexdigest() != hashes.get(name):
        raise ValueError('Published archive checksum mismatch')
    with zipfile.ZipFile(original) as old, zipfile.ZipFile(package) as new:
        old_names, new_names = old.namelist(), new.namelist()
        if len(old_names) != len(set(old_names)) or len(new_names) != len(set(new_names)) or set(old_names) != set(new_names):
            raise ValueError('Published package member set changed; bump version')
        if any(old.read(member) != new.read(member) for member in old_names):
            raise ValueError('Published package content changed; bump version')
    shutil.copyfile(original, package)


def retain_published_assets(out, baseline=ROOT/'rebuild/published-packages'):
    """Keep old installed version URLs working when a new version is published."""
    hashes = json.loads((baseline/'SHA256.json').read_text())
    for name, digest in hashes.items():
        if PurePosixPath(name).name != name or not name.endswith('.aix') or '\\' in name or ':' in name:
            raise ValueError('Invalid published package name')
        original = baseline/name
        if not original.is_file() or hashlib.sha256(original.read_bytes()).hexdigest() != digest:
            raise ValueError('Missing or corrupt published archive')
        with zipfile.ZipFile(original) as archive:
            assets = {'sources/'+name: original.read_bytes(),
                      'icons/'+name[:-4]+'.png': archive.read('Payload/icon.png')}
        for relative, data in assets.items():
            target = out/relative
            if target.exists() and target.read_bytes() != data:
                raise ValueError('Versioned asset changed: '+relative)
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)


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
        preserve_published_package(package, f"{info['id']}-v{info['version']}.aix")
        source_infos[row['id']]=source_info
        packages.append(str(package))
        results.append(dict(id=row['id'],version=info['version'],sha256=hashlib.sha256(package.read_bytes()).hexdigest(),package_verified=True,runtime_tested=row.get('runtime_tested',False),device_tested=row.get('device_tested',False),release_authorized=row.get('release_authorized',False)))
    run('aidoku','build','-o',str(out),'-n','PanelNest'+('' if args.release else ' — staging'),*packages)
    index=json.loads((out/'index.json').read_text())
    index = enrich_catalog(index, selected)
    (out/'index.json').write_text(json.dumps(index, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
    (out/'index.min.json').write_text(json.dumps(index, ensure_ascii=False, separators=(',', ':'))+'\n', encoding='utf-8')
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
    retain_published_assets(out)
    (out/'.nojekyll').touch()
    (out/'build-report.json').write_text(json.dumps(dict(release=args.release,requested=6,built=len(results),sources=results),indent=2)+'\n')
    checksums=[]
    for p in sorted(out.rglob('*')):
        if p.is_file(): checksums.append(f'{hashlib.sha256(p.read_bytes()).hexdigest()}  {p.relative_to(out).as_posix()}')
    (out/'CHECKSUMS.sha256').write_text('\n'.join(checksums)+'\n')
    print(json.dumps(dict(requested=6,built=len(actual),release=args.release,ids=actual)))

if __name__=='__main__': main()
