import type { DependencyNode } from '$lib/api/studio';
export function removalImpact(nodes:DependencyNode[],file:string):DependencyNode[] {
    const providers=new Map<string,string>();
    for(const node of nodes){providers.set(node.id,node.id);for(const alias of node.provides)providers.set(alias,node.id);}
    const dependents=new Map<string,Set<string>>();
    for(const node of nodes)for(const id of node.requires){const target=providers.get(id);if(!target)continue;let list=dependents.get(target);if(!list){list=new Set();dependents.set(target,list);}list.add(node.id);}
    const impacted=new Set(nodes.filter(node=>node.file===file).map(node=>node.id));
    const queue=[...impacted];
    for(let index=0;index<queue.length;index++)for(const id of dependents.get(queue[index]) || [])if(!impacted.has(id)){impacted.add(id);queue.push(id);}
    return nodes.filter(node=>impacted.has(node.id) && node.file!==file);
}
