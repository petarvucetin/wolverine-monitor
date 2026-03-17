<script lang="ts">
  import type { IncomingEnvelope, OutgoingEnvelope, DeadLetter } from "$lib/types";
  import { shortenMessageType } from "$lib/format";

  type AnyEnvelope = IncomingEnvelope | OutgoingEnvelope | DeadLetter;

  interface Props {
    items: AnyEnvelope[];
    table: string;
    total: number;
    page: number;
    pageSize: number;
    onPageChange: (page: number) => void;
    onSelect: (item: AnyEnvelope) => void;
  }

  let { items, table, total, page, pageSize, onPageChange, onSelect }: Props = $props();

  let totalPages = $derived(Math.max(1, Math.ceil(total / pageSize)));

  function truncateId(id: string): string {
    return id.length > 8 ? id.slice(0, 8) + "\u2026" : id;
  }

  function getStatus(item: AnyEnvelope): string {
    if ("status" in item) return item.status;
    return "\u2014";
  }

  function getContextColumn(item: AnyEnvelope): string {
    if (table === "incoming" && "status" in item) return item.status;
    if (table === "outgoing" && "destination" in item) return item.destination;
    if (table === "dead_letter" && "exception_type" in item) return item.exception_type ?? "\u2014";
    return "\u2014";
  }

  function getContextHeader(): string {
    switch (table) {
      case "incoming": return "Status";
      case "outgoing": return "Destination";
      case "dead_letter": return "Exception";
      default: return "Details";
    }
  }
</script>

<div class="rounded-lg bg-[var(--color-surface-raised)] border border-[var(--color-border)] overflow-hidden">
  {#if items.length === 0}
    <div class="px-4 py-8 text-center text-sm text-[var(--color-text-secondary)]">
      No envelopes found.
    </div>
  {:else}
    <table class="w-full text-sm">
      <thead>
        <tr class="border-b border-[var(--color-border)] text-left text-xs uppercase text-[var(--color-text-secondary)]">
          <th class="px-4 py-3">ID</th>
          <th class="px-4 py-3">Message Type</th>
          <th class="px-4 py-3">{getContextHeader()}</th>
          <th class="px-4 py-3 text-right">Attempts</th>
        </tr>
      </thead>
      <tbody>
        {#each items as item (item.id)}
          <tr
            class="border-b border-[var(--color-border)] last:border-b-0 hover:bg-[var(--color-surface-overlay)] cursor-pointer transition-colors"
            onclick={() => onSelect(item)}
            onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onSelect(item); } }}
            tabindex="0"
            role="button"
          >
            <td class="px-4 py-2 font-mono text-xs">{truncateId(item.id)}</td>
            <td class="px-4 py-2">{shortenMessageType(item.message_type)}</td>
            <td class="px-4 py-2">{getContextColumn(item)}</td>
            <td class="px-4 py-2 text-right">{"attempts" in item ? item.attempts : "\u2014"}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}

  {#if total > 0}
    <div class="flex items-center justify-between px-4 py-3 border-t border-[var(--color-border)] text-xs text-[var(--color-text-secondary)]">
      <span>{total} total</span>
      <div class="flex items-center gap-2">
        <button
          onclick={() => onPageChange(page - 1)}
          disabled={page <= 1}
          class="px-2 py-1 rounded border border-[var(--color-border)] hover:bg-[var(--color-surface-overlay)] disabled:opacity-30"
        >
          Prev
        </button>
        <span>Page {page} of {totalPages}</span>
        <button
          onclick={() => onPageChange(page + 1)}
          disabled={page >= totalPages}
          class="px-2 py-1 rounded border border-[var(--color-border)] hover:bg-[var(--color-surface-overlay)] disabled:opacity-30"
        >
          Next
        </button>
      </div>
    </div>
  {/if}
</div>
