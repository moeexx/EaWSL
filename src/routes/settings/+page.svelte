<script lang="ts">
  import { getVersion } from "@tauri-apps/api/app";
  import { open } from "@tauri-apps/plugin-dialog";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import ExternalLink from "@lucide/svelte/icons/external-link";
  import Folder from "@lucide/svelte/icons/folder";
  import Languages from "@lucide/svelte/icons/languages";
  import ListChecks from "@lucide/svelte/icons/list-checks";
  import RotateCcw from "@lucide/svelte/icons/rotate-ccw";
  import Save from "@lucide/svelte/icons/save";
  import Timer from "@lucide/svelte/icons/timer";
  import { get } from "svelte/store";

  import Button from "$lib/ui/Button.svelte";
  import StatusNotice from "$lib/ui/StatusNotice.svelte";
  import {
    i18nState,
    isAppLanguage,
    setLanguage,
    type AppLanguage,
  } from "$lib/i18n";
  import { configureQueryCacheBackgroundRefresh } from "$lib/query-cache";
  import { hasTauriBridge, toErrorMessage } from "$lib/shared/runtime";
  import { openUrl } from "$lib/tauri/opener";
  import {
    createDefaultAppSettings,
    DEFAULT_INSTALL_LOCATION,
    MAX_BACKGROUND_REFRESH_INTERVAL_MINUTES,
    MIN_BACKGROUND_REFRESH_INTERVAL_MINUTES,
    normalizeAppSettings,
    orderBackgroundRefreshTargets,
    readAppSettingsOrDefault,
    resolveDefaultInstallLocation,
    saveAppSettings,
    type AppSettings,
    type BackgroundRefreshTarget,
  } from "$lib/tauri/settings";

  const DEFAULT_LANGUAGE: AppLanguage = "en-US";
  const FALLBACK_APP_VERSION = "0.3.0";
  const APP_LOGO_SRC = "/favicon.png";
  const REPOSITORY_URL = "https://github.com/moeexx/EaWSL";
  const ISSUE_URL = `${REPOSITORY_URL}/issues`;
  const currentYear = new Date().getFullYear();
  const languageOptions = [
    "en-US",
    "zh-CN",
  ] as const satisfies readonly AppLanguage[];
  const targetOptions = [
    "distros",
    "systemOverviewStorage",
    "wslVersion",
    "onlineDistros",
  ] as const satisfies readonly BackgroundRefreshTarget[];

  let draft = $state<AppSettings>(
    normalizeAppSettings(createDefaultAppSettings(DEFAULT_INSTALL_LOCATION)),
  );
  let draftLanguage = $state<AppLanguage>(get(i18nState).language);
  let loading = $state(false);
  let saving = $state(false);
  let saveError = $state<string | null>(null);
  let appVersion = $state(FALLBACK_APP_VERSION);
  let languageMenuOpen = $state(false);
  let disposed = false;

  const settingsCopy = $derived($i18nState.copy.settings);
  const commonCopy = $derived($i18nState.copy.common);
  const defaultInstallLocation = $derived(draft.defaultInstallLocation.trim());
  const errors = $derived({
    install: defaultInstallLocation
      ? null
      : settingsCopy.validation.defaultInstallLocationRequired,
    interval: getBackgroundIntervalError(
      draft.backgroundRefresh.intervalMinutes,
    ),
    targets: draft.backgroundRefresh.targets.length
      ? null
      : settingsCopy.validation.backgroundTargetsRequired,
  });
  const canSave = $derived(
    !loading &&
      !saving &&
      Object.values(errors).every((error) => error === null),
  );
  const aboutLinks = $derived([
    { label: settingsCopy.about.reportIssue, url: ISSUE_URL },
    { label: settingsCopy.about.repository, url: REPOSITORY_URL },
  ]);
  const selectedLanguageLabel = $derived(
    settingsCopy.language.options[draftLanguage],
  );

  const settingCardClass =
    "grid gap-3 rounded-[8px] border-[0.5px] border-shell-200/80 bg-white/90 p-4 sm:grid-cols-[40px_minmax(0,1fr)_minmax(280px,380px)] sm:items-center";
  const iconBoxClass =
    "flex h-10 w-10 items-center justify-center rounded-[8px] border-[0.5px] border-shell-200 bg-white text-shell-600";
  const labelClass = "text-[15px] font-semibold text-shell-950";
  const descriptionClass = "mt-1 text-[13px] leading-5 text-shell-500";
  const linkButtonClass =
    "flex min-h-[48px] w-full items-center justify-between gap-3 px-4 text-left text-[14px] font-medium text-shell-800 transition duration-150 hover:bg-accent-50/70 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-200";

  function inputClass(invalid = false): string {
    return `min-h-[36px] rounded-[8px] border-[0.5px] bg-white px-3 text-[14px] text-shell-900 transition duration-150 hover:border-accent-200 hover:bg-accent-50/70 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-200 disabled:cursor-not-allowed disabled:bg-shell-50 disabled:text-shell-500 disabled:hover:border-shell-200/80 disabled:hover:bg-shell-50 ${invalid ? "border-rose-200" : "border-shell-200/80"}`;
  }

  function clearError(): void {
    saveError = null;
  }

  function updateDraft(patch: Partial<AppSettings>): void {
    draft = { ...draft, ...patch };
    clearError();
  }

  function updateBackgroundRefresh(
    patch: Partial<AppSettings["backgroundRefresh"]>,
  ): void {
    updateDraft({
      backgroundRefresh: { ...draft.backgroundRefresh, ...patch },
    });
  }

  async function loadSettings(): Promise<void> {
    loading = true;
    clearError();
    const settings = await readAppSettingsOrDefault();
    if (disposed) return;
    draft = settings;
    draftLanguage = get(i18nState).language;
    loading = false;
  }

  async function loadAppVersion(): Promise<void> {
    if (!hasTauriBridge()) return;
    try {
      const version = await getVersion();
      if (!disposed) appVersion = version;
    } catch {
      if (!disposed) appVersion = FALLBACK_APP_VERSION;
    }
  }

  async function chooseDefaultInstallLocation(): Promise<void> {
    if (!hasTauriBridge()) {
      saveError = settingsCopy.errors.noTauriDirectoryPicker;
      return;
    }
    const selected = await open({
      title: settingsCopy.defaultInstallLocation.dialogTitle,
      directory: true,
      multiple: false,
      defaultPath: defaultInstallLocation || DEFAULT_INSTALL_LOCATION,
    });
    if (typeof selected === "string")
      updateDraft({ defaultInstallLocation: selected });
  }

  function setDraftLanguage(value: string): void {
    if (!isAppLanguage(value)) return;
    draftLanguage = value;
    languageMenuOpen = false;
    clearError();
  }

  function closeLanguageMenuOnFocusOut(event: FocusEvent): void {
    const currentTarget = event.currentTarget as HTMLElement;
    const nextTarget = event.relatedTarget;

    if (!(nextTarget instanceof Node) || !currentTarget.contains(nextTarget)) {
      languageMenuOpen = false;
    }
  }

  function setBackgroundTarget(
    target: BackgroundRefreshTarget,
    checked: boolean,
  ): void {
    const targets = checked
      ? [...new Set([...draft.backgroundRefresh.targets, target])]
      : draft.backgroundRefresh.targets.filter((item) => item !== target);
    updateBackgroundRefresh({
      targets: orderBackgroundRefreshTargets(targets),
    });
  }

  async function restoreDefaults(): Promise<void> {
    if (loading || saving) return;
    const installLocation = await resolveDefaultInstallLocation();
    if (disposed) return;
    draft = normalizeAppSettings(createDefaultAppSettings(installLocation));
    draftLanguage = DEFAULT_LANGUAGE;
    clearError();
  }

  async function saveSettings(): Promise<void> {
    const validationError = Object.values(errors).find(Boolean);
    if (validationError) {
      saveError = validationError;
      return;
    }
    if (!hasTauriBridge()) {
      saveError = settingsCopy.errors.noTauriSave;
      return;
    }
    saving = true;
    clearError();
    try {
      const settings = await saveAppSettings(draft);
      if (disposed) return;
      draft = settings;
      configureQueryCacheBackgroundRefresh(settings.backgroundRefresh);
      setLanguage(draftLanguage);
    } catch (error) {
      if (!disposed) saveError = toErrorMessage(error);
    } finally {
      if (!disposed) saving = false;
    }
  }

  async function openExternalLink(url: string): Promise<void> {
    try {
      if (hasTauriBridge()) {
        await openUrl(url);
        return;
      }
      if (typeof window !== "undefined")
        window.open(url, "_blank", "noopener,noreferrer");
    } catch (error) {
      if (!disposed) saveError = toErrorMessage(error);
    }
  }

  $effect(() => {
    disposed = false;
    void loadSettings();
    void loadAppVersion();
    return () => {
      disposed = true;
    };
  });

  function getBackgroundIntervalError(value: number): string | null {
    if (!Number.isInteger(value))
      return settingsCopy.validation.backgroundIntervalInteger;
    const min = MIN_BACKGROUND_REFRESH_INTERVAL_MINUTES;
    const max = MAX_BACKGROUND_REFRESH_INTERVAL_MINUTES;
    return value < min || value > max
      ? settingsCopy.validation.backgroundIntervalRange(min, max)
      : null;
  }
</script>

<div class="page-stack mx-auto w-full max-w-5xl">
  <header class="px-1">
    <h1 class="text-[1.6rem] font-semibold text-shell-950">
      {settingsCopy.page.title}
    </h1>
  </header>

  <section class="grid gap-3">
    <h2 class="px-1 text-[1rem] font-semibold text-shell-900">
      {settingsCopy.sections.basic}
    </h2>

    <div class="grid gap-2.5">
      <article class={settingCardClass}>
        <div class={iconBoxClass} aria-hidden="true">
          <Languages size={19} strokeWidth={1.9} />
        </div>
        <div class="min-w-0">
          <h3 class={labelClass}>{settingsCopy.language.label}</h3>
          <p class={descriptionClass}>{settingsCopy.language.description}</p>
        </div>
        <div class="relative" onfocusout={closeLanguageMenuOnFocusOut}>
          <button
            aria-expanded={languageMenuOpen}
            aria-haspopup="listbox"
            aria-label={settingsCopy.language.label}
            class={`${inputClass()} flex w-full items-center justify-between gap-3 text-left disabled:hover:border-shell-200/80 disabled:hover:bg-shell-50`}
            disabled={loading || saving}
            onclick={() => {
              languageMenuOpen = !languageMenuOpen;
            }}
            onkeydown={(event) => {
              if (event.key === "Escape") languageMenuOpen = false;
            }}
            type="button"
          >
            <span>{selectedLanguageLabel}</span>
            <ChevronDown
              class={`shrink-0 text-shell-500 transition duration-150 ${languageMenuOpen ? "rotate-180" : ""}`}
              size={17}
              strokeWidth={1.9}
            />
          </button>

          {#if languageMenuOpen}
            <div
              class="absolute left-0 right-0 z-20 mt-1 overflow-hidden rounded-[8px] border-[0.5px] border-shell-200/80 bg-white shadow-[0_10px_24px_rgba(17,26,39,0.12)]"
              role="listbox"
              tabindex="-1"
            >
              {#each languageOptions as language}
                <button
                  aria-selected={draftLanguage === language}
                  class={`flex min-h-[38px] w-full items-center px-3 text-left text-[14px] transition duration-150 hover:bg-accent-50/80 ${draftLanguage === language ? "bg-accent-50 text-accent-700" : "text-shell-800"}`}
                  onclick={() => setDraftLanguage(language)}
                  role="option"
                  type="button"
                >
                  {settingsCopy.language.options[language]}
                </button>
              {/each}
            </div>
          {/if}
        </div>
      </article>

      <article class={settingCardClass}>
        <div class={iconBoxClass} aria-hidden="true">
          <Folder size={19} strokeWidth={1.9} />
        </div>
        <div class="min-w-0">
          <h3 class={labelClass}>
            {settingsCopy.defaultInstallLocation.label}
          </h3>
          <p class={descriptionClass}>
            {settingsCopy.defaultInstallLocation.description}
          </p>
        </div>
        <div class="grid gap-1.5">
          <div class="flex min-w-0 flex-col gap-2 sm:flex-row">
            <input
              aria-invalid={errors.install !== null}
              aria-label={settingsCopy.defaultInstallLocation.label}
              class={`${inputClass(errors.install !== null)} min-w-0 flex-1`}
              disabled={loading || saving}
              oninput={(event) =>
                updateDraft({
                  defaultInstallLocation: event.currentTarget.value,
                })}
              placeholder={DEFAULT_INSTALL_LOCATION}
              type="text"
              value={draft.defaultInstallLocation}
            />
            <Button
              label={commonCopy.chooseDirectory}
              icon={Folder}
              variant="secondary"
              className="min-h-[36px] shrink-0 hover:border-accent-200 hover:bg-accent-50/70"
              disabled={loading || saving}
              onclick={() => void chooseDefaultInstallLocation()}
            />
          </div>
          {#if errors.install}<p class="text-[12px] leading-5 text-rose-700">
              {errors.install}
            </p>{/if}
        </div>
      </article>

      <article class={settingCardClass}>
        <div class={iconBoxClass} aria-hidden="true">
          <Timer size={19} strokeWidth={1.9} />
        </div>
        <div class="min-w-0">
          <h3 class={labelClass}>
            {settingsCopy.backgroundRefresh.intervalLabel}
          </h3>
          <p class={descriptionClass}>
            {settingsCopy.backgroundRefresh.intervalDescription}
          </p>
        </div>
        <div class="grid gap-1.5">
          <div class="relative">
            <input
              aria-invalid={errors.interval !== null}
              aria-label={settingsCopy.backgroundRefresh.intervalLabel}
              class={`${inputClass(errors.interval !== null)} w-full pr-14`}
              disabled={loading || saving}
              id="background-refresh-interval"
              max={MAX_BACKGROUND_REFRESH_INTERVAL_MINUTES}
              min={MIN_BACKGROUND_REFRESH_INTERVAL_MINUTES}
              oninput={(event) =>
                updateBackgroundRefresh({
                  intervalMinutes: Number(event.currentTarget.value),
                })}
              step="1"
              type="number"
              value={draft.backgroundRefresh.intervalMinutes}
            />
            <span
              class="pointer-events-none absolute inset-y-0 right-3 flex items-center text-[13px] font-medium text-shell-500"
            >
              {settingsCopy.backgroundRefresh.intervalUnit}
            </span>
          </div>
          {#if errors.interval}<p class="text-[12px] leading-5 text-rose-700">
              {errors.interval}
            </p>{/if}
        </div>
      </article>

      <article class={settingCardClass}>
        <div class={iconBoxClass} aria-hidden="true">
          <ListChecks size={19} strokeWidth={1.9} />
        </div>
        <div class="min-w-0">
          <h3 class={labelClass}>
            {settingsCopy.backgroundRefresh.targetsLabel}
          </h3>
          <p class={descriptionClass}>
            {settingsCopy.backgroundRefresh.targetsDescription}
          </p>
        </div>
        <div class="grid grid-cols-1 gap-2 sm:grid-cols-2">
          {#each targetOptions as target}
            <label
              class="inline-flex min-h-[34px] items-center gap-2 rounded-[8px] border-[0.5px] border-shell-200/80 bg-white px-3 py-1.5 text-[13px] leading-4 text-shell-700 transition duration-150 hover:border-accent-200 hover:bg-accent-50/70"
            >
              <input
                checked={draft.backgroundRefresh.targets.includes(target)}
                class="h-4 w-4 accent-[#1d78d7]"
                disabled={loading || saving}
                onchange={(event) =>
                  setBackgroundTarget(target, event.currentTarget.checked)}
                type="checkbox"
              />
              <span>{settingsCopy.backgroundRefresh.targets[target]}</span>
            </label>
          {/each}
          {#if errors.targets}<p class="text-[12px] leading-5 text-rose-700">
              {errors.targets}
            </p>{/if}
        </div>
      </article>
    </div>

    {#if saveError}
      <StatusNotice
        tone="error"
        title={settingsCopy.errors.saveFailed}
        message={saveError}
      />
    {/if}

    <div class="flex flex-wrap justify-end gap-2">
      <Button
        label={settingsCopy.actions.restoreDefaults}
        icon={RotateCcw}
        variant="secondary"
        size="lg"
        className="hover:border-accent-200 hover:bg-accent-50/70"
        disabled={loading || saving}
        onclick={() => void restoreDefaults()}
      />
      <Button
        label={saving
          ? settingsCopy.actions.saving
          : settingsCopy.actions.saveSettings}
        icon={Save}
        size="lg"
        disabled={!canSave}
        onclick={() => void saveSettings()}
      />
    </div>
  </section>

  <section class="mt-2 grid gap-3">
    <h2 class="px-1 text-[1rem] font-semibold text-shell-900">
      {settingsCopy.sections.about}
    </h2>
    <div
      class="overflow-hidden rounded-[8px] border-[0.5px] border-shell-200/80 bg-white/90"
    >
      <div
        class="grid grid-cols-[44px_minmax(0,1fr)] items-center gap-3 px-4 py-3.5"
      >
        <img
          alt=""
          class="h-11 w-11 rounded-[8px] border-[0.5px] border-shell-200 bg-white object-contain p-1"
          src={APP_LOGO_SRC}
        />
        <span class="min-w-0">
          <span class="block text-[16px] font-semibold text-shell-950"
            >{settingsCopy.about.appName}</span
          >
          <span class="mt-1 block text-[13px] leading-5 text-shell-500"
            >{settingsCopy.about.meta(currentYear, appVersion)}</span
          >
        </span>
      </div>
      <div class="border-t-[0.5px] border-shell-200/80">
        {#each aboutLinks as link, index}
          <button
            class={`${linkButtonClass} ${index > 0 ? "border-t-[0.5px] border-shell-200/80" : ""}`}
            onclick={() => void openExternalLink(link.url)}
            type="button"
          >
            <span>{link.label}</span>
            <ExternalLink class="text-shell-500" size={18} strokeWidth={1.9} />
          </button>
        {/each}
      </div>
    </div>
  </section>
</div>
