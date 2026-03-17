import { describe, it, expect } from "vitest";
import { get } from "svelte/store";
import { currentRoute, navigate } from "$lib/stores/router";

describe("router store", () => {
  it("starts with dashboard as default route", () => {
    expect(get(currentRoute)).toBe("dashboard");
  });

  it("navigate changes the current route", () => {
    navigate("explorer");
    expect(get(currentRoute)).toBe("explorer");
  });

  it("navigate to deadletters", () => {
    navigate("deadletters");
    expect(get(currentRoute)).toBe("deadletters");
  });

  it("navigate to nodes", () => {
    navigate("nodes");
    expect(get(currentRoute)).toBe("nodes");
  });

  it("navigate to connections", () => {
    navigate("connections");
    expect(get(currentRoute)).toBe("connections");
  });

  it("navigate back to dashboard", () => {
    navigate("dashboard");
    expect(get(currentRoute)).toBe("dashboard");
  });
});
