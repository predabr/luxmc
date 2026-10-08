import { describe,it,expect } from 'vitest';
import { removalImpact } from './dependencyImpact';
import type { DependencyNode } from '$lib/api/studio';
const node=(id:string,file:string,requires:string[]=[],provides:string[]=[]):DependencyNode=>({id,name:id,file,requires,provides,version:'1',requiredBy:[],missing:[],metadataKnown:true});
describe('dependency removal impact',()=>{
    it('follows aliases, bundled modules and transitive dependencies through cycles',()=>{
        const nodes=[node('api','api.jar',[],['alias']),node('bundled','api.jar'),node('client','client.jar',['alias','final']),node('final','final.jar',['client']),node('other','other.jar')];
        expect(removalImpact(nodes,'api.jar').map(item=>item.id)).toEqual(['client','final']);
    });
    it('handles large chains without recursive stack growth',()=>{
        const nodes=Array.from({length:5000},(_,i)=>node(String(i),`${i}.jar`,i?[String(i-1)]:[]));
        expect(removalImpact(nodes,'0.jar')).toHaveLength(4999);
    });
});
