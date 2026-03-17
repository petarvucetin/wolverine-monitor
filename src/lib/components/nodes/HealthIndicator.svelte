<script lang="ts">
  import { getNodeHealth } from "$lib/format";

  interface Props {
    healthCheck: string | null;
  }

  let { healthCheck }: Props = $props();

  let health = $derived(getNodeHealth(healthCheck));

  let dotColor = $derived(
    health === "Healthy"
      ? "bg-green-400"
      : health === "Warning"
        ? "bg-yellow-400"
        : health === "Critical"
          ? "bg-red-400"
          : "bg-gray-400"
  );

  let labelColor = $derived(
    health === "Healthy"
      ? "text-green-400"
      : health === "Warning"
        ? "text-yellow-400"
        : health === "Critical"
          ? "text-red-400"
          : "text-[var(--color-text-secondary)]"
  );
</script>

<div class="flex items-center gap-2">
  <span class="inline-block w-2 h-2 rounded-full {dotColor}"></span>
  <span class="text-sm {labelColor}">{health}</span>
</div>
