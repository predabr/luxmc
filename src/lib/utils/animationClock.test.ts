import { describe, expect, it } from "vitest";
import { createAnimationClock } from "./animationClock";
describe("preview animation timing", () => {
    for (const hz of [60, 90, 120, 144]) it(`keeps elapsed animation time stable at ${hz}Hz`, () => {
        const clock = createAnimationClock(0);
        let animated = 0;
        for (let frame = 1; frame <= hz; frame++) animated += clock.take(frame * 1000 / hz);
        expect(animated).toBeGreaterThan(.96);
        expect(animated).toBeLessThanOrEqual(1.001);
    });
    it("does not jump after losing visibility or a stalled frame", () => {
        const clock = createAnimationClock(0);
        expect(clock.take(2000)).toBe(.05);
        clock.reset(3000);
        expect(clock.take(3017)).toBeCloseTo(.017);
        expect(clock.take(Number.NaN)).toBe(0);
    });
});
