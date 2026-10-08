import type {Body, Vector, Constraint, Vertex} from 'matter-js';

export type LogoPhase = 'reveal' | 'shatter' | 'reassemble' | 'complete';

export async function createLogoFracture(canvas: HTMLCanvasElement, image: HTMLImageElement, phaseChanged: (phase: LogoPhase) => void) {
    const {default: Matter} = await import('matter-js');
    const {Engine, Bodies, Body: RigidBody, Composite, Constraint: Spring, Vertices, Events} = Matter;
    const context = canvas.getContext('2d');
    if (!context) throw new Error('Canvas unavailable');
    const original = document.createElement('canvas');
    original.width = image.naturalWidth; original.height = image.naturalHeight;
    const originalContext = original.getContext('2d')!;
    originalContext.drawImage(image, 0, 0);
    const pixels = originalContext.getImageData(0, 0, original.width, original.height).data;
    let xMin = original.width, yMin = original.height, xMax = 0, yMax = 0;
    for (let y = 0; y < original.height; y++) for (let x = 0; x < original.width; x++) {
        if (pixels[(y * original.width + x) * 4 + 3] > 8) { xMin = Math.min(xMin, x); yMin = Math.min(yMin, y); xMax = Math.max(xMax, x); yMax = Math.max(yMax, y); }
    }
    const size = 320, left = 340, top = 180;
    const texture = document.createElement('canvas'); texture.width = size; texture.height = size;
    texture.getContext('2d')!.drawImage(original, xMin, yMin, xMax - xMin + 1, yMax - yMin + 1, 0, 0, size, size);
    const engine = Engine.create({positionIterations: 8, velocityIterations: 6});
    engine.gravity.y = .85;
    const floor = Bodies.rectangle(500, 595, 1400, 50, {isStatic: true, restitution: .35});
    Composite.add(engine.world, [floor, Bodies.rectangle(-25, 350, 50, 900, {isStatic: true}), Bodies.rectangle(1025, 350, 50, 900, {isStatic: true})]);
    let contacts = 0;
    Events.on(engine, 'collisionStart', event => { contacts += event.pairs.filter(pair => pair.bodyA === floor || pair.bodyB === floor).length; });
    let seed = 27;
    const random = () => { seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0; return seed / 4294967296; };
    const grid: Vector[][] = Array.from({length: 6}, (_, row) => Array.from({length: 6}, (_, col) => ({x: left + col * size / 5 + (col > 0 && col < 5 ? (random() - .5) * 23 : 0), y: top + row * size / 5 + (row > 0 && row < 5 ? (random() - .5) * 23 : 0)})));
    const fragments: {body: Body; home: Vector; patch: HTMLCanvasElement; offset: Vector; spring?: Constraint}[] = [];
    for (let row = 0; row < 5; row++) for (let col = 0; col < 5; col++) {
        const a = grid[row][col], b = grid[row][col + 1], c = grid[row + 1][col + 1], d = grid[row + 1][col];
        for (const polygon of (row + col) % 2 ? [[a, b, d], [b, c, d]] : [[a, b, c], [a, c, d]]) {
            const home = Vertices.centre(polygon as Vertex[]);
            const minX = Math.floor(Math.min(...polygon.map(point => point.x))), minY = Math.floor(Math.min(...polygon.map(point => point.y)));
            const patch = document.createElement('canvas');
            patch.width = Math.ceil(Math.max(...polygon.map(point => point.x))) - minX;
            patch.height = Math.ceil(Math.max(...polygon.map(point => point.y))) - minY;
            const paint = patch.getContext('2d')!;
            paint.beginPath(); polygon.forEach((point, index) => index ? paint.lineTo(point.x - minX, point.y - minY) : paint.moveTo(point.x - minX, point.y - minY)); paint.closePath(); paint.clip();
            paint.drawImage(texture, left - minX, top - minY);
            if (!paint.getImageData(0, 0, patch.width, patch.height).data.some((value, index) => index % 4 === 3 && value > 8)) continue;
            const body = Bodies.fromVertices(home.x, home.y, [polygon], {density: .002, restitution: .4, friction: .5, frictionAir: .018, slop: .01});
            RigidBody.setStatic(body, true);
            fragments.push({body, home, patch, offset: {x: minX - home.x, y: minY - home.y}});
            Composite.add(engine.world, body);
        }
    }
    let phase: LogoPhase = 'reveal', broken = false, returning = false, settled = false, accumulator = 0;
    const setPhase = (value: LogoPhase) => { if (phase !== value) { phase = value; phaseChanged(value); } };
    const resize = () => { const ratio = Math.min(devicePixelRatio || 1, 1.5); canvas.width = Math.round(canvas.clientWidth * ratio); canvas.height = Math.round(canvas.clientHeight * ratio); };
    resize();
    const observer = new ResizeObserver(resize); observer.observe(canvas);
    return {
        render(elapsed: number, delta: number) {
            if (elapsed >= 900 && !broken) {
                broken = true; setPhase('shatter');
                for (const fragment of fragments) {
                    RigidBody.setStatic(fragment.body, false);
                    const dx = fragment.home.x - 500, dy = fragment.home.y - 340;
                    RigidBody.setVelocity(fragment.body, {x: dx * .035 + (random() - .5) * 2, y: dy * .025 - 5 - random() * 3});
                    RigidBody.setAngularVelocity(fragment.body, (random() - .5) * .16);
                }
            }
            if (elapsed >= 2450 && !returning) {
                returning = true; setPhase('reassemble'); engine.gravity.y = 0;
                for (const fragment of fragments) {
                    fragment.body.collisionFilter.mask = 0;
                    fragment.body.frictionAir = .09;
                    fragment.spring = Spring.create({pointA: fragment.home, bodyB: fragment.body, length: 0, stiffness: .035, damping: .22});
                    Composite.add(engine.world, fragment.spring);
                }
            }
            if (broken && !settled) {
                accumulator += Math.min(delta, 50);
                while (accumulator >= 1000 / 120) {
                    if (returning) for (const fragment of fragments) {
                        if (fragment.spring) fragment.spring.stiffness = .035 + Math.min(1, (elapsed - 2450) / 1700) * .06;
                        fragment.body.torque = fragment.body.inertia * (-fragment.body.angle * .0003 - fragment.body.angularVelocity * .005);
                    }
                    Engine.update(engine, 1000 / 120); accumulator -= 1000 / 120;
                }
            }
            const error = Math.max(...fragments.map(({body, home}) => Math.hypot(body.position.x - home.x, body.position.y - home.y)));
            canvas.dataset.fragments = String(fragments.length); canvas.dataset.displacement = error.toFixed(2);
            canvas.dataset.elapsed = elapsed.toFixed(0);
            canvas.dataset.floorContacts = String(contacts); canvas.dataset.rotationError = Math.max(...fragments.map(({body}) => Math.abs(body.angle))).toFixed(4);
            if (elapsed >= 4750) { settled = true; setPhase('complete'); }
            context.setTransform(1, 0, 0, 1, 0, 0); context.clearRect(0, 0, canvas.width, canvas.height);
            const scale = Math.min(canvas.width / 1000, canvas.height / 700);
            context.setTransform(scale, 0, 0, scale, (canvas.width - 1000 * scale) / 2, (canvas.height - 700 * scale) / 2);
            context.globalAlpha = Math.min(1, elapsed / 500);
            if (!broken || settled) context.drawImage(texture, left, top, size, size);
            else for (const fragment of fragments) { context.save(); context.translate(fragment.body.position.x, fragment.body.position.y); context.rotate(fragment.body.angle); context.drawImage(fragment.patch, fragment.offset.x, fragment.offset.y); context.restore(); }
        },
        destroy() { observer.disconnect(); Composite.clear(engine.world, false); Engine.clear(engine); },
    };
}
