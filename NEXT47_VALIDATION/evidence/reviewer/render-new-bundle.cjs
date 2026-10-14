const fs=require('fs'), path=require('path');
const {pathToFileURL}=require('url');
const {chromium}=require('C:/Users/16429/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright');
(async()=>{
 const browser=await chromium.launch({executablePath:'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe',headless:true});
 const context=await browser.newContext({viewport:{width:1600,height:1000}});
 const out=__dirname, records=[];
 const files=[['new-detailed-release',path.join(out,'new-detailed-release.html')]];
 for(const [name,file] of files){
  const page=await context.newPage(),requests=[],errors=[];
  page.on('pageerror',e=>errors.push(String(e)));
  await page.route('**/*',r=>{if(/^https?:/.test(r.request().url())){requests.push(r.request().url());return r.abort();}return r.continue();});
  await page.goto(pathToFileURL(file).href);await page.evaluate(()=>document.fonts.ready);
  await page.screenshot({path:path.join(out,name+'-screen.png'),fullPage:true});
  const layout=await page.evaluate(()=>({width:document.documentElement.clientWidth,scrollWidth:document.documentElement.scrollWidth,rows:document.querySelectorAll('tbody tr').length,text:document.body.innerText,rects:[...document.querySelectorAll('svg rect')].map(x=>({w:+x.getAttribute('width'),h:+x.getAttribute('height')})),textBounds:[...document.querySelectorAll('svg text')].map(x=>{const b=x.getBBox();return{x:b.x,y:b.y,w:b.width,h:b.height,text:x.textContent}})}));
  await page.pdf({path:path.join(out,name+'-a4.pdf'),format:'A4',printBackground:true,preferCSSPageSize:true});
  if(requests.length||errors.length)throw Error(name+' network/script errors');
  if(layout.scrollWidth>layout.width||layout.rects.some(r=>!Number.isFinite(r.w)||r.w<=0))throw Error(name+' invalid geometry');
  const outside=layout.textBounds.filter(r=>r.x<0||r.y<0||r.x+r.w>1600||r.y+r.h>900);
  if(outside.length)throw Error(name+' text outside SVG '+JSON.stringify(outside));
  records.push({name,file,requests,errors,layout,print:'Actual headless Edge A4 browser print, not native app PDF'});await page.close();
 }
 await browser.close();fs.writeFileSync(path.join(out,'browser-new-bundle-review.json'),JSON.stringify(records,null,2));
})().catch(e=>{console.error(e);process.exitCode=1});
