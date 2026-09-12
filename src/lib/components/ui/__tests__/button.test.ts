import { describe, it, expect } from "vitest";
import { button } from "../button";

describe("button", () => {
  it("renders with default variant", () => {
    const classes = button();
    expect(classes).toContain("inline-flex");
    expect(classes).toContain("font-bold");
  });

  it("renders with solid variant", () => {
    const classes = button({ variant: "solid" });
    expect(classes).toContain("bg-gradient-to-b");
  });

  it("renders with primary variant", () => {
    const classes = button({ variant: "primary" });
    expect(classes).toContain("from-brand-500");
  });

  it("renders with danger variant", () => {
    const classes = button({ variant: "danger" });
    expect(classes).toContain("to-danger");
  });

  it("renders with sm size", () => {
    const classes = button({ size: "sm" });
    expect(classes).toContain("h-8");
  });

  it("renders with lg size", () => {
    const classes = button({ size: "lg" });
    expect(classes).toContain("h-11");
  });

  it("renders block when specified", () => {
    const classes = button({ block: true });
    expect(classes).toContain("w-full");
  });

  it("merges custom classes", () => {
    const classes = button({ class: "my-custom-class" });
    expect(classes).toContain("my-custom-class");
  });
});
