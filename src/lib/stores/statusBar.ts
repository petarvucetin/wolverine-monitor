import { writable } from "svelte/store";

export interface StatusBarState {
  status: "ready" | "loading" | "error";
  message: string;
}

export const statusBar = writable<StatusBarState>({
  status: "ready",
  message: "Ready",
});
