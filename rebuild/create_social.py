"""Rebuild the original PanelNest social card using Pillow; no remote assets."""
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

def create():
    image=Image.new('RGB',(1200,630),'#f4f7fb'); d=ImageDraw.Draw(image)
    d.rounded_rectangle((760,70,1130,560), radius=45, fill='#e6edfa')
    d.rounded_rectangle((860,185,1030,355), radius=40, fill='#265cc5')
    def line(points): d.line(points,fill='white',width=8,joint='curve')
    line([(902,270),(902,226),(990,226),(990,270)])
    line([(946,226),(946,265)]); line([(902,265),(990,265)])
    line([(891,286),(946,317),(1001,286)])
    line([(899,302),(946,329),(993,302)])
    line([(918,278),(946,294),(974,278)])
    # Pillow's bundled font avoids OS-specific font files and remote downloads.
    d.text((70,178),'PanelNest.',font=ImageFont.load_default(size=76),fill='#172b46')
    d.text((75,288),'Independent sources for Aidoku',font=ImageFont.load_default(size=29),fill='#526279')
    d.text((75,438),'iPhone + iPad  /  Experimental',font=ImageFont.load_default(size=22),fill='#265cc5')
    image.save(Path(__file__).with_name('site-assets')/'social.png',optimize=True)
if __name__=='__main__': create()
