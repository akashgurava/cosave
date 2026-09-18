<script lang="ts">
  import { authStore } from "../store";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";

  interface Props {
    isOpen: boolean;
    onClose: () => void;
  }

  let { isOpen, onClose }: Props = $props();

  let mode = $state<"login" | "register">("login");
  let name = $state("");
  let password = $state("");
  let errorMsg = $state<string | null>(null);
  let isSubmitting = $state(false);

  let showPassword = $state(false);

  function resetForm(): void {
    name = "";
    password = "";
    showPassword = false;
    errorMsg = null;
  }

  function switchMode(newMode: "login" | "register"): void {
    mode = newMode;
    errorMsg = null;
  }

  async function handleSubmit(event: SubmitEvent): Promise<void> {
    event.preventDefault();
    errorMsg = null;

    // Read directly from DOM elements as a fallback for password managers (such as Bitwarden)
    // that inject values directly into input nodes without dispatching full synthetic events
    const form = event.currentTarget as HTMLFormElement;
    const usernameInput = form.elements.namedItem("username") as HTMLInputElement | null;
    const passwordInput = form.elements.namedItem("password") as HTMLInputElement | null;

    const effectiveName = (usernameInput?.value ?? name).trim();
    const effectivePassword = passwordInput?.value ?? password;

    name = effectiveName;
    password = effectivePassword;

    if (!effectiveName) {
      errorMsg = "Please enter your username.";
      return;
    }

    if (effectivePassword.length < 6) {
      errorMsg = "Password must be at least 6 characters.";
      return;
    }

    isSubmitting = true;
    try {
      if (mode === "login") {
        await authStore.login({ name: effectiveName, password: effectivePassword });
      } else {
        await authStore.register({
          name: effectiveName,
          password: effectivePassword,
        });
      }
      resetForm();
      onClose();
    } catch (err: unknown) {
      if (err && typeof err === "object" && "apiStatus" in err) {
        const apiStatus = (err as { apiStatus: string }).apiStatus;
        if (apiStatus === "INVALID_CREDENTIALS") {
          errorMsg = "Invalid username or password.";
        } else if (apiStatus === "USER_EXISTS") {
          errorMsg = "A user with this username already exists.";
        } else {
          errorMsg = "Authentication failed. Please check your details.";
        }
      } else {
        errorMsg = "Connection error. Please ensure the backend is running.";
      }
    } finally {
      isSubmitting = false;
    }
  }

  function handleKeydown(event: KeyboardEvent): void {
    if (event.key === "Escape") {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if isOpen}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center p-4"
    role="dialog"
    aria-modal="true"
    aria-labelledby="auth-modal-title"
    tabindex="-1"
  >
    <!-- Accessible clickable backdrop button -->
    <button
      type="button"
      tabindex="-1"
      aria-label="Close dialog overlay"
      class="fixed inset-0 bg-black/60 backdrop-blur-sm transition-opacity"
      onclick={onClose}
    ></button>

    <div
      class="relative z-10 w-full max-w-md rounded-2xl border border-(--border-subtle) bg-(--bg-surface) p-6 shadow-2xl transition-all sm:p-8"
    >
      <!-- Modal Header -->
      <div class="flex items-center justify-between border-b border-(--border-subtle) pb-4">
        <div>
          <h2 id="auth-modal-title" class="text-lg font-bold tracking-tight text-(--text-primary)">
            {mode === "login" ? "Sign In to CoSave" : "Create an Account"}
          </h2>
          <p class="mt-1 text-xs text-(--text-secondary)">
            {mode === "login"
              ? "Access your family finance dashboard and accounts."
              : "Set up your credentials to start managing family finances."}
          </p>
        </div>
        <button
          type="button"
          onclick={onClose}
          aria-label="Close dialog"
          class="flex size-8 cursor-pointer items-center justify-center rounded-lg text-(--text-muted) transition-colors hover:bg-(--bg-hover) hover:text-(--text-primary)"
        >
          <svg
            class="size-4"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <path d="M18 6L6 18M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Mode Toggle Tabs -->
      <div class="mt-5 flex rounded-xl border border-(--border-subtle) bg-(--bg-hover) p-1">
        <button
          type="button"
          onclick={() => switchMode("login")}
          class="flex-1 rounded-lg py-1.5 text-xs font-semibold transition-all {mode === 'login'
            ? 'bg-(--bg-surface) text-(--text-primary) shadow-sm'
            : 'text-(--text-secondary) hover:text-(--text-primary)'}"
        >
          Sign In
        </button>
        <button
          type="button"
          onclick={() => switchMode("register")}
          class="flex-1 rounded-lg py-1.5 text-xs font-semibold transition-all {mode === 'register'
            ? 'bg-(--bg-surface) text-(--text-primary) shadow-sm'
            : 'text-(--text-secondary) hover:text-(--text-primary)'}"
        >
          Register
        </button>
      </div>

      <!-- Error Notification -->
      {#if errorMsg}
        <div
          class="mt-4 flex items-center gap-2 rounded-xl border border-rose-500/30 bg-rose-500/10 px-3.5 py-2.5 text-xs font-medium text-rose-400"
          role="alert"
        >
          <svg
            class="size-4 shrink-0"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
          >
            <circle cx="12" cy="12" r="10" />
            <line x1="12" y1="8" x2="12" y2="12" />
            <line x1="12" y1="16" x2="12.01" y2="16" />
          </svg>
          <span>{errorMsg}</span>
        </div>
      {/if}

      <!-- Form with explicit post method and credential autocomplete targets for Bitwarden & password managers -->
      <form method="post" action="#" onsubmit={handleSubmit} class="mt-5 space-y-4">
        <div>
          <label for="auth-username" class="block text-xs font-medium text-(--text-secondary)">
            Username
          </label>
          <Input
            id="auth-username"
            name="username"
            autocomplete="username"
            type="text"
            bind:value={name}
            onchange={(e) => (name = (e.currentTarget as HTMLInputElement).value)}
            required
            placeholder="e.g. alex"
            class="mt-1.5"
          />
        </div>

        <div>
          <label for="auth-password" class="block text-xs font-medium text-(--text-secondary)">
            Password
          </label>
          <div class="relative mt-1.5">
            <Input
              id="auth-password"
              name="password"
              autocomplete={mode === "login" ? "current-password" : "new-password"}
              type={showPassword ? "text" : "password"}
              bind:value={password}
              onchange={(e) => (password = (e.currentTarget as HTMLInputElement).value)}
              required
              minlength={6}
              placeholder="••••••••"
              class="pr-10"
            />
            <button
              type="button"
              onclick={() => (showPassword = !showPassword)}
              aria-label={showPassword ? "Hide password" : "Show password"}
              class="absolute inset-y-0 right-0 flex cursor-pointer items-center pr-3 text-(--text-muted) transition-colors hover:text-(--text-primary)"
              tabindex="-1"
            >
              {#if showPassword}
                <svg
                  class="size-4"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                >
                  <path
                    d="M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24"
                  />
                  <line x1="1" y1="1" x2="23" y2="23" />
                </svg>
              {:else}
                <svg
                  class="size-4"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  stroke-width="2"
                >
                  <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
                  <circle cx="12" cy="12" r="3" />
                </svg>
              {/if}
            </button>
          </div>
        </div>

        <Button type="submit" disabled={isSubmitting} class="mt-2 w-full">
          {#if isSubmitting}
            <span
              class="mr-2 inline-block size-3.5 animate-spin rounded-full border-2 border-white/20 border-t-white"
            ></span>
            Processing...
          {:else}
            {mode === "login" ? "Sign In" : "Create Account"}
          {/if}
        </Button>
      </form>
    </div>
  </div>
{/if}
