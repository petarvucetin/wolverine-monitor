<script lang="ts">
  import ConnectionForm from "$lib/components/connections/ConnectionForm.svelte";
  import ConnectionList from "$lib/components/connections/ConnectionList.svelte";
</script>

<div class="p-6 max-w-3xl">
  <h1 class="text-xl font-semibold mb-6">Connections</h1>

  <div class="space-y-6">
    <ConnectionList />
    <ConnectionForm />
  </div>
</div>
