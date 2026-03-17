<script lang="ts">
  import type { DeadLetter } from "$lib/types";
  import { shortenMessageType, formatRelativeTime } from "$lib/format";
  import { selectedIds } from "$lib/stores/deadLetters";

  interface Props {
    items: DeadLetter[];
    onSelect: (item: DeadLetter) => void;
  }

  let { items, onSelect }: Props = $props();

  let allSelected = $derived(
    items.length > 0 && items.every((item) => $selectedIds.has(item.id))
  );

  function toggleSelect(id: string) {
    selectedIds.update((ids) => {
      const next = new Set(ids);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  }

  function toggleAll() {
    if (allSelected) {
      selectedIds.update((ids) => {
        const next = new Set(ids);
        for (const item of items) {
          next.delete(item.id);
        }
        return next;
      });
    } else {
      selectedIds.update((ids) => {
        const next = new Set(ids);
        for (const item of items) {
          next.add(item.id);
        }
        return next;
      });
    }
  }

  function truncateId(id: string): string {
    return id.length > 8 ? id.slice(0, 8) + "\u2026" : id;
  }
</script>

<div class="rounded-lg bg-[var(--color-surface-raised)] border border-[var(--color-border)] overflow-hidden">
  {#if items.length === 0}
    <div class="px-4 py-8 text-center text-sm text-[var(--color-text-secondary)]">
      No dead letters found.
    </div>
  {:else}
    <table class="w-full text-sm">
      <thead>
        <tr class="border-b border-[var(--color-border)] text-left text-xs uppercase text-[var(--color-text-secondary)]">
          <th class="px-4 py-3 w-10">
            <input
              type="checkbox"
              checked={allSelected}
              onchange={toggleAll}
              class="rounded border-[var(--color-border)]"
            />
          </th>
          <th class="px-4 py-3">ID</th>
          <th class="px-4 py-3">Message Type</th>
          <th class="px-4 py-3">Exception</th>
          <th class="px-4 py-3">Sent</th>
          <th class="px-4 py-3 text-center">Replayable</th>
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
            <td class="px-4 py-2" onclick={(e) => e.stopPropagation()}>
              <input
                type="checkbox"
                checked={$selectedIds.has(item.id)}
                onchange={() => toggleSelect(item.id)}
                class="rounded border-[var(--color-border)]"
              />
            </td>
            <td class="px-4 py-2 font-mono text-xs">{truncateId(item.id)}</td>
            <td class="px-4 py-2">{shortenMessageType(item.message_type)}</td>
            <td class="px-4 py-2 text-red-400 truncate max-w-[200px]">
              {item.exception_type ?? "\u2014"}
            </td>
            <td class="px-4 py-2 text-[var(--color-text-secondary)]">
              {formatRelativeTime(item.sent_at)}
            </td>
            <td class="px-4 py-2 text-center">
              {#if item.replayable}
                <span class="text-green-400">Yes</span>
              {:else}
                <span class="text-[var(--color-text-secondary)]">No</span>
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>
