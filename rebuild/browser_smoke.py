"""Browser smoke: python rebuild/browser_smoke.py [base URL]. Requires Playwright.
Run against the generated local site or deployed URL; never fakes source responses.
"""
import json
from pathlib import Path
import sys
from urllib.parse import urlparse, parse_qs
from playwright.sync_api import sync_playwright

base = (sys.argv[1] if len(sys.argv)>1 else 'http://127.0.0.1:8765/panelnest/').rstrip('/')+'/'
out = Path('output/playwright'); out.mkdir(parents=True, exist_ok=True)
results=[]
with sync_playwright() as p:
    browser = p.chromium.launch(channel='chrome', headless=True)
    context = browser.new_context(locale='en-US', permissions=['clipboard-read','clipboard-write'])
    page = context.new_page()
    errors=[]; page.on('pageerror', lambda error: errors.append(str(error)))
    for width in [1440,390,320]:
        page.set_viewport_size({'width':width,'height':900})
        for lang in ['en','de','es','fr','pt']:
            response=page.goto(base+lang+'/'); assert response.status==200
            assert page.locator('html').get_attribute('lang')==lang
            assert page.locator('h1').count()==1
            assert page.locator('.source-card').count()==6
            assert page.locator('link[rel=alternate]').count()==6
            assert page.locator('meta[name=description]').get_attribute('content')
            assert page.title().startswith('PanelNest')
            assert page.evaluate('document.documentElement.scrollWidth <= innerWidth'), (lang,width,'overflow')
            # Lazy images need to enter the viewport before load is expected.
            for image in page.locator('img').all():
                image.scroll_into_view_if_needed()
                image.evaluate('(img)=>img.decode()')
            assert page.locator('img').evaluate_all('(imgs)=>imgs.every(i=>i.complete && i.naturalWidth>0)')
            page.evaluate('window.scrollTo(0, 0)')
            href=page.locator('#import-list').get_attribute('href')
            assert urlparse(href).netloc=='aidoku.app'
            list_url=parse_qs(urlparse(href).query)['url'][0]
            assert list_url.endswith('/panelnest/index.min.json')
            assert page.locator('link[rel=canonical]').get_attribute('href').endswith('/panelnest/' if lang == 'en' else '/'+lang+'/')
            results.append({'lang':lang,'width':width,'title':page.title(),'import':href,'overflow':False})
            if width in [1440,390] and lang in ['en','de']:
                page.screenshot(path=str(out/f'{urlparse(base).hostname}-{lang}-{width}.png'),full_page=True)
    page.goto(base+'en/')
    page.locator('#copy-url').click()
    from playwright.sync_api import expect
    expect(page.locator('#copy-status')).to_have_text('List URL copied.')
    assert page.evaluate('navigator.clipboard.readText()')==list_url
    page.evaluate("Object.defineProperty(navigator,'clipboard',{value:{writeText:()=>Promise.reject(new Error('blocked'))},configurable:true})")
    page.locator('#copy-url').click()
    assert 'Copy unavailable' in page.locator('#copy-status').inner_text()
    page.locator('.languages summary').click(); page.locator('[data-language=de]').click()
    assert page.locator('html').get_attribute('lang')=='de'
    assert page.evaluate("localStorage.getItem('panelnest.language')")=='de'
    page.goto(base); page.wait_for_url('**/de/')
    page.goto(base+'fr/'); assert page.locator('html').get_attribute('lang')=='fr'
    page.keyboard.press('Tab')
    assert page.evaluate('document.activeElement.tagName')=='A'
    # Check every same-origin URL, including assets and anchor targets.
    urls=page.locator('a[href],link[href],img[src],script[src]').evaluate_all('(els)=>els.map(e=>e.href||e.src)')
    for url in set(urls):
        if urlparse(url).netloc==urlparse(base).netloc:
            assert context.request.get(url.split('#')[0]).status==200, url
    assert not errors, errors
    context.close()
    german=browser.new_context(locale='de-DE', color_scheme='light'); fresh=german.new_page(); fresh.goto(base)
    assert fresh.locator('html').get_attribute('lang')=='en'
    assert fresh.locator('html').get_attribute('data-theme')=='dark'
    toggle=fresh.locator('#theme-toggle'); toggle.focus(); fresh.keyboard.press('Enter')
    assert fresh.locator('html').get_attribute('data-theme')=='light'
    fresh.reload(); assert fresh.locator('html').get_attribute('data-theme')=='light'
    toggle.focus(); fresh.keyboard.press('Space')
    assert fresh.locator('html').get_attribute('data-theme')=='dark'
    german.close()
    nojs=browser.new_context(java_script_enabled=False); plain=nojs.new_page(); plain.goto(base+'es/'); assert plain.locator('.source-card').count()==6; assert plain.locator('#import-list').get_attribute('href'); nojs.close()
    browser.close()
(out/'browser-results.json').write_text(json.dumps(results,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps({'routes_and_viewports':len(results),'errors':errors,'copy':'success + denied fallback','locale':'persisted + English despite German browser + explicit route','no_js':'passed','local_links':'passed','screenshots':str(out)}))
