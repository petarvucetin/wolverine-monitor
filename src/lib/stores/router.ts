import { writable } from "svelte/store";
import type { Route } from "$lib/types";

export const currentRoute = writable<Route>("dashboard");

export function navigate(route: Route) {
  currentRoute.set(route);
}
