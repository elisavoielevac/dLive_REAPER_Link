from pathlib import Path
from xml.sax.saxutils import escape
import re
from reportlab.platypus import SimpleDocTemplate,Paragraph,Spacer,Image,Table,TableStyle,PageBreak,KeepTogether
from reportlab.lib.styles import getSampleStyleSheet,ParagraphStyle
from reportlab.lib import colors
from reportlab.lib.enums import TA_CENTER
from reportlab.lib.pagesizes import A4
from reportlab.lib.utils import ImageReader

ROOT=Path(__file__).resolve().parent
SOURCE=ROOT/'USER_MANUAL.md'
OUTPUT=ROOT/'dLive-REAPER-Link-User-Guide.pdf'
W,H=A4
INK=colors.HexColor('#243238');TEAL=colors.HexColor('#24746e');MUTED=colors.HexColor('#53616a')
styles=getSampleStyleSheet()
styles.add(ParagraphStyle('GuideTitle',fontName='Helvetica-Bold',fontSize=24,leading=29,textColor=INK,spaceAfter=14))
styles.add(ParagraphStyle('GuideSubtitle',fontName='Helvetica',fontSize=13,leading=18,textColor=TEAL,spaceAfter=12))
styles.add(ParagraphStyle('GuideHeading',fontName='Helvetica-Bold',fontSize=11.5,leading=16,textColor=TEAL,spaceBefore=11,spaceAfter=6))
styles.add(ParagraphStyle('GuideBody',fontName='Helvetica',fontSize=10,leading=14,textColor=INK,spaceAfter=8))
styles.add(ParagraphStyle('GuideCaption',fontName='Helvetica-Oblique',fontSize=8,leading=11,textColor=MUTED,spaceAfter=9))
styles.add(ParagraphStyle('GuideCell',fontName='Helvetica',fontSize=9,leading=12,textColor=INK))

def inline(text):
    text=escape(text)
    text=re.sub(r'\*\*(.+?)\*\*',r'<b>\1</b>',text)
    return text

def footer(canvas,doc):
    canvas.setStrokeColor(TEAL);canvas.setLineWidth(.8);canvas.line(44,H-32,W-44,H-32)
    canvas.setFont('Helvetica',8);canvas.setFillColor(MUTED)
    canvas.drawString(44,H-24,'dLive REAPER Link  /  User guide')
    canvas.drawString(44,25,'v0.3.0-beta.1  |  24 October 2026')
    canvas.drawRightString(W-44,25,str(doc.page))

story=[]
for pi,page in enumerate(SOURCE.read_text().split('<!-- page -->')):
    if pi:story.append(PageBreak())
    lines=page.strip().splitlines();i=0
    while i<len(lines):
        line=lines[i].strip();i+=1
        if not line:continue
        if line.startswith('# '):story.append(Paragraph(inline(line[2:]),styles['GuideTitle']));continue
        if line.startswith('## '):story.append(Paragraph(inline(line[3:]),styles['GuideSubtitle']));continue
        if line.startswith('### '):story.append(Paragraph(inline(line[4:]),styles['GuideHeading']));continue
        match=re.fullmatch(r'!\[(.*?)\]\((.*?)\)',line)
        if match:
            path=ROOT/match[2];iw,ih=ImageReader(str(path)).getSize()
            maxw=300 if match[2].endswith('live.png') else W-100
            maxh=260 if 'reaper-osc' in match[2] else 295
            scale=min(maxw/iw,maxh/ih)
            im=Image(str(path),width=iw*scale,height=ih*scale);im.hAlign='CENTER'
            story.extend([im,Spacer(1,7)]);continue
        if line.startswith('|'):
            tablelines=[line]
            while i<len(lines) and lines[i].strip().startswith('|'):
                tablelines.append(lines[i].strip());i+=1
            rows=[]
            for row in tablelines:
                cells=[v.strip() for v in row.strip('|').split('|')]
                if all(re.fullmatch(r':?-+:?',v) for v in cells):continue
                rows.append([Paragraph(inline(v),styles['GuideCell']) for v in cells])
            table=Table(rows,colWidths=[175,W-263],hAlign='LEFT')
            table.setStyle(TableStyle([('BACKGROUND',(0,0),(-1,0),colors.HexColor('#e5efee')),('FONTNAME',(0,0),(-1,0),'Helvetica-Bold'),('VALIGN',(0,0),(-1,-1),'TOP'),('LEFTPADDING',(0,0),(-1,-1),8),('RIGHTPADDING',(0,0),(-1,-1),8),('TOPPADDING',(0,0),(-1,-1),5),('BOTTOMPADDING',(0,0),(-1,-1),5),('LINEBELOW',(0,0),(-1,-1),.3,colors.HexColor('#d5dfdf'))]))
            story.extend([table,Spacer(1,9)]);continue
        if line.startswith('*') and line.endswith('*') and not line.startswith('**'):
            story.append(Paragraph(inline(line[1:-1]),styles['GuideCaption']));continue
        if line.startswith('- '):
            story.append(Paragraph(inline(line[2:]),styles['GuideBody'],bulletText='-'));continue
        number=re.match(r'(\d+)\. (.*)',line)
        if number:
            story.append(Paragraph(inline(number[2]),styles['GuideBody'],bulletText=number[1]+'.'));continue
        story.append(Paragraph(inline(line),styles['GuideBody']))
SimpleDocTemplate(str(OUTPUT),pagesize=A4,rightMargin=44,leftMargin=44,topMargin=48,bottomMargin=46,title='dLive REAPER Link - User Guide v0.3.0-beta.1',author='dLive REAPER Link',subject='Installation, scene lists, REAPER OSC, playback recall and troubleshooting').build(story,onFirstPage=footer,onLaterPages=footer)
print(OUTPUT)
