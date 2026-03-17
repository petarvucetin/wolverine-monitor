import { writable, get } from "svelte/store";
import type { Route } from "$lib/types";
import { connections, activeConnectionId } from "./connections";

export const currentRoute = writable<Route>("dashboard");

export function navigate(route: Route) {
  currentRoute.set(route);

  // Auto-select the connection that owns this route
  if (route === "connections") return;

  const conns = get(connections);
  const match = conns.find((c) => c.config.routes.includes(route));
  if (match) {
    activeConnectionId.set(match.config.id);
  }
}
