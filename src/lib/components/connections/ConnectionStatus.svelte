<script lang="ts">
  import type { ConnectionStatus } from "$lib/types";

  interface Props {
    status: ConnectionStatus;
  }
  let { status }: Props = $props();

  const statusText = $derived(typeof status === "string" ? status : "Error");
  const statusColor = $derived(
    status === "Connected"
      ? "bg-green-500"
      : status === "Reconnecting"
        ? "bg-yellow-500"
        : status === "Disconnected"
          ? "bg-gray-500"
          : "bg-red-500"
  );
</script>

<span class="inline-flex items-center gap-1.5 text-xs">
  <span class="w-2 h-2 rounded-full {statusColor}"></span>
  {statusText}
</span>
