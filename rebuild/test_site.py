"""Static, localized publication contracts."""
import json
from pathlib import Path
import tempfile
import unittest
from html.parser import HTMLParser
from urllib.parse import urlparse, parse_qs
import site_output

class Tags(HTMLParser):
    def __init__(self, text):
        super().__init__(); self.tags=[]; self.feed(text)
    def handle_starttag(self, tag, attrs): self.tags.append((tag,dict(attrs)))

class LocalizedSiteTests(unittest.TestCase):
    def test_five_static_routes_and_safe_import(self):
        with tempfile.TemporaryDirectory() as td:
            root=Path(td)
            (root/'index.json').write_text(json.dumps({'sources':[{'id':'fixture','name':'Fixture <safe>','version':7,'languages':['en'],'contentRating':2}]}))
            (root/'build-report.json').write_text('{"release":false}')
            site_output.prepare(root)
            for lang in ['en','de','es','fr','pt']:
                path=root/lang/'index.html'
                self.assertTrue(path.exists(), f'missing static {lang} route')
                text=path.read_text(encoding='utf-8'); tags=Tags(text).tags
                self.assertIn(('html',{'lang':lang}),tags)
                self.assertIn('Fixture &lt;safe&gt;',text)
                self.assertEqual(sum(t=='h1' for t,a in tags),1)
                self.assertEqual(sum(a.get('rel')=='alternate' for t,a in tags),6)
                cta=next(a['href'] for t,a in tags if a.get('id')=='import-list')
                self.assertEqual(urlparse(cta).netloc,'aidoku.app')
                self.assertEqual(parse_qs(urlparse(cta).query)['url'],['https://luc1ddream.github.io/panelnest/experimental/index.min.json'])
                self.assertIn('source-request.yml',text); self.assertIn('bug-report.yml',text)
            self.assertEqual(json.loads((root/'index.json').read_text())['sources'],[])
            self.assertTrue((root/'sitemap.xml').exists())
