import type { SkinViewer } from "skinview3d";
type CapeMesh = SkinViewer["playerObject"]["cape"]["cape"];
interface CapeCloth { update(time: number, speed: number, turn: number, wind: number, enabled: boolean): void }
const cloths = new WeakMap<CapeMesh,CapeCloth>();
export function createCapeCloth(mesh: CapeMesh): CapeCloth {
    const existing = cloths.get(mesh);
    if (existing) return existing;
    const original = mesh.geometry;
    const originalUV = original.getAttribute("uv");
    const CapeGeometry = original.constructor as new (...args: number[]) => CapeMesh["geometry"];
    const geometry = new CapeGeometry(10, 16, 1, 6, 16, 1);
    const uv = geometry.getAttribute("uv");
    const indices = geometry.index!;
    geometry.groups.forEach((group, face) => {
        const visited = new Set<number>();
        for (let i = group.start; i < group.start + group.count; i++) {
            const vertex = indices.getX(i);
            if (visited.has(vertex)) continue;
            visited.add(vertex);
            const u = uv.getX(vertex), v = uv.getY(vertex);
            const offset = face * 4;
            const bottomX = originalUV.getX(offset+2)*(1-u)+originalUV.getX(offset+3)*u;
            const topX = originalUV.getX(offset)*(1-u)+originalUV.getX(offset+1)*u;
            const bottomY = originalUV.getY(offset+2)*(1-u)+originalUV.getY(offset+3)*u;
            const topY = originalUV.getY(offset)*(1-u)+originalUV.getY(offset+1)*u;
            uv.setXY(vertex,bottomX*(1-v)+topX*v,bottomY*(1-v)+topY*v);
        }
    });
    mesh.geometry = geometry;
    original.dispose();
    const position = geometry.getAttribute("position");
    const rest = new Float32Array(position.array);
    const cloth: CapeCloth = {
        update(time: number, speed: number, turn: number, wind: number, enabled: boolean) {
            for (let i=0;i<position.count;i++) {
                const x=rest[i*3],y=rest[i*3+1],z=rest[i*3+2];
                const length=Math.max(0,Math.min(1,(8-y)/16));
                const bend=enabled ? length*length : 0;
                if (bend === 0) { position.setXYZ(i,x,y,z); continue; }
                const ripple=Math.sin(time*3.5-length*5+x*.16)*(.2+speed*.28)*wind;
                position.setXYZ(i,x+bend*Math.sin(time*2-length*3)*.22*wind,y,z+bend*(ripple+speed*.7+turn*x*.12));
            }
            position.needsUpdate=true;
            geometry.computeVertexNormals();
        }
    };
    cloths.set(mesh,cloth);
    return cloth;
}
