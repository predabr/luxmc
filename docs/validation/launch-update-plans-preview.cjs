const {chromium}=require('playwright-core');
const fs=require('node:fs');
(async()=>{
 const browser=await chromium.launch({headless:true,executablePath:'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe'});
 fs.mkdirSync('docs/validation/launch-update-plans',{recursive:true});
 try{
  const results=[];
  for(const [name,width,height] of [['desktop',1440,1000],['mobile',390,844]]){
   const page=await browser.newPage({viewport:{width,height},reducedMotion:'reduce'});const errors=[];page.on('pageerror',e=>errors.push(String(e)));
   await page.goto('http://127.0.0.1:8790/planos');await page.getByRole('heading',{name:'Luxmc Pro',exact:true}).waitFor();
   await page.screenshot({path:`docs/validation/launch-update-plans/plans-${name}.png`,fullPage:true});
   const layout=await page.evaluate(()=>({overflow:document.documentElement.scrollWidth>innerWidth+1,heroBackground:getComputedStyle(document.querySelector('.plans-hero')).backgroundColor,cardBackground:getComputedStyle(document.querySelector('.plan-card')).backgroundColor,links:[...document.querySelectorAll('.navbar a')].map(a=>a.getAttribute('href'))}));
   if(layout.overflow)throw Error(`${name} horizontal overflow`);
   if(errors.length)throw Error(errors.join('\n'));results.push({name,...layout,errors});await page.close();
  }
  fs.writeFileSync('docs/validation/launch-update-plans/plans-layout.json',JSON.stringify(results,null,2));console.log(JSON.stringify(results));
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1});
