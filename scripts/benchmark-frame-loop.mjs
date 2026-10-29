import ts from 'typescript';
import {readFileSync,writeFileSync} from 'node:fs';
import {resolve} from 'node:path';

const source=readFileSync(resolve('src/lib/utils/frameLoop.ts'),'utf8');
const code=ts.transpileModule(source,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.ES2022}}).outputText;
const original={performance:globalThis.performance,requestAnimationFrame:globalThis.requestAnimationFrame,cancelAnimationFrame:globalThis.cancelAnimationFrame,setTimeout:globalThis.setTimeout,clearTimeout:globalThis.clearTimeout};
const report=[];
try {
    for(const count of [1,8,24]) {
        let now=0,id=0,rafCallbacks=0,timerCallbacks=0;
        const queue=new Map();
        const schedule=(callback,at,type)=>{queue.set(++id,{at,callback,type});return id;};
        globalThis.performance={now:()=>now};
        globalThis.requestAnimationFrame=callback=>schedule(()=>callback(now),(Math.floor(now/(1000/144)+.000001)+1)*(1000/144),'raf');
        globalThis.cancelAnimationFrame=key=>queue.delete(key);
        globalThis.setTimeout=(callback,delay)=>schedule(callback,now+delay,'timer');
        globalThis.clearTimeout=key=>queue.delete(key);
        const {frameLoop}=await import(`data:text/javascript;base64,${Buffer.from(code+`\nvoid ${count}`).toString('base64')}`);
        const rendered=Array(count).fill(0);
        const stops=rendered.map((_,index)=>frameLoop(()=>rendered[index]++,index%2?30:60));
        while(queue.size) {
            const [key,event]=[...queue.entries()].sort((a,b)=>a[1].at-b[1].at)[0];
            if(event.at>2000)break;
            now=event.at;queue.delete(key);
            if(event.type==='raf')rafCallbacks++;else timerCallbacks++;
            event.callback();
        }
        stops.forEach(stop=>stop());
        if(queue.size)throw new Error('Animation callbacks remain after stopping');
        report.push({views:count,displayHz:144,simulatedMs:2000,rafCallbacks,timerCallbacks,rendered});
    }
} finally {Object.assign(globalThis,original);}
const result={kind:'deterministic scheduler benchmark; not a hardware FPS measurement',scenarios:report};
if(process.argv[2])writeFileSync(process.argv[2],JSON.stringify(result,null,2));
console.log(JSON.stringify(result));
