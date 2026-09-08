"""Real Chromium release acceptance; no content images downloaded."""
import json
from pathlib import Path
import sys
from playwright.sync_api import sync_playwright

base=sys.argv[1].rstrip('/')+'/'
out=Path(sys.argv[2] if len(sys.argv)>2 else 'output/playwright/theme');out.mkdir(parents=True,exist_ok=True)
results=[]
with sync_playwright() as p:
    browser=p.chromium.launch(channel='chrome',headless=True)
    for lang in ['en','de','es','fr','pt']:
        for width in [1440,390,320]:
            context=browser.new_context(viewport={'width':width,'height':900},locale='de-DE',color_scheme='light')
            page=context.new_page(); errors=[];page.on('pageerror',lambda e:errors.append(str(e)))
            page.goto(base+lang+'/');page.wait_for_load_state('networkidle')
            assert page.locator('html').get_attribute('data-theme')=='dark'
            toggle=page.locator('#theme-toggle');label=toggle.get_attribute('aria-label');assert label and toggle.is_visible()
            for mode,key in [('light','Enter'),('dark','Space')]:
                toggle.focus();assert toggle.evaluate('(el)=>el===document.activeElement')
                page.keyboard.press(key)
                assert page.locator('html').get_attribute('data-theme')==mode
                assert toggle.get_attribute('aria-label')==toggle.inner_text()
                assert page.evaluate('localStorage.getItem("panelnest.theme")')==mode
                assert page.evaluate('document.documentElement.scrollWidth<=innerWidth')
                # At first animation frame, saved preference is already applied.
                page.add_init_script('requestAnimationFrame(()=>window.firstTheme=document.documentElement.dataset.theme)')
                page.reload();page.wait_for_load_state('networkidle')
                assert page.evaluate('window.firstTheme')==mode
                assert page.locator('html').get_attribute('data-theme')==mode
                # WCAG sRGB contrast for primary and muted text, links and buttons.
                ratios=page.evaluate('''() => {
                  function rgb(s){return (s.match(/[\\d.]+/g)||[]).map(Number)}
                  function lum(c){return c.slice(0,3).map(v=>{v/=255;return v<=.04045?v/12.92:((v+.055)/1.055)**2.4}).reduce((a,v,i)=>a+v*[.2126,.7152,.0722][i],0)}
                  return ['body','.intro','.source-card p','.source-card h3','.manual','.list-note','footer','.button.primary','#theme-toggle'].map(sel=>{
                    let el=document.querySelector(sel),s=getComputedStyle(el),bg=s.backgroundColor, parent=el;
                    while(rgb(bg)[3]===0 && parent.parentElement){parent=parent.parentElement;bg=getComputedStyle(parent).backgroundColor}
                    const a=lum(rgb(s.color)),b=lum(rgb(bg));return {selector:sel,ratio:(Math.max(a,b)+.05)/(Math.min(a,b)+.05)}
                  });
                }''')
                for r in ratios:assert r['ratio']>=4.5,(lang,width,mode,r)
                results.append({'language':lang,'width':width,'theme':mode,'contrast':ratios,'keyboard':key,'first_frame':mode})
                if width==390:page.screenshot(path=str(out/f'{lang}-{mode}.png'),full_page=True)
            assert not errors,errors
            context.close()
    context=browser.new_context(locale='de-DE',color_scheme='light');page=context.new_page();page.goto(base);page.wait_for_load_state('networkidle')
    assert page.locator('html').get_attribute('lang')=='en'
    assert page.locator('html').get_attribute('data-theme')=='dark'
    assert 'experimental' not in page.locator('body').inner_text().lower()
    assert page.locator('.source-card').count()==6
    context.close();browser.close()
assert len(results)==30
(out/'results.json').write_text(json.dumps({'base':base,'combinations':len(results),'checks':results},indent=2))
print(json.dumps({'base':base,'theme_locale_viewport_combinations':len(results),'fresh_light_OS':'dark','default_language':'English','keyboard':'Enter + Space','persistence':'reload + first animation frame','contrast':'all measured >=4.5','results':str(out/'results.json')}))
