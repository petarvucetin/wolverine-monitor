export type EnvelopeStatus = "Incoming" | "Scheduled" | "Handled";

export interface IncomingEnvelope {
  id: string;
  status: EnvelopeStatus;
  owner_id: number;
  execution_time: string | null;
  attempts: number;
  body: number[];
  message_type: string;
  received_at: string | null;
  keep_until: string | null;
}

export interface OutgoingEnvelope {
  id: string;
  owner_id: number;
  destination: string;
  deliver_by: string | null;
  body: number[];
  attempts: number;
  message_type: string;
}

export interface DeadLetter {
  id: string;
  execution_time: string | null;
  body: number[];
  message_type: string;
  received_at: string | null;
  source: string | null;
  exception_type: string | null;
  exception_message: string | null;
  sent_at: string | null;
  replayable: boolean;
}

export interface WolverineNode {
  id: string;
  node_number: number;
  description: string | null;
  uri: string | null;
  started: string | null;
  health_check: string | null;
  version: string | null;
  capabilities: string | null;
}

export type NodeHealth = "Healthy" | "Warning" | "Critical" | "Unknown";

export type SslMode = "Disable" | "Prefer" | "Require" | "VerifyCa";

export interface ConnectionConfig {
  id: string;
  name: string;
  routes: Route[];
  host: string;
  port: number;
  database: string;
  schema: string;
  table_prefix: string;
  username: string;
  password: string;
  ssl_mode: SslMode;
}

export type ConnectionStatus =
  | "Connected"
  | "Disconnected"
  | "Reconnecting"
  | { Error: string };

export interface ConnectionInfo {
  config: ConnectionConfig;
  status: ConnectionStatus;
  triggers_installed: boolean;
}

export interface DashboardStats {
  incoming_count: number;
  incoming_scheduled: number;
  incoming_handled: number;
  outgoing_count: number;
  dead_letter_count: number;
  throughput: ThroughputPoint[];
}

export interface ThroughputPoint {
  timestamp: string;
  incoming: number;
  outgoing: number;
}

export interface PaginatedResult<T> {
  items: T[];
  total: number;
  page: number;
  page_size: number;
}

export interface EnvelopeFilters {
  status?: string;
  message_type?: string;
  date_from?: string;
  date_to?: string;
}

export interface BulkReplayResult {
  succeeded: number;
  failed: number;
  errors: { id: string; reason: string }[];
}

export type NotifyOp = "INSERT" | "UPDATE" | "DELETE";

export interface NotifyEvent {
  connection_id: string;
  table: "incoming" | "outgoing" | "dead_letter";
  op: NotifyOp;
  id: string;
  message_type: string;
}

export type Route = "dashboard" | "explorer" | "deadletters" | "nodes" | "queues" | "connections";

export interface QueueInfo {
  name: string;
  table_name: string;
  count: number;
  scheduled_count: number;
  has_scheduled_table: boolean;
}

export interface AlertRule {
  id: string;
  enabled: boolean;
  kind: AlertRuleKind;
}

export type AlertRuleKind =
  | { type: "AnyDeadLetter" }
  | { type: "DeadLetterMessageType"; message_type: string }
  | { type: "IncomingQueueDepth"; threshold: number };
