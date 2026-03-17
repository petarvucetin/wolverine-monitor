import { writable, derived } from "svelte/store";
import type { ConnectionInfo } from "$lib/types";
import { addConnection, removeConnection, testConnection as testConn, listConnections } from "$lib/tauri";
import { toasts } from "./toasts";

export const connections = writable<ConnectionInfo[]>([]);
export const activeConnectionId = writable<string | null>(null);

export const activeConnection = derived(
  [connections, activeConnectionId],
  ([$connections, $activeId]) =>
    $connections.find((c) => c.config.id === $activeId) ?? null
);

export async function loadConnections() {
  try {
    const list = await listConnections();
    connections.set(list);
  } catch (e) {
    toasts.add(`Failed to load connections: ${e}`, "error");
  }
}

export async function createConnection(config: Omit<import("$lib/types").ConnectionConfig, "id">) {
  try {
    const id = await addConnection(config as any);
    toasts.add(`Connected to ${config.name}`, "success");
    // Refresh connection list
    const list = await listConnections();
    connections.set(list);
    activeConnectionId.set(id);
    return id;
  } catch (e) {
    toasts.add(`Connection failed: ${e}`, "error");
    throw e;
  }
}

export async function deleteConnection(id: string) {
  try {
    await removeConnection(id);
    connections.update((cs) => cs.filter((c) => c.config.id !== id));
    activeConnectionId.update((current) => (current === id ? null : current));
    toasts.add("Connection removed", "info");
  } catch (e) {
    toasts.add(`Failed to remove: ${e}`, "error");
  }
}

export async function testConnectionConfig(
  host: string, port: number, database: string,
  username: string, password: string, sslMode: import("$lib/types").SslMode
) {
  try {
    await testConn(host, port, database, username, password, sslMode);
    toasts.add("Connection successful!", "success");
    return true;
  } catch (e) {
    toasts.add(`Connection test failed: ${e}`, "error");
    return false;
  }
}
