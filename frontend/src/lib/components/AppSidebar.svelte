<script lang="ts">
  import * as Sidebar from "$lib/components/ui/sidebar/index.js";
  import * as Collapsible from "$lib/components/ui/collapsible/index.js";
  import { BackendStatusDot } from "$components";
  import { authStore } from "$lib/features/auth";
  import { page } from "$app/state";
  import { resolve } from "$app/paths";
  import HomeIcon from "@lucide/svelte/icons/home";
  import SettingsIcon from "@lucide/svelte/icons/settings";
  import LogOutIcon from "@lucide/svelte/icons/log-out";
  import WalletIcon from "@lucide/svelte/icons/wallet";
  import UserIcon from "@lucide/svelte/icons/user";
  import SlidersHorizontalIcon from "@lucide/svelte/icons/sliders-horizontal";
  import ChevronRightIcon from "@lucide/svelte/icons/chevron-right";
  import UsersIcon from "@lucide/svelte/icons/users";
  import FolderTreeIcon from "@lucide/svelte/icons/folder-tree";
  import XIcon from "@lucide/svelte/icons/x";
  import { Button } from "$lib/components/ui/button/index.js";
  import { useSidebar } from "$lib/components/ui/sidebar/index.js";

  import { untrack } from "svelte";

  const sidebar = useSidebar();
  const isHomeActive = $derived(page.url.pathname === resolve("/"));
  const isFamilyActive = $derived(page.url.pathname === resolve("/configuration/family"));
  const isHierarchyActive = $derived(page.url.pathname === resolve("/configuration/categories"));
  const isConfigurationActive = $derived(isFamilyActive || isHierarchyActive);
  const isSettingsActive = $derived(page.url.pathname === resolve("/settings"));
  const username = $derived(authStore.currentUser?.name ?? "User");

  let isConfigOpen = $state(true);
  let lastPath = $state(page.url.pathname);

  function handleNavClick(): void {
    if (sidebar.isMobile && sidebar.openMobile) {
      sidebar.setOpenMobile(false);
    }
  }

  $effect(() => {
    if (page.url.pathname.startsWith(resolve("/configuration"))) {
      isConfigOpen = true;
    }
  });

  $effect(() => {
    const current = page.url.pathname;
    if (current !== lastPath) {
      lastPath = current;
      untrack(() => {
        handleNavClick();
      });
    }
  });
</script>

<Sidebar.Root collapsible="icon">
  <Sidebar.Header>
    <div
      class="flex items-center justify-between gap-2 p-2 group-data-[collapsible=icon]:flex-col group-data-[collapsible=icon]:items-center group-data-[collapsible=icon]:gap-2 group-data-[collapsible=icon]:px-0"
    >
      <div
        class="flex items-center gap-2 overflow-hidden group-data-[collapsible=icon]:overflow-visible"
      >
        <div
          class="flex size-8 shrink-0 items-center justify-center rounded-lg bg-emerald-600 text-white shadow-sm"
        >
          <WalletIcon class="size-4" />
        </div>
        <div class="flex flex-col truncate group-data-[collapsible=icon]:hidden">
          <span class="truncate text-sm leading-tight font-semibold">CoSave</span>
          <span class="text-muted-foreground truncate text-xs">Finance Hub</span>
        </div>
      </div>
      <div
        class="flex items-center gap-1 group-data-[collapsible=icon]:w-full group-data-[collapsible=icon]:justify-center"
      >
        <div class="group-data-[collapsible=icon]:hidden">
          <BackendStatusDot />
        </div>
        {#if sidebar.isMobile}
          <Button
            variant="ghost"
            size="icon-sm"
            class="text-muted-foreground hover:text-foreground size-8"
            onclick={() => sidebar.setOpenMobile(false)}
            aria-label="Close menu"
          >
            <XIcon class="size-4" />
          </Button>
        {:else}
          <Sidebar.Trigger />
        {/if}
      </div>
    </div>
  </Sidebar.Header>

  <Sidebar.Content>
    <Sidebar.Group>
      <Sidebar.GroupLabel>Navigation</Sidebar.GroupLabel>
      <Sidebar.GroupContent>
        <Sidebar.Menu class="group-data-[collapsible=icon]:items-center">
          <Sidebar.MenuItem
            class="group-data-[collapsible=icon]:flex group-data-[collapsible=icon]:w-full group-data-[collapsible=icon]:justify-center"
          >
            <Sidebar.MenuButton tooltipContent="Home" isActive={isHomeActive}>
              {#snippet child({ props })}
                <a href={resolve("/")} onclick={handleNavClick} {...props}>
                  <HomeIcon />
                  <span>Home</span>
                </a>
              {/snippet}
            </Sidebar.MenuButton>
          </Sidebar.MenuItem>

          <!-- Collapsible Configuration Section -->
          <Collapsible.Root bind:open={isConfigOpen} class="group/collapsible w-full">
            <Sidebar.MenuItem
              class="group-data-[collapsible=icon]:flex group-data-[collapsible=icon]:w-full group-data-[collapsible=icon]:justify-center"
            >
              <Collapsible.Trigger>
                {#snippet child({ props })}
                  <Sidebar.MenuButton
                    tooltipContent="Configuration"
                    isActive={isConfigurationActive}
                    {...props}
                  >
                    <SlidersHorizontalIcon />
                    <span>Configuration</span>
                    <ChevronRightIcon
                      class="ml-auto size-4 transition-transform duration-200 group-data-[collapsible=icon]:hidden group-data-[state=open]/collapsible:rotate-90"
                    />
                  </Sidebar.MenuButton>
                {/snippet}
              </Collapsible.Trigger>
              <Collapsible.Content>
                <Sidebar.MenuSub>
                  <Sidebar.MenuSubItem>
                    <Sidebar.MenuSubButton isActive={isFamilyActive}>
                      {#snippet child({ props })}
                        <a
                          href={resolve("/configuration/family")}
                          onclick={handleNavClick}
                          {...props}
                        >
                          <UsersIcon />
                          <span>Family</span>
                        </a>
                      {/snippet}
                    </Sidebar.MenuSubButton>
                  </Sidebar.MenuSubItem>
                  <Sidebar.MenuSubItem>
                    <Sidebar.MenuSubButton isActive={isHierarchyActive}>
                      {#snippet child({ props })}
                        <a
                          href={resolve("/configuration/categories")}
                          onclick={handleNavClick}
                          {...props}
                        >
                          <FolderTreeIcon />
                          <span>Transaction Hierarchy</span>
                        </a>
                      {/snippet}
                    </Sidebar.MenuSubButton>
                  </Sidebar.MenuSubItem>
                </Sidebar.MenuSub>
              </Collapsible.Content>
            </Sidebar.MenuItem>
          </Collapsible.Root>
        </Sidebar.Menu>
      </Sidebar.GroupContent>
    </Sidebar.Group>
  </Sidebar.Content>

  <Sidebar.Footer>
    <Sidebar.Menu class="group-data-[collapsible=icon]:items-center">
      <Sidebar.MenuItem
        class="group-data-[collapsible=icon]:flex group-data-[collapsible=icon]:w-full group-data-[collapsible=icon]:justify-center"
      >
        <Sidebar.MenuButton tooltipContent="Settings" isActive={isSettingsActive}>
          {#snippet child({ props })}
            <a href={resolve("/settings")} onclick={handleNavClick} {...props}>
              <SettingsIcon />
              <span>Settings</span>
            </a>
          {/snippet}
        </Sidebar.MenuButton>
      </Sidebar.MenuItem>

      <Sidebar.MenuItem
        class="group-data-[collapsible=icon]:flex group-data-[collapsible=icon]:w-full group-data-[collapsible=icon]:justify-center"
      >
        <Sidebar.MenuButton
          tooltipContent="Log Out"
          onclick={() => {
            handleNavClick();
            authStore.logout();
          }}
        >
          <LogOutIcon />
          <span>Log Out</span>
        </Sidebar.MenuButton>
      </Sidebar.MenuItem>
    </Sidebar.Menu>

    <div
      class="border-border/40 mt-2 flex items-center gap-2 rounded-md border p-2 text-xs group-data-[collapsible=icon]:hidden"
    >
      <div
        class="bg-muted text-muted-foreground flex size-6 shrink-0 items-center justify-center rounded-full"
      >
        <UserIcon class="size-3.5" />
      </div>
      <div class="flex flex-col truncate">
        <span class="text-foreground truncate font-medium">{username}</span>
        <span class="text-muted-foreground truncate text-[10px]">Logged in</span>
      </div>
    </div>
  </Sidebar.Footer>
</Sidebar.Root>
