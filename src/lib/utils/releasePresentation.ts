export function releaseImages(notes: string): string[] {
    const images: string[] = [];
    for (const match of notes.matchAll(/!\[[^\]]*\]\((https:\/\/[^\s)]+)\)/g)) {
        try { const url=new URL(match[1]); if (['github.com','raw.githubusercontent.com','user-images.githubusercontent.com','private-user-images.githubusercontent.com','luxmc-r92.pages.dev'].includes(url.hostname) && !url.username && !url.password) images.push(url.href); } catch {}
    }
    return [...new Set(images)].slice(0,3);
}
export function releaseSections(notes: string): { title: string; items: string[] }[] {
    const groups: {title:string;items:string[]}[]=[];
    let group={title:'',items:[] as string[]};
    for (const line of notes.split(/\r?\n/)) {
        const heading=/^#{2,4}\s+(.+)$/.exec(line);
        if (heading) { if (group.items.length) groups.push(group); group={title:heading[1],items:[]}; }
        else { const item=/^\s*[-*]\s+(.+)$/.exec(line); if (item && group.items.length<5) group.items.push(item[1].replace(/[*`]/g,'')); }
    }
    if (group.items.length) groups.push(group);
    return groups.slice(0,4);
}
