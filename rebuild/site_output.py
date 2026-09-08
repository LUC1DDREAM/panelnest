"""Generate public PanelNest pages and preserve installed legacy aliases."""
import hashlib
from html import escape as esc
import json
import os
from pathlib import Path
import shutil
from urllib.parse import quote, urlparse

HERE = Path(__file__).resolve().parent
LOCALES = json.loads((HERE/'site-locales.json').read_text(encoding='utf-8'))
DEFAULT_URL = 'https://luc1ddream.github.io/panelnest/'


def render(root, base, repository):
    catalog = json.loads((root/'experimental/index.json').read_text(encoding='utf-8'))
    sources = catalog['sources']
    list_url = base+'index.min.json'
    import_url = 'https://aidoku.app/add-source-list/?url='+quote(list_url, safe='')
    prefix = urlparse(base).path
    assets = prefix+'assets/'
    shutil.copytree(HERE/'site-assets', root/'assets', dirs_exist_ok=True)
    routes = [('', 'en'), ('experimental/', 'en')]+[(lang+'/', lang) for lang in LOCALES]
    for route, lang in routes:
        t = {k: esc(v) for k, v in LOCALES[lang].items()}
        canonical = base if lang == 'en' else base+lang+'/'
        alternates = ''.join(f'<link rel="alternate" hreflang="{code}" href="{base if code == "en" else base+code+"/"}">' for code in LOCALES)
        alternates += f'<link rel="alternate" hreflang="x-default" href="{base}">'
        languages = ''.join(f'<a href="{prefix}{code}/" lang="{code}" hreflang="{code}" data-language="{code}"'+(' aria-current="page"' if code==lang else '')+f'>{esc(data["native"])}</a>' for code,data in LOCALES.items())
        cards = []
        for item in sources:
            name = esc(item.get('name', item['id']))
            icon = item.get('iconURL')
            # Catalog assets must stay local, including in fixtures and future builds.
            if icon and (urlparse(icon).scheme or icon.startswith('/') or '..' in Path(icon).parts):
                raise ValueError('Source icon must be a local catalog asset')
            image = f'<img src="{prefix}experimental/{esc(icon)}" alt="" width="64" height="64" loading="lazy">' if icon else ''
            labels = ' · '.join(t['english'] if code=='en' else t['multi'] if code=='multi' else esc(code) for code in item.get('languages',[]))
            adult = f'<span class="adult">{t["adult"]}</span>' if item.get('contentRating')==2 else ''
            cards.append(f'<article class="source-card" id="source-{esc(item["id"])}">{image}<div><h3>{name}</h3><p class="meta">{labels} <span aria-hidden="true">/</span> {t["version"]} {esc(str(item.get("version","")))}</p></div><p class="capabilities">{t["features"]}</p>{adult}</article>')
        steps = ''.join(f'<li><h3>{t[f"step{i}"]}</h3><p>{t[f"body{i}"]}</p>'+('<a href="https://aidoku.app/">aidoku.app ↗</a>' if i==1 else '')+'</li>' for i in range(1,4))
        faq = ''.join(f'<details><summary>{t[f"q{i}"]}</summary><p>{t[f"a{i}"]}</p></details>' for i in range(1,5))
        schema = json.dumps({'@context':'https://schema.org','@graph':[
            {'@type':'WebSite','@id':base+'#website','name':'PanelNest','url':base,'inLanguage':list(LOCALES)},
            {'@type':'CollectionPage','@id':canonical+'#webpage','url':canonical,
             'name':LOCALES[lang]['title'],'description':LOCALES[lang]['description'],
             'inLanguage':lang,'isPartOf':{'@id':base+'#website'},
             'mainEntity':{'@type':'ItemList','numberOfItems':len(sources),'itemListElement':[
                 {'@type':'ListItem','position':i,'name':s.get('name',s['id']),'url':canonical+'#source-'+s['id']}
                 for i,s in enumerate(sources,1)]}}
        ]}, ensure_ascii=False).replace('<','\\u003c')
        html = f'''<!doctype html>
<html lang="{lang}" data-theme="dark"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="google-site-verification" content="WWwPJxNyRoW9NBZ3JSonNBXYxp76m_qN3I87ieOerE8" />
<title>{t['title']}</title><meta name="description" content="{t['description']}"><meta name="theme-color" content="#101b2a">
<link rel="canonical" href="{canonical}">{alternates}
<meta property="og:type" content="website"><meta property="og:site_name" content="PanelNest"><meta property="og:title" content="{t['title']}"><meta property="og:description" content="{t['description']}"><meta property="og:url" content="{canonical}"><meta property="og:image" content="{base}assets/social.png"><meta property="og:image:width" content="1200"><meta property="og:image:height" content="630"><meta property="og:image:alt" content="PanelNest">
<meta name="twitter:card" content="summary_large_image"><meta name="twitter:title" content="{t['title']}"><meta name="twitter:description" content="{t['description']}"><meta name="twitter:image" content="{base}assets/social.png">
<link rel="icon" href="{assets}mark.svg" type="image/svg+xml"><script src="{assets}theme.js"></script><link rel="stylesheet" href="{assets}site.css"><script src="{assets}site.js" defer></script><script type="application/ld+json">{schema}</script></head>
<body data-base="{prefix}" data-auto-language="{str(route in ('','experimental/')).lower()}"><a class="skip" href="#main">{t['skip']}</a>
<header class="wrap"><a class="brand" href="{prefix}{lang}/"><img src="{assets}mark.svg" width="40" height="40" alt="">PanelNest<span class="brand-dot" aria-hidden="true">.</span></a><nav aria-label="{t['sources']}"><a href="#sources">{t['sources']}</a><a href="#setup">{t['guide']}</a><a href="#help">{t['help']}</a></nav><details class="languages"><summary>{t['native']} <span aria-hidden="true">⌄</span></summary><nav aria-label="{t['language']}">{languages}</nav></details><button id="theme-toggle" class="theme-toggle" data-light="{t['light_mode']}" data-dark="{t['dark_mode']}" hidden>{t['light_mode']}</button></header>
<main id="main" class="wrap"><section class="hero" aria-labelledby="headline"><div><p class="eyebrow">{t['eyebrow']}</p><h1 id="headline">{t['headline'].replace(chr(10),'<br>')}</h1><p class="intro">{t['intro']}</p><div class="actions"><a class="button primary" id="import-list" href="{import_url}">{t['add']} <span aria-hidden="true">↗</span></a><a class="button secondary" href="#setup">{t['guide']} <span aria-hidden="true">↓</span></a></div><p class="prereq">{t['prereq']}</p></div><aside class="list-note"><img class="hero-mark" src="{assets}mark.svg" alt="" width="104" height="104"><p class="eyebrow">PanelNest / Aidoku</p><h2>{t['list_label']}</h2><p>{t['notice']}</p><a href="{prefix}experimental/build-report.json">{t['evidence']} ↗</a></aside></section>
<section id="sources" class="catalog"><div class="section-heading"><h2>{t['catalog']} <span class="count">{len(sources):02d}</span></h2><p>{t['catalogintro']}</p></div><div class="source-grid">{''.join(cards) or '<p>'+t['empty']+'</p>'}</div></section>
<section id="setup" class="setup"><div class="section-heading"><h2>{t['guide']}</h2><span class="eyebrow">iPhone &amp; iPad</span></div><ol class="steps">{steps}</ol><div class="manual"><div><h3>{t['manual']}</h3><code id="list-url" tabindex="0">{list_url}</code></div><button class="button secondary" id="copy-url" data-success="{t['copied']}" data-failure="{t['copyfail']}" hidden>{t['copy']}</button><p id="copy-status" role="status" aria-live="polite"></p></div></section>
<section id="faq" class="faq" aria-labelledby="faq-title"><h2 id="faq-title">{t["faq_title"]}</h2>{faq}</section>
<section id="help" class="help"><div><h2>{t['help']}</h2><p>{t['helptext']}</p></div><div class="help-links"><a href="https://github.com/{repository}/issues/new?template=bug-report.yml">{t['report']} <span aria-hidden="true">↗</span></a><a href="https://github.com/{repository}/issues/new?template=source-request.yml">{t['request']} <span aria-hidden="true">↗</span></a></div></section></main>
<footer class="wrap"><a class="brand" href="{prefix}{lang}/">PanelNest.</a><div><p>{t['footer']}</p><p>{t['privacy']}</p><a href="https://github.com/{repository}">GitHub ↗</a></div></footer></body></html>'''
        destination = root/route/'index.html'
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(html, encoding='utf-8')
    urls = [base]+[base+lang+'/' for lang in LOCALES if lang != 'en']
    (root/'sitemap.xml').write_text('<?xml version="1.0" encoding="UTF-8"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">'+''.join('<url><loc>'+esc(url)+'</loc></url>' for url in urls)+'</urlset>', encoding='utf-8')
    (root/'robots.txt').write_text('User-agent: *\nAllow: /\nSitemap: '+base+'sitemap.xml\n',encoding='utf-8')


def prepare(root, base=None, repository=None):
    base = (base or os.getenv('PANELNEST_SITE_URL', DEFAULT_URL)).rstrip('/')+'/'
    repository = repository or os.getenv('GITHUB_REPOSITORY', 'LUC1DDREAM/panelnest')
    if urlparse(base).scheme != 'https' or not urlparse(base).netloc:
        raise ValueError('Canonical site URL must be HTTPS')
    report = json.loads((root/'build-report.json').read_text())
    if report.get('release') is not True:
        raise ValueError('Only explicitly authorized release builds accepted')
    experimental = root/'experimental'
    if experimental.exists():
        raise ValueError('Already prepared')
    files = list(root.iterdir())
    experimental.mkdir()
    for path in files:
        shutil.move(str(path), str(experimental/path.name))
    catalog = json.loads((experimental/'index.json').read_text(encoding='utf-8'))
    catalog['name'] = 'PanelNest'
    # Identical lists resolve assets at either base; keep installed legacy paths.
    for folder in ('sources', 'icons'):
        if (experimental/folder).exists():
            shutil.copytree(experimental/folder, root/folder)
    for name in ('index.json','index.min.json'):
        data = json.dumps(catalog, ensure_ascii=False)+'\n'
        (root/name).write_text(data, encoding='utf-8')
        (experimental/name).write_text(data, encoding='utf-8')
    (root/'.nojekyll').touch()
    render(root, base, repository)
    checksums = [f'{hashlib.sha256(p.read_bytes()).hexdigest()}  {p.relative_to(root).as_posix()}' for p in sorted(root.rglob('*')) if p.is_file() and p != root/'CHECKSUMS.sha256']
    (root/'CHECKSUMS.sha256').write_text('\n'.join(checksums)+'\n')


if __name__ == '__main__':
    prepare(HERE.parent/'dist')
