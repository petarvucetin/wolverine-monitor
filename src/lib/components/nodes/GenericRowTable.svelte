<script lang="ts">
  interface Props {
    title: string;
    rows: Record<string, unknown>[];
  }

  let { title, rows }: Props = $props();

  let columns = $derived(
    rows.length > 0 ? Object.keys(rows[0]) : []
  );

  function formatHeader(key: string): string {
    return key.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
  }

  function formatCell(value: unknown): string {
    if (value === null || value === undefined) return "\u2014";
    if (typeof value === "object") return JSON.stringify(value);
    return String(value);
  }
</script>

<div>
  <h2 class="text-lg font-semibold mb-3">{title}</h2>
  <div class="rounded-lg bg-[var(--color-surface-raised)] border border-[var(--color-border)] overflow-hidden">
    {#if rows.length === 0}
      <div class="px-4 py-8 text-center text-sm text-[var(--color-text-secondary)]">
        No data found.
      </div>
    {:else}
      <div class="overflow-x-auto">
        <table class="w-full text-sm">
          <thead>
            <tr class="border-b border-[var(--color-border)] text-left text-xs uppercase text-[var(--color-text-secondary)]">
              {#each columns as col}
                <th class="px-4 py-3 whitespace-nowrap">{formatHeader(col)}</th>
              {/each}
            </tr>
          </thead>
          <tbody>
            {#each rows as row, i}
              <tr class="border-b border-[var(--color-border)] last:border-b-0 hover:bg-[var(--color-surface-overlay)] transition-colors">
                {#each columns as col}
                  <td class="px-4 py-2 font-mono text-xs whitespace-nowrap max-w-xs truncate" title={formatCell(row[col])}>
                    {formatCell(row[col])}
                  </td>
                {/each}
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    {/if}
  </div>
</div>
