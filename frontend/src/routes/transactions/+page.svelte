<script lang="ts">
  import { onMount } from "svelte";
  import { transactionsStore, TransactionsView } from "$lib/features/transactions";
  import { familyStore } from "$lib/features/family";
  import { categoryStore } from "$lib/features/categories";

  onMount(async () => {
    await Promise.all([familyStore.load(), categoryStore.load()]);
    await transactionsStore.load();
  });
</script>

<svelte:head>
  <title>Transactions — CoSave</title>
</svelte:head>

<div class="container mx-auto max-w-7xl px-4 py-6">
  <TransactionsView />
</div>
