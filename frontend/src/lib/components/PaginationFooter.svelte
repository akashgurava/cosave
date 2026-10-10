<script lang="ts">
  import * as Pagination from "$lib/components/ui/pagination";
  import { cn } from "$lib/utils";

  interface Props {
    totalCount: number;
    pageSize?: number;
    currentPage?: number;
    siblingCount?: number;
    entityLabel?: string;
    class?: string;
  }

  let {
    totalCount,
    pageSize = 20,
    currentPage = $bindable(1),
    siblingCount = 1,
    entityLabel = "transactions",
    class: className = "",
  }: Props = $props();

  const totalPages = $derived(Math.max(1, Math.ceil(totalCount / pageSize)));
  const rangeStart = $derived(totalCount === 0 ? 0 : (currentPage - 1) * pageSize + 1);
  const rangeEnd = $derived(Math.min(currentPage * pageSize, totalCount));

  $effect(() => {
    if (currentPage > totalPages && totalPages > 0) {
      currentPage = totalPages;
    }
  });
</script>

{#if totalCount > 0}
  <div
    class={cn(
      "border-border/40 grid grid-cols-1 items-center gap-3 border-t pt-4 sm:grid-cols-3",
      className,
    )}
  >
    <!-- Left spacer to maintain symmetry for centering the middle column -->
    <div class="hidden sm:block"></div>

    <!-- Center column: Paginator strictly centered -->
    <div class="flex justify-center">
      <Pagination.Root
        count={totalCount}
        perPage={pageSize}
        bind:page={currentPage}
        {siblingCount}
        class="mx-0 w-auto"
      >
        {#snippet children({ pages })}
          <Pagination.Content>
            <Pagination.Item>
              <Pagination.Previous />
            </Pagination.Item>
            {#each pages as page (page.key)}
              {#if page.type === "ellipsis"}
                <Pagination.Item>
                  <Pagination.Ellipsis />
                </Pagination.Item>
              {:else}
                <Pagination.Item>
                  <Pagination.Link {page} isActive={currentPage === page.value}>
                    {page.value}
                  </Pagination.Link>
                </Pagination.Item>
              {/if}
            {/each}
            <Pagination.Item>
              <Pagination.Next />
            </Pagination.Item>
          </Pagination.Content>
        {/snippet}
      </Pagination.Root>
    </div>

    <!-- Right column: Summary line all in 1 line at the right end of the table -->
    <div class="flex justify-center sm:justify-end">
      <p class="text-muted-foreground font-mono text-xs whitespace-nowrap">
        Showing <span class="text-foreground font-medium">{rangeStart}</span>–<span
          class="text-foreground font-medium">{rangeEnd}</span
        >
        of <span class="text-foreground font-medium">{totalCount}</span>
        {entityLabel}
      </p>
    </div>
  </div>
{/if}
