type Animation = {render:(now:number)=>void;interval:number;next?:number};
const animations=new Set<Animation>();
let frame=0;
let timer:ReturnType<typeof setTimeout>|undefined;
let ticking=false;

function schedule() {
    cancelAnimationFrame(frame);clearTimeout(timer);frame=0;timer=undefined;
    if(!animations.size)return;
    const now=performance.now();
    let next=Infinity;
    for(const animation of animations)next=Math.min(next,animation.next??now);
    const delay=Math.max(0,next-now-4);
    if(delay===0)frame=requestAnimationFrame(tick);
    else timer=setTimeout(()=>{timer=undefined;frame=requestAnimationFrame(tick);},delay);
}

function tick(now:number) {
    frame=0;ticking=true;
    try {
        for(const animation of [...animations]) {
            if(!animations.has(animation) || animation.next!==undefined && now<animation.next-.1)continue;
            try {animation.render(now);}
            catch(error) {animations.delete(animation);queueMicrotask(()=>{throw error;});continue;}
            if(!animations.has(animation))continue;
            animation.next=(animation.next??now)+animation.interval;
            if(animation.next<=now)animation.next=now+animation.interval;
        }
    } finally {ticking=false;schedule();}
}

export function frameLoop(render:(now:number)=>void,fps:number):()=>void {
    const animation:Animation={render,interval:1000/Math.max(1,Math.min(60,fps))};
    animations.add(animation);
    if(!ticking)schedule();
    return ()=>{animations.delete(animation);if(!ticking)schedule();};
}
