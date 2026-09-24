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
            (root/'index.json').write_text(json.dumps({'sources':[{'id':'fixture','name':'Fixture <safe>','version':7,'languages':['en'],'contentRating':2,'features':['search','details'],'listings':['Popular <safe>'],'limitations':'Remote site may block requests.'}]}))
            (root/'build-report.json').write_text('{"release":true}')
            site_output.prepare(root)
            for lang in ['en','de','es','fr','pt']:
                path=root/lang/'index.html'
                self.assertTrue(path.exists(), f'missing static {lang} route')
                text=path.read_text(encoding='utf-8'); tags=Tags(text).tags
                self.assertIn(('html',{'lang':lang,'data-theme':'dark'}),tags)
                self.assertIn('Fixture &lt;safe&gt;',text)
                self.assertEqual(sum(t=='h1' for t,a in tags),1)
                self.assertEqual(sum(a.get('rel')=='alternate' for t,a in tags),6)
                cta=next(a['href'] for t,a in tags if a.get('id')=='import-list')
                self.assertEqual(urlparse(cta).netloc,'aidoku.app')
                self.assertEqual(parse_qs(urlparse(cta).query)['url'],['https://luc1ddream.github.io/panelnest/index.min.json'])
                self.assertIn('source-request.yml',text); self.assertIn('bug-report.yml',text)
                expected = {'en':'Search · Details','de':'Suche · Details','es':'Búsqueda · Detalles','fr':'Recherche · Détails','pt':'Busca · Detalhes'}[lang]
                full = {'en':'Search · Details · Chapters · Pages','de':'Suche · Details · Kapitel · Seiten','es':'Búsqueda · Detalles · Capítulos · Páginas','fr':'Recherche · Détails · Chapitres · Pages','pt':'Busca · Detalhes · Capítulos · Páginas'}[lang]
                self.assertIn(expected,text)
                self.assertNotIn(full,text)
                self.assertIn('Popular &lt;safe&gt;',text)
                self.assertIn('Remote site may block requests.',text)
            self.assertEqual(len(json.loads((root/'index.json').read_text())['sources']),1)
            self.assertTrue((root/'sitemap.xml').exists())

    def test_extended_capabilities_render_in_every_locale(self):
        with tempfile.TemporaryDirectory() as td:
            root=Path(td)
            (root/'index.json').write_text(json.dumps({'sources':[{'id':'fixture','name':'Fixture','version':7,'languages':['en'],'contentRating':0,'features':['page-descriptions','image-request']}]}))
            (root/'build-report.json').write_text('{"release":true}')
            site_output.prepare(root)
            expected={
                'en':'Image requests · Page descriptions',
                'de':'Bildanfragen · Seitenbeschreibungen',
                'es':'Solicitudes de imagen · Descripciones de página',
                'fr':'Requêtes d’image · Descriptions des pages',
                'pt':'Pedidos de imagem · Descrições das páginas',
            }
            for lang, labels in expected.items():
                self.assertIn(labels,(root/lang/'index.html').read_text(encoding='utf-8'))
