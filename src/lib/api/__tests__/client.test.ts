import { describe, it, expect, vi } from "vitest";
import { api } from "../client";

describe("api", () => {
  it("has invoke method", () => {
    expect(typeof api.invoke).toBe("function");
  });
});
