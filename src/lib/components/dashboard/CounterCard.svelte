<script lang="ts">
  interface Props {
    label: string;
    value: number;
    color: string;
    rate?: string;
  }

  let { label, value, color, rate }: Props = $props();
</script>

<div class="p-4 rounded-lg bg-[var(--color-surface-raised)] border border-[var(--color-border)]">
  <div class="text-xs uppercase tracking-wide text-[var(--color-text-secondary)]">
    {label}
  </div>
  <div class="text-3xl font-bold mt-1" style="color: {color}">
    {value.toLocaleString()}
  </div>
  {#if rate}
    <div class="text-xs text-[var(--color-text-secondary)] mt-1">
      {rate}
    </div>
  {/if}
</div>
