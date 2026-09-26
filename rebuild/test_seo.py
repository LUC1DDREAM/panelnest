"""Searchable useful content and canonical SEO contracts."""
import json,re,tempfile,unittest
from pathlib import Path
import site_output
from test_site import Tags

class SeoTests(unittest.TestCase):
    def test_localized_catalog_is_described_by_visible_content_and_schema(self):
        with tempfile.TemporaryDirectory() as td:
            root=Path(td);(root/'experimental').mkdir()
            (root/'experimental/index.json').write_text(json.dumps({'sources':[{'id':'en.fixture','name':'Fixture [PN]','version':1,'languages':['en']}]}))
            site_output.render(root,site_output.DEFAULT_URL,'LUC1DDREAM/panelnest')
            for lang in site_output.LOCALES:
                text=(root/lang/'index.html').read_text(encoding='utf-8');tags=Tags(text).tags
                headline=re.search(r'<h1[^>]*>(.*?)</h1>',text,re.S).group(1)
                self.assertIn('Aidoku',headline)
                verification=[a for t,a in tags if t=='meta' and a.get('name')=='google-site-verification']
                self.assertEqual(verification,[{'name':'google-site-verification','content':'WWwPJxNyRoW9NBZ3JSonNBXYxp76m_qN3I87ieOerE8'}])
                self.assertIn('name="google-site-verification"',text.split('</head>')[0])
                self.assertTrue(any(a.get('id')=='faq' for _,a in tags))
                self.assertGreaterEqual(sum(t=='summary' for t,a in tags),5)
                self.assertTrue(any(a.get('id')=='source-en.fixture' for _,a in tags))
                schema=json.loads(re.search(r'<script type="application/ld\+json">(.*?)</script>',text,re.S).group(1))
                self.assertEqual(schema['@context'],'https://schema.org')
                page=next(x for x in schema['@graph'] if x['@type']=='CollectionPage')
                self.assertEqual(page['inLanguage'],lang)
                canonical=next(a['href'] for t,a in tags if a.get('rel')=='canonical')
                self.assertEqual(page['url'],canonical)
                item=page['mainEntity']['itemListElement'][0]
                self.assertEqual(item['name'],'Fixture [PN]')
                self.assertEqual(item['url'],canonical+'sources/en.fixture/')
                self.assertNotIn('aggregateRating',text)
                self.assertNotIn('noindex',text)
