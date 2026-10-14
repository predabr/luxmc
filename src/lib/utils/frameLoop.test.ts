import {afterEach,describe,expect,it,vi} from 'vitest';
import {frameLoop} from './frameLoop';

afterEach(()=>vi.unstubAllGlobals());

describe('display independent animation scheduling',()=>{
 for(const refreshRate of [60,120,144,165,240]){
  for(const fps of [30,60]){
   it(`keeps ${fps} FPS on a ${refreshRate} Hz display and cancels all work`,()=>{
    let now=0,id=0,frames=0;
    const queue=new Map<number,{at:number;callback:()=>void}>();
    const refresh=1000/refreshRate;
    const schedule=(callback:()=>void,at:number)=>{queue.set(++id,{at,callback});return id;};
    vi.stubGlobal('performance',{now:()=>now});
    vi.stubGlobal('requestAnimationFrame',(callback:FrameRequestCallback)=>schedule(()=>callback(now),(Math.floor(now/refresh+.000001)+1)*refresh));
    vi.stubGlobal('cancelAnimationFrame',(id:number)=>queue.delete(id));
    vi.stubGlobal('setTimeout',(callback:()=>void,delay:number)=>schedule(callback,now+delay));
    vi.stubGlobal('clearTimeout',(id:number)=>queue.delete(id));
    const stop=frameLoop(()=>frames++,fps);
    const advance=(until:number)=>{
     while(queue.size){
      const [id,event]=[...queue.entries()].sort((a,b)=>a[1].at-b[1].at)[0];
      if(event.at>until)break;
      now=event.at;queue.delete(id);event.callback();
     }
     now=until;
    };
    advance(2000);
    expect(frames).toBeGreaterThanOrEqual(fps*2-2);
    expect(frames).toBeLessThanOrEqual(fps*2+1);
    stop();const ended=frames;advance(3000);
    expect(frames).toBe(ended);
    expect(queue.size).toBe(0);
   });
  }
 }
});
