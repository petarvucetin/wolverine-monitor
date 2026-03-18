import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type {
  ConnectionConfig,
  ConnectionInfo,
  DashboardStats,
  DeadLetter,
  EnvelopeFilters,
  IncomingEnvelope,
  OutgoingEnvelope,
  WolverineNode,
  PaginatedResult,
  BulkReplayResult,
  NotifyEvent,
  SslMode,
} from "./types";

// Connection commands
export const listConnections = () =>
  invoke<ConnectionInfo[]>("list_connections");

export const addConnection = (config: Omit<ConnectionConfig, "id">) =>
  invoke<string>("add_connection", { config });

export const updateConnection = (connectionId: string, updates: Partial<ConnectionConfig>) =>
  invoke<void>("update_connection", { connectionId, updates });

export const removeConnection = (connectionId: string) =>
  invoke<void>("remove_connection", { connectionId });

export const testConnection = (
  host: string, port: number, database: string,
  username: string, password: string, sslMode: SslMode
) =>
  invoke<void>("test_connection", { host, port, database, username, password, sslMode });

// Envelope commands
export const getIncomingEnvelopes = (
  connectionId: string, filters: EnvelopeFilters, page: number, pageSize: number
) =>
  invoke<PaginatedResult<IncomingEnvelope>>("get_incoming_envelopes", {
    connectionId, filters, page, pageSize,
  });

export const getOutgoingEnvelopes = (
  connectionId: string, filters: EnvelopeFilters, page: number, pageSize: number
) =>
  invoke<PaginatedResult<OutgoingEnvelope>>("get_outgoing_envelopes", {
    connectionId, filters, page, pageSize,
  });

// Dead letter commands
export const getDeadLetters = (
  connectionId: string, filters: EnvelopeFilters, page: number, pageSize: number
) =>
  invoke<PaginatedResult<DeadLetter>>("get_dead_letters", {
    connectionId, filters, page, pageSize,
  });

export const replayDeadLetter = (connectionId: string, id: string) =>
  invoke<void>("replay_dead_letter", { connectionId, id });

export const replayDeadLettersBulk = (connectionId: string, ids: string[]) =>
  invoke<BulkReplayResult>("replay_dead_letters_bulk", { connectionId, ids });

// Node commands
export const getNodes = (connectionId: string) =>
  invoke<WolverineNode[]>("get_nodes", { connectionId });

// Queue commands
export const getQueues = (connectionId: string) =>
  invoke<import("./types").QueueInfo[]>("get_queues", { connectionId });

export const getQueueMessages = (
  connectionId: string, queueName: string, scheduled: boolean,
  page: number, pageSize: number
) =>
  invoke<import("./types").PaginatedResult<Record<string, unknown>>>("get_queue_messages", {
    connectionId, queueName, scheduled, page, pageSize,
  });

// Dashboard commands
export const getDashboardStats = (connectionId: string) =>
  invoke<DashboardStats>("get_dashboard_stats", { connectionId });

export const getMessageDetail = (connectionId: string, table: string, id: string) =>
  invoke<Record<string, unknown>>("get_message_detail", { connectionId, table, id });

// Trigger commands
export const installTriggers = (connectionId: string) =>
  invoke<void>("install_triggers", { connectionId });

export const uninstallTriggers = (connectionId: string) =>
  invoke<void>("uninstall_triggers", { connectionId });

// Event listeners
export const onEnvelopeChange = (callback: (event: NotifyEvent) => void): Promise<UnlistenFn> =>
  listen<NotifyEvent>("envelope:changed", (e) => callback(e.payload));

export const onConnectionStatus = (
  callback: (event: { connection_id: string; status: string }) => void
): Promise<UnlistenFn> =>
  listen("connection:status", (e) => callback(e.payload as any));

export const onAlert = (
  callback: (event: { connection_id: string; message: string }) => void
): Promise<UnlistenFn> =>
  listen("alert:triggered", (e) => callback(e.payload as any));
