"""Generate public PanelNest pages and preserve installed legacy aliases."""
import hashlib
from html import escape as esc
import json
import os
from pathlib import Path
import re
import shutil
from urllib.parse import quote, urlparse

HERE = Path(__file__).resolve().parent
LOCALES = json.loads((HERE/'site-locales.json').read_text(encoding='utf-8'))
DEFAULT_URL = 'https://luc1ddream.github.io/panelnest/'
FEATURE_LABELS = ('search', 'details', 'chapters', 'pages', 'home', 'listings',
                  'dynamic-listings', 'dynamic-filters', 'deep-links', 'image-request',
                  'alternate-covers', 'page-descriptions', 'web-login', 'migration', 'notifications')


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
        locale_route = '' if lang == 'en' else lang+'/'
        locale_path = prefix+locale_route
        languages = ''.join(f'<a href="{prefix if code == "en" else prefix+code+"/"}" lang="{code}" hreflang="{code}" data-language="{code}"'+(' aria-current="page"' if code==lang else '')+f'>{esc(data["native"])}</a>' for code,data in LOCALES.items())
        guide_path = locale_path+'guides/add-aidoku-source-list/'
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
            implemented = set(item.get('features', ()))
            feature_labels = [t['feature_' + key] for key in FEATURE_LABELS if key in implemented]
            feature_text = ' · '.join(feature_labels)
            listings = item.get('listings', [])
            listing_text = ', '.join(esc(str(value)) for value in listings)
            capabilities = (f'<p class="capabilities">{esc(feature_text)}</p>' if feature_text else '')
            discovery = (f'<p class="discovery"><strong>{t["discovery"]}:</strong> {listing_text}</p>'
                         if listing_text else '')
            limitations = item.get('limitations')
            if limitations and limitations is True:
                limitations = t['limitation']
            limitation_text = (f'<p class="limitations">{esc(str(limitations))}</p>'
                               if limitations else '')
            source_path = locale_path+'sources/'+quote(str(item['id']), safe='-._')+'/'
            cards.append(
                f'<article class="source-card" id="source-{esc(item["id"])}">'
                f'<div class="source-top">{image}<div class="source-title"><h3><a href="{source_path}">{name}</a></h3>'
                f'<p class="meta">{labels}</p></div></div>'
                f'<div class="source-bottom"><span class="version">{t["version"]} {esc(str(item.get("version", "")))}</span>{adult}</div>'
                f'<details class="source-details"><summary>{t["card_details"]}</summary>'
                f'<div class="source-detail-content">{capabilities}{discovery}{limitation_text}</div></details></article>'
            )
        steps = ''.join(f'<li><h3>{t[f"step{i}"]}</h3><p>{t[f"body{i}"]}</p>'+('<a href="https://aidoku.app/">aidoku.app ↗</a>' if i==1 else '')+'</li>' for i in range(1,4))
        faq = ''.join(f'<details><summary>{t[f"q{i}"]}</summary><p>{t[f"a{i}"]}</p></details>' for i in range(1,5))
        schema = json.dumps({'@context':'https://schema.org','@graph':[
            {'@type':'WebSite','@id':base+'#website','name':'PanelNest','url':base,'inLanguage':list(LOCALES)},
            {'@type':'CollectionPage','@id':canonical+'#webpage','url':canonical,
             'name':LOCALES[lang]['title'],'description':LOCALES[lang]['description'],
             'inLanguage':lang,'isPartOf':{'@id':base+'#website'},
             'mainEntity':{'@type':'ItemList','numberOfItems':len(sources),'itemListElement':[
                 {'@type':'ListItem','position':i,'name':s.get('name',s['id']),
                  'url':canonical+'sources/'+quote(str(s['id']), safe='-._')+'/'}
                 for i,s in enumerate(sources,1)]}}
        ]}, ensure_ascii=False).replace('<','\\u003c')
        html = f'''<!doctype html>
<html lang="{lang}" data-theme="dark"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="google-site-verification" content="WWwPJxNyRoW9NBZ3JSonNBXYxp76m_qN3I87ieOerE8" />
<title>{t['title']}</title><meta name="description" content="{t['description']}"><meta name="theme-color" content="#0d1423">
<link rel="canonical" href="{canonical}">{alternates}
<meta property="og:type" content="website"><meta property="og:site_name" content="PanelNest"><meta property="og:title" content="{t['title']}"><meta property="og:description" content="{t['description']}"><meta property="og:url" content="{canonical}"><meta property="og:image" content="{base}assets/social.png"><meta property="og:image:width" content="1200"><meta property="og:image:height" content="630"><meta property="og:image:alt" content="PanelNest">
<meta name="twitter:card" content="summary_large_image"><meta name="twitter:title" content="{t['title']}"><meta name="twitter:description" content="{t['description']}"><meta name="twitter:image" content="{base}assets/social.png">
<link rel="icon" href="{assets}mark.svg" type="image/svg+xml"><script src="{assets}theme.js"></script><link rel="stylesheet" href="{assets}site.css"><script src="{assets}site.js" defer></script><script type="application/ld+json">{schema}</script></head>
<body data-base="{prefix}" data-auto-language="{str(route in ('','experimental/')).lower()}"><a class="skip" href="#main">{t['skip']}</a>
<header class="wrap"><a class="brand" href="{locale_path}"><img src="{assets}mark.svg" width="40" height="40" alt="">PanelNest<span class="brand-dot" aria-hidden="true">.</span></a><nav aria-label="{t['sources']}"><a href="#sources">{t['sources']}</a><a href="#setup">{t['guide']}</a><a href="#help">{t['help']}</a></nav><details class="languages"><summary>{t['native']} <span aria-hidden="true">⌄</span></summary><nav aria-label="{t['language']}">{languages}</nav></details><button id="theme-toggle" class="theme-toggle" data-light="{t['light_mode']}" data-dark="{t['dark_mode']}" hidden>{t['light_mode']}</button></header>
<main id="main" class="wrap"><section class="hero" aria-labelledby="headline"><div><p class="eyebrow">{t['eyebrow']}</p><h1 id="headline">{t['headline'].replace(chr(10),'<br>')}</h1><p class="intro">{t['intro']}</p><div class="actions"><a class="button primary" id="import-list" href="{import_url}">{t['add']} <span aria-hidden="true">↗</span></a><a class="button secondary" href="#setup">{t['guide']} <span aria-hidden="true">↓</span></a></div><p class="prereq">{t['prereq']}</p></div><aside class="list-note"><img class="hero-mark" src="{assets}mark.svg" alt="" width="104" height="104"><p class="eyebrow">PanelNest / Aidoku</p><h2>{t['list_label']}</h2><p>{t['notice']}</p><a href="{prefix}experimental/build-report.json">{t['evidence']} ↗</a></aside></section>
<section id="sources" class="catalog"><div class="section-heading"><h2>{t['catalog']} <span class="count">{len(sources):02d}</span></h2><p>{t['catalogintro']}</p></div><div class="source-grid">{''.join(cards) or '<p>'+t['empty']+'</p>'}</div></section>
<section id="setup" class="setup"><div class="section-heading"><h2>{t['guide']}</h2><span class="eyebrow">iPhone &amp; iPad</span></div><ol class="steps">{steps}</ol><div class="manual"><div><h3>{t['manual']}</h3><code id="list-url" tabindex="0">{list_url}</code></div><button class="button secondary" id="copy-url" data-success="{t['copied']}" data-failure="{t['copyfail']}" hidden>{t['copy']}</button><p id="copy-status" role="status" aria-live="polite"></p></div><p class="subpage-link"><a href="{guide_path}">{t['source_guide']} →</a></p></section>
<section id="faq" class="faq" aria-labelledby="faq-title"><h2 id="faq-title">{t["faq_title"]}</h2>{faq}</section>
<section id="help" class="help"><div><h2>{t['help']}</h2><p>{t['helptext']}</p></div><div class="help-links"><a href="https://github.com/{repository}/issues/new?template=bug-report.yml">{t['report']} <span aria-hidden="true">↗</span></a><a href="https://github.com/{repository}/issues/new?template=source-request.yml">{t['request']} <span aria-hidden="true">↗</span></a></div></section></main>
<footer class="wrap"><a class="brand" href="{locale_path}">PanelNest.</a><div><p>{t['footer']}</p><p>{t['privacy']}</p><a href="{guide_path}">{t['source_guide']}</a> · <a href="https://github.com/{repository}">GitHub ↗</a></div></footer></body></html>'''
        destination = root/route/'index.html'
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(html, encoding='utf-8')
    def render_detail(route, lang, canonical, title, description, crumbs, content, schema):
        t = {k: esc(v) for k, v in LOCALES[lang].items()}
        locale_route = '' if lang == 'en' else lang+'/'
        locale_path = prefix+locale_route
        relative_route = route[len(locale_route):]
        relative_url_route = relative_route.rstrip('/')+'/'
        alternates = ''.join(
            f'<link rel="alternate" hreflang="{code}" href="{base if code == "en" else base+code+"/"}{relative_url_route}">'
            for code in LOCALES
        )+f'<link rel="alternate" hreflang="x-default" href="{base+relative_url_route}">'
        languages = ''.join(
            f'<a href="{prefix if code == "en" else prefix+code+"/"}{relative_url_route}" lang="{code}" hreflang="{code}" data-language="{code}"'
            +(' aria-current="page"' if code==lang else '')+f'>{esc(data["native"])}</a>'
            for code,data in LOCALES.items()
        )
        schema_text = json.dumps(schema, ensure_ascii=False).replace('<','\\u003c')
        breadcrumb_html = ' <span aria-hidden="true">/</span> '.join(
            f'<a href="{href}">{esc(label)}</a>' if href else f'<span aria-current="page">{esc(label)}</span>'
            for label,href in crumbs
        )
        html = f'''<!doctype html>
<html lang="{lang}" data-theme="dark"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="google-site-verification" content="WWwPJxNyRoW9NBZ3JSonNBXYxp76m_qN3I87ieOerE8"><meta name="theme-color" content="#0d1423">
<title>{esc(title)}</title><meta name="description" content="{esc(description)}"><link rel="canonical" href="{canonical}">{alternates}
<meta property="og:type" content="website"><meta property="og:site_name" content="PanelNest"><meta property="og:title" content="{esc(title)}"><meta property="og:description" content="{esc(description)}"><meta property="og:url" content="{canonical}"><meta property="og:image" content="{base}assets/social.png"><meta property="og:image:width" content="1200"><meta property="og:image:height" content="630"><meta property="og:image:alt" content="PanelNest">
<meta name="twitter:card" content="summary_large_image"><meta name="twitter:title" content="{esc(title)}"><meta name="twitter:description" content="{esc(description)}"><meta name="twitter:image" content="{base}assets/social.png">
<link rel="icon" href="{prefix}assets/mark.svg" type="image/svg+xml"><script src="{prefix}assets/theme.js"></script><link rel="stylesheet" href="{prefix}assets/site.css"><script src="{prefix}assets/site.js" defer></script><script type="application/ld+json">{schema_text}</script></head>
<body data-base="{prefix}" data-auto-language="false"><a class="skip" href="#main">{t['skip']}</a>
<header class="wrap"><a class="brand" href="{locale_path}"><img src="{prefix}assets/mark.svg" width="40" height="40" alt="">PanelNest<span class="brand-dot" aria-hidden="true">.</span></a><nav aria-label="{t['sources']}"><a href="{locale_path}#sources">{t['sources']}</a><a href="{locale_path}#setup">{t['guide']}</a><a href="{locale_path}#help">{t['help']}</a></nav><details class="languages"><summary>{t['native']} <span aria-hidden="true">⌄</span></summary><nav aria-label="{t['language']}">{languages}</nav></details><button id="theme-toggle" class="theme-toggle" data-light="{t['light_mode']}" data-dark="{t['dark_mode']}" hidden>{t['light_mode']}</button></header>
<main id="main" class="wrap"><nav class="breadcrumbs" aria-label="Breadcrumb">{breadcrumb_html}</nav><article class="seo-page">{content}</article></main>
<footer class="wrap"><a class="brand" href="{locale_path}">PanelNest.</a><div><p>{t['footer']}</p><p>{t['privacy']}</p><a href="{locale_path}guides/add-aidoku-source-list/">{t['source_guide']}</a> · <a href="https://github.com/{repository}">GitHub ↗</a></div></footer></body></html>'''
        destination = root/route/'index.html'
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(html, encoding='utf-8')

    for lang in LOCALES:
        locale_path = '' if lang == 'en' else lang+'/'
        locale_url = base+locale_path
        guide_route = locale_path+'guides/add-aidoku-source-list'
        guide_url = locale_url+'guides/add-aidoku-source-list/'
        t = {k: esc(v) for k, v in LOCALES[lang].items()}
        manual_url = base+'index.min.json'
        import_url = 'https://aidoku.app/add-source-list/?url='+quote(manual_url, safe='')
        guide_title = LOCALES[lang]['guide_title']
        guide_description = LOCALES[lang]['guide_description']
        guide_content = f'''<p class="eyebrow">PanelNest / Aidoku</p><h1>{t['guide_headline']}</h1><p class="intro">{t['guide_intro']}</p><p class="guide-note">{t['guide_requirement']}</p>
<ol class="steps"><li><h2>{t['step1']}</h2><p>{t['body1']}</p><a href="https://aidoku.app/">aidoku.app ↗</a></li><li><h2>{t['step2']}</h2><p>{t['body2']}</p><a class="button primary" href="{import_url}">{t['add']} ↗</a></li><li><h2>{t['step3']}</h2><p>{t['body3']}</p></li></ol>
<section class="guide-section"><h2>{t['guide_updates_title']}</h2><p>{t['guide_updates']}</p></section><section class="guide-section"><h2>{t['guide_troubleshooting']}</h2><details><summary>{t['guide_filter_title']}</summary><p>{t['guide_filter']}</p></details><details><summary>{t['guide_add_title']}</summary><p>{t['guide_add_help']}</p></details><details><summary>{t['guide_site_title']}</summary><p>{t['guide_site']}</p></details><details><summary>{t['guide_legacy_title']}</summary><p>{t['guide_legacy']}</p></details></section>
<div class="manual"><div><h2>{t['manual']}</h2><code>{esc(manual_url)}</code></div><a class="button secondary" href="{prefix}{locale_path}#sources">{t['guide_catalog']}</a></div>'''
        guide_schema = {'@context':'https://schema.org','@graph':[
            {'@type':'WebSite','@id':base+'#website','name':'PanelNest','url':base,'inLanguage':list(LOCALES)},
            {'@type':'WebPage','@id':guide_url+'#webpage','url':guide_url,'name':guide_title,
             'description':guide_description,'inLanguage':lang,'isPartOf':{'@id':base+'#website'}},
            {'@type':'BreadcrumbList','itemListElement':[
                {'@type':'ListItem','position':1,'name':'PanelNest','item':locale_url},
                {'@type':'ListItem','position':2,'name':guide_title,'item':guide_url}]}
        ]}
        render_detail(guide_route,lang,guide_url,guide_title,guide_description,
                      [(t['sources'],prefix+locale_path+'#sources'),(guide_title,None)],guide_content,guide_schema)
        for item in sources:
            source_id = str(item['id'])
            source_slug = quote(source_id, safe='-._')
            source_route = locale_path+'sources/'+source_slug
            source_url = locale_url+'sources/'+source_slug+'/'
            source_name = str(item.get('name',source_id))
            display_name = re.sub(r'\s*\[PN\]$', '', source_name)
            source_name_html = esc(source_name)
            source_name_plain = esc(display_name)
            site_url = str(item.get('baseURL',''))
            parsed_site = urlparse(site_url)
            site_name = parsed_site.netloc or site_url
            feeds = [str(value) for value in item.get('listings',[])]
            feed_summary = ', '.join(feeds[:3]) if feeds else LOCALES[lang]['catalog']
            source_title = LOCALES[lang]['source_title'].format(name=display_name)
            source_heading = esc(LOCALES[lang]['source_heading'].format(name=display_name))
            source_eyebrow = t['source_eyebrow']
            source_description = LOCALES[lang]['source_description'].format(name=display_name,feeds=feed_summary)
            features = [t['feature_'+key] for key in FEATURE_LABELS if key in set(item.get('features',()))]
            feature_html = ''.join(f'<li>{label}</li>' for label in features)
            feed_html = ''.join(f'<li>{esc(feed)}</li>' for feed in feeds)
            language_labels = [t['english'] if code=='en' else t['multi'] if code=='multi' else esc(code)
                               for code in item.get('languages',[])]
            language_text = ' · '.join(language_labels)
            limitations = item.get('limitations')
            if limitations is True:
                limitations = t['limitation']
            notes = f'<section class="guide-section"><h2>{t["source_notes"]}</h2><p>{esc(str(limitations))}</p></section>' if limitations else ''
            website_link = ''
            if parsed_site.scheme == 'https' and parsed_site.netloc:
                website_link = f'<a class="button secondary" href="{esc(site_url)}">{t["source_website"].format(name=source_name_html)} ↗</a>'
            repo_url = f'https://github.com/{repository}/tree/main/rust/{source_slug}'
            source_content = f'''<p class="eyebrow">{source_eyebrow}</p><h1>{source_heading}</h1><p class="intro">{esc(LOCALES[lang]['source_intro'].format(name=display_name,site=site_name))}</p>
<div class="actions"><a class="button primary" href="{import_url}">{t['source_add']} ↗</a><a class="button secondary" href="{prefix}{locale_path}guides/add-aidoku-source-list/">{t['source_guide']}</a>{website_link}</div>
<div class="detail-facts"><p><strong>{t['source_version']}:</strong> {esc(str(item.get('version','')))}</p><p><strong>{t['source_languages']}:</strong> {language_text}</p></div>
<section class="guide-section"><h2>{t['source_features']}</h2><ul class="detail-list">{feature_html}</ul></section>
<section class="guide-section"><h2>{t['source_feeds']}</h2><ul class="detail-list">{feed_html}</ul></section>{notes}
<p class="subpage-link"><a href="{esc(repo_url)}">{t['source_code']} ↗</a> · <a href="{prefix}{locale_path}#sources">{t['source_all']}</a></p>'''
            source_schema = {'@context':'https://schema.org','@graph':[
                {'@type':'WebSite','@id':base+'#website','name':'PanelNest','url':base,'inLanguage':list(LOCALES)},
                {'@type':'WebPage','@id':source_url+'#webpage','url':source_url,'name':source_title,
                 'description':source_description,'inLanguage':lang,'isPartOf':{'@id':base+'#website'},
                 'mainEntity':{'@type':'SoftwareSourceCode','name':source_name,'codeRepository':repo_url,
                               'programmingLanguage':'Rust','runtimePlatform':'Aidoku for iOS and iPadOS'}},
                {'@type':'BreadcrumbList','itemListElement':[
                    {'@type':'ListItem','position':1,'name':'PanelNest','item':locale_url},
                    {'@type':'ListItem','position':2,'name':t['sources'],'item':locale_url+'#sources'},
                    {'@type':'ListItem','position':3,'name':source_name,'item':source_url}]}
            ]}
            render_detail(source_route,lang,source_url,source_title,source_description,
                          [(t['sources'],prefix+locale_path+'#sources'),(source_name,None)],
                          source_content,source_schema)
    sitemap_groups = [(base,[base if code=='en' else base+code+'/' for code in LOCALES])]
    sitemap_groups += [(base+'guides/add-aidoku-source-list/',
                        [base+('' if code=='en' else code+'/')+'guides/add-aidoku-source-list/' for code in LOCALES])]
    for item in sources:
        slug=quote(str(item['id']),safe='-._')
        sitemap_groups.append((base+'sources/'+slug+'/',
            [base+('' if code=='en' else code+'/')+'sources/'+slug+'/' for code in LOCALES]))
    sitemap_entries=[]
    for url,localized in sitemap_groups:
        alternates=''.join(f'<xhtml:link rel="alternate" hreflang="{code}" href="{esc(href)}"/>' for code,href in zip(LOCALES,localized))
        alternates+=f'<xhtml:link rel="alternate" hreflang="x-default" href="{esc(localized[0])}"/>'
        sitemap_entries.extend(f'<url><loc>{esc(href)}</loc>{alternates}</url>' for href in localized)
    (root/'sitemap.xml').write_text('<?xml version="1.0" encoding="UTF-8"?><urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">'+''.join(sitemap_entries)+'</urlset>', encoding='utf-8')
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
