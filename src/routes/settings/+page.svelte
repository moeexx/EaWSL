<script lang="ts">
  import { open } from "@tauri-apps/plugin-dialog";
  import { untrack } from "svelte";
  import { get } from "svelte/store";

  import Button from "$lib/ui/Button.svelte";
  import PageHeader from "$lib/ui/PageHeader.svelte";
  import PathPickerField from "$lib/ui/PathPickerField.svelte";
  import SectionPanel from "$lib/ui/SectionPanel.svelte";
  import StatusNotice from "$lib/ui/StatusNotice.svelte";
  import {
    i18nState,
    isAppLanguage,
    setLanguage,
    SUPPORTED_LANGUAGES,
    type AppLanguage,
  } from "$lib/i18n";
  import { configureQueryCacheBackgroundRefresh } from "$lib/query-cache";
  import { hasTauriBridge, toErrorMessage } from "$lib/shared/runtime";
  import {
    DEFAULT_APP_SETTINGS,
    DEFAULT_INSTALL_LOCATION,
    MAX_BACKGROUND_REFRESH_INTERVAL_MINUTES,
    MIN_BACKGROUND_REFRESH_INTERVAL_MINUTES,
    normalizeAppSettings,
    orderBackgroundRefreshTargets,
    readAppSettingsOrDefault,
    saveAppSettings,
    type AppSettings,
    type BackgroundRefreshTarget,
  } from "$lib/tauri/settings";

  const targetOptions = [
    { target: "distros" },
    { target: "systemOverviewStorage" },
    { target: "wslVersion" },
    { target: "onlineDistros" },
  ] satisfies readonly {
    target: BackgroundRefreshTarget;
  }[];

  let draft = $state<AppSettings>(normalizeAppSettings(DEFAULT_APP_SETTINGS));
  let draftLanguage = $state<AppLanguage>(get(i18nState).language);
  let loading = $state(false);
  let saving = $state(false);
  let saveError = $state<string | null>(null);
  let disposed = false;

  const settingLabelClass = "text-[15px] font-semibold text-shell-950";
  const settingsCopy = $derived($i18nState.copy.settings);
  const defaultInstallLocation = $derived(draft.defaultInstallLocation.trim());
  const defaultInstallLocationError = $derived(
    defaultInstallLocation.length === 0
      ? settingsCopy.validation.defaultInstallLocationRequired
      : null,
  );
  const backgroundIntervalError = $derived(
    getBackgroundIntervalError(draft.backgroundRefresh.intervalMinutes),
  );
  const backgroundTargetsError = $derived(
    draft.backgroundRefresh.targets.length === 0
      ? settingsCopy.validation.backgroundTargetsRequired
      : null,
  );
  const commonCopy = $derived($i18nState.copy.common);
  const canSave = $derived(
    !loading &&
      !saving &&
      defaultInstallLocationError === null &&
      backgroundIntervalError === null &&
      backgroundTargetsError === null,
  );

  function inputClass(invalid: boolean): string {
    return `min-h-[36px] rounded-[10px] border-[0.5px] bg-white px-3 text-[14px] text-shell-900 transition duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-200 disabled:cursor-not-allowed disabled:bg-shell-50 disabled:text-shell-500 ${
      invalid ? "border-rose-200" : "border-shell-200/80"
    }`;
  }

  async function loadSettings(): Promise<void> {
    loading = true;
    saveError = null;

    const settings = await readAppSettingsOrDefault();
    if (disposed) return;

    draft = settings;
    draftLanguage = get(i18nState).language;
    loading = false;
  }

  function updateDefaultInstallLocation(value: string): void {
    draft = { ...draft, defaultInstallLocation: value };
    saveError = null;
  }

  function updateDraftLanguage(value: string): void {
    if (isAppLanguage(value)) {
      draftLanguage = value;
      saveError = null;
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

    if (typeof selected === "string") {
      updateDefaultInstallLocation(selected);
    }
  }

  function updateBackgroundInterval(value: string): void {
    draft = {
      ...draft,
      backgroundRefresh: {
        ...draft.backgroundRefresh,
        intervalMinutes: Number(value),
      },
    };
    saveError = null;
  }

  function toggleBackgroundTarget(
    target: BackgroundRefreshTarget,
    checked: boolean,
  ): void {
    const targets = checked
      ? [...draft.backgroundRefresh.targets, target]
      : draft.backgroundRefresh.targets.filter((item) => item !== target);

    draft = {
      ...draft,
      backgroundRefresh: {
        ...draft.backgroundRefresh,
        targets: orderBackgroundRefreshTargets(targets),
      },
    };
    saveError = null;
  }

  async function saveSettings(): Promise<void> {
    const validationError =
      defaultInstallLocationError ??
      backgroundIntervalError ??
      backgroundTargetsError;

    if (validationError) {
      saveError = validationError;
      return;
    }

    if (!hasTauriBridge()) {
      saveError = settingsCopy.errors.noTauriSave;
      return;
    }

    saving = true;
    saveError = null;
    const nextLanguage = draftLanguage;

    try {
      const settings = await saveAppSettings(draft);
      if (disposed) return;

      draft = settings;
      configureQueryCacheBackgroundRefresh(settings.backgroundRefresh);
      setLanguage(nextLanguage);
    } catch (error) {
      if (!disposed) saveError = toErrorMessage(error);
    } finally {
      if (!disposed) saving = false;
    }
  }

  $effect(() =>
    untrack(() => {
      disposed = false;
      void loadSettings();

      return () => {
        disposed = true;
      };
    }),
  );

  function getBackgroundIntervalError(value: number): string | null {
    if (!Number.isInteger(value)) {
      return settingsCopy.validation.backgroundIntervalInteger;
    }

    if (
      value < MIN_BACKGROUND_REFRESH_INTERVAL_MINUTES ||
      value > MAX_BACKGROUND_REFRESH_INTERVAL_MINUTES
    ) {
      return settingsCopy.validation.backgroundIntervalRange(
        MIN_BACKGROUND_REFRESH_INTERVAL_MINUTES,
        MAX_BACKGROUND_REFRESH_INTERVAL_MINUTES,
      );
    }

    return null;
  }
</script>

<div class="page-stack">
  <PageHeader
    eyebrow={settingsCopy.page.eyebrow}
    title={settingsCopy.page.title}
  />

  <SectionPanel title="">
    <div class="grid gap-4">
      <label class="grid gap-1.5 sm:max-w-[260px]">
        <span class={settingLabelClass}>
          {settingsCopy.language.label}
        </span>
        <select
          class={inputClass(false)}
          disabled={loading || saving}
          onchange={(event) => {
            updateDraftLanguage(event.currentTarget.value);
          }}
          value={draftLanguage}
        >
          {#each SUPPORTED_LANGUAGES as language}
            <option value={language}>
              {settingsCopy.language.options[language]}
            </option>
          {/each}
        </select>
      </label>

      <PathPickerField
        id="default-install-location"
        label={settingsCopy.defaultInstallLocation.label}
        value={draft.defaultInstallLocation}
        error={defaultInstallLocationError}
        placeholder={DEFAULT_INSTALL_LOCATION}
        chooseLabel={commonCopy.chooseDirectory}
        chooseIcon={null}
        disabled={loading || saving}
        labelClassName={settingLabelClass}
        oninput={updateDefaultInstallLocation}
        onchoose={() => void chooseDefaultInstallLocation()}
      />

      <div class="grid gap-3">
        <div class="grid gap-1.5 sm:max-w-[260px]">
          <label
            class={settingLabelClass}
            for="background-refresh-interval"
          >
            {settingsCopy.backgroundRefresh.intervalLabel}
          </label>
          <input
            aria-invalid={backgroundIntervalError !== null}
            class={inputClass(backgroundIntervalError !== null)}
            disabled={loading || saving}
            id="background-refresh-interval"
            max={MAX_BACKGROUND_REFRESH_INTERVAL_MINUTES}
            min={MIN_BACKGROUND_REFRESH_INTERVAL_MINUTES}
            oninput={(event) => {
              updateBackgroundInterval(event.currentTarget.value);
            }}
            step="1"
            type="number"
            value={draft.backgroundRefresh.intervalMinutes}
          />
          {#if backgroundIntervalError}
            <p class="text-[12px] leading-5 text-rose-700">
              {backgroundIntervalError}
            </p>
          {/if}
        </div>

        <div class="grid gap-2">
          <div class={settingLabelClass}>
            {settingsCopy.backgroundRefresh.targetsLabel}
          </div>

          <div class="grid gap-2 sm:grid-cols-2 xl:grid-cols-4">
            {#each targetOptions as option}
              <label
                class="inline-flex min-h-[38px] items-center gap-2 rounded-[10px] border-[0.5px] border-shell-200/80 bg-white px-3 py-2 text-[13px] text-shell-700"
              >
                <input
                  checked={draft.backgroundRefresh.targets.includes(
                    option.target,
                  )}
                  class="h-4 w-4 accent-[#1d78d7]"
                  disabled={loading || saving}
                  onchange={(event) => {
                    toggleBackgroundTarget(
                      option.target,
                      event.currentTarget.checked,
                    );
                  }}
                  type="checkbox"
                />
                <span>
                  {settingsCopy.backgroundRefresh.targets[option.target]}
                </span>
              </label>
            {/each}
          </div>

          {#if backgroundTargetsError}
            <p class="text-[12px] leading-5 text-rose-700">
              {backgroundTargetsError}
            </p>
          {/if}
        </div>
      </div>

      {#if saveError}
        <StatusNotice
          tone="error"
          title={settingsCopy.errors.saveFailed}
          message={saveError}
        />
      {/if}

      <div class="flex justify-start">
        <Button
          label={saving ? settingsCopy.actions.saving : settingsCopy.actions.saveSettings}
          className="min-h-[36px]"
          disabled={!canSave}
          onclick={() => void saveSettings()}
        />
      </div>
    </div>
  </SectionPanel>
</div>
