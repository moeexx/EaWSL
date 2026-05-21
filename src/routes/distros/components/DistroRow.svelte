<script lang="ts">
  import BadgeCheck from "@lucide/svelte/icons/badge-check";
  import ChevronDown from "@lucide/svelte/icons/chevron-down";
  import ChevronUp from "@lucide/svelte/icons/chevron-up";
  import FileOutput from "@lucide/svelte/icons/file-output";
  import Layers3 from "@lucide/svelte/icons/layers-3";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import Square from "@lucide/svelte/icons/square";
  import Star from "@lucide/svelte/icons/star";
  import Tag from "@lucide/svelte/icons/tag";
  import Trash2 from "@lucide/svelte/icons/trash-2";
  import { slide } from "svelte/transition";

  import { i18nState } from "$lib/i18n";
  import { longTaskState } from "$lib/long-tasks";
  import { hasTauriBridge } from "$lib/shared/runtime";
  import Button from "$lib/ui/Button.svelte";
  import DistroLogo from "$lib/ui/DistroLogo.svelte";
  import PathPickerField from "$lib/ui/PathPickerField.svelte";
  import RefreshButton from "$lib/ui/RefreshButton.svelte";

  import {
    getExportFileNameError,
    getExportTargetFile,
  } from "../view-model/distro-export-rules";
  import type {
    DistroRowView,
    DistroWorkspaceCallbacks,
    DistroExportFormatOption,
  } from "../view-model/distro-workspace-types";

  type Props = {
    row: DistroRowView;
    callbacks: DistroWorkspaceCallbacks;
  };

  let { row, callbacks }: Props = $props();
  const rowCopy = $derived($i18nState.copy.distros.row);
  const commonCopy = $derived($i18nState.copy.common);
  const tauriAvailable = hasTauriBridge();
  let exportFileName = $state("");
  let exportDirectory = $state("");
  let exportFormat: DistroExportFormatOption["format"] = $state("Tar");
  let exportOpen = $state(false);
  let exportSubmitting = $state(false);
  let hasActiveLongTask = $state(false);
  let exportFileNameTouched = false;
  let exportDirectoryTouched = false;

  const rowButtonClass =
    "border-shell-200 bg-white text-shell-700 hover:border-shell-300 hover:bg-shell-50";
  const rowExportButtonClass =
    "border-sky-200 bg-sky-50 text-sky-700 hover:border-sky-300 hover:bg-sky-100";
  const rowDangerButtonClass =
    "border-rose-200 bg-rose-50/75 text-rose-700 hover:border-rose-300 hover:bg-rose-100";
  const badgeBaseClass =
    "inline-flex max-w-full items-center gap-1.5 rounded-full border px-3 py-1 text-[12px] font-semibold leading-none shadow-[0_1px_2px_rgba(15,23,42,0.04)]";
  const defaultBadgeClass = `${badgeBaseClass} border-accent-200 bg-accent-50 text-accent-700`;
  const versionBadgeClass = `${badgeBaseClass} border-shell-200 bg-shell-50 text-shell-700`;
  const flavorBadgeClass =
    "inline-flex min-w-0 max-w-full items-center gap-1.5 rounded-full border border-shell-200/80 bg-shell-50/80 px-2.5 py-1 text-[11.5px] font-medium leading-none text-shell-600";
  const detailValueClass = "text-[13px] leading-5 text-shell-800";
  const detailValueStrongClass =
    "text-[13px] font-medium leading-5 text-shell-900";
  const selectClass =
    "min-h-[36px] rounded-[10px] border-[0.5px] border-shell-200/80 bg-white px-3 text-[14px] text-shell-900 transition duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-200";
  const exportTargetPathClass =
    "min-w-0 truncate rounded-[8px] border border-shell-150 bg-shell-50 px-3 py-2 font-mono text-[12px] leading-5 text-shell-700";
  const fallbackExportFormat: DistroExportFormatOption = {
    label: ".tar",
    suffix: ".tar",
    format: "Tar",
  };

  const selectedFormat = $derived(
    row.exportMenu.formats.find((option) => option.format === exportFormat) ??
      fallbackExportFormat,
  );
  const exportFileNameError = $derived(
    getExportFileNameError(exportFileName, row.exportMenu.errors),
  );
  const exportDirectoryError = $derived(
    !tauriAvailable
      ? row.exportMenu.errors.noTauriDirectoryPicker
      : exportDirectory.trim().length === 0
        ? row.exportMenu.errors.directoryRequired
        : null,
  );
  const exportTargetFile = $derived(
    getExportTargetFile(exportDirectory, exportFileName, selectedFormat),
  );
  const exportToggleIcon = $derived(exportOpen ? ChevronUp : FileOutput);
  const detailToggleIcon = $derived(row.expanded ? ChevronUp : ChevronDown);
  const exportSubmitDisabled = $derived(
    row.actionsDisabled ||
      hasActiveLongTask ||
      exportSubmitting ||
      exportFileNameError !== null ||
      exportDirectoryError !== null ||
      exportTargetFile === null,
  );

  $effect(() => {
    const unsubscribe = longTaskState.subscribe((state) => {
      hasActiveLongTask = state.hasActiveLongTask;
    });

    return unsubscribe;
  });

  $effect(() => {
    if (row.expanded) {
      exportOpen = false;
    }
  });

  $effect(() => {
    if (row.isProtected) {
      exportOpen = false;
    }
  });

  $effect(() => {
    if (!exportOpen) {
      return;
    }

    if (!exportFileNameTouched) {
      exportFileName = row.exportMenu.defaultFileName;
    }

    if (!exportDirectoryTouched) {
      exportDirectory = row.exportMenu.defaultDirectory;
    }
  });

  async function toggleExportMenu(): Promise<void> {
    if (exportOpen) {
      exportOpen = false;
      return;
    }

    if (row.expanded) {
      await callbacks.toggleExpanded(row.name);
    }

    exportOpen = true;
  }

  async function chooseExportDirectory(): Promise<void> {
    if (!tauriAvailable) {
      return;
    }

    const defaultPath =
      exportDirectory.trim().length > 0 ? exportDirectory : undefined;
    const selected = await callbacks.chooseExportDirectory(
      row.name,
      defaultPath,
    );

    if (selected !== null) {
      exportDirectoryTouched = true;
      exportDirectory = selected;
    }
  }

  async function submitExport(): Promise<void> {
    if (exportSubmitDisabled || exportTargetFile === null) {
      return;
    }

    exportSubmitting = true;

    try {
      await callbacks.submitExport(
        row.name,
        exportTargetFile,
        selectedFormat.format,
        row.logoSrc,
      );
    } finally {
      exportSubmitting = false;
    }
  }

  function getDetailClass(
    variant: DistroRowView["details"][number]["variant"],
  ): string {
    if (variant === "strong") {
      return detailValueStrongClass;
    }

    if (variant === "mono") {
      return `${detailValueClass} break-all font-mono`;
    }

    return detailValueClass;
  }
</script>

<li
  class="rounded-[10px] border border-shell-200/85 bg-white/[0.9] px-4 py-3 shadow-[0_10px_24px_rgba(15,23,42,0.04)]"
>
  <div class="flex flex-col gap-3">
    <div
      class="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between"
    >
      <div class="flex min-w-0 items-center gap-3">
        <DistroLogo src={row.logoSrc} />

        <div class="min-w-0 flex-1">
          <div class="flex flex-wrap items-center gap-x-3 gap-y-1.5">
            <strong
              class="min-w-0 max-w-full truncate text-[18px] font-semibold leading-none tracking-[-0.03em] text-shell-950"
            >
              {row.name}
            </strong>
            <span
              class={`inline-flex items-center gap-1.5 text-[12.5px] font-medium leading-none ${row.state.textClass}`}
            >
              <span
                aria-hidden="true"
                class={`h-2 w-2 rounded-full ${row.state.dotClass}`}
              ></span>
              {row.state.label}
            </span>
          </div>
        </div>
      </div>

      <div class="flex min-w-0 flex-wrap items-center gap-2">
        {#if row.isDefault}
          <span class={defaultBadgeClass}>
            <BadgeCheck size={13} strokeWidth={2.25} />
            <span>{rowCopy.defaultBadge}</span>
          </span>
        {/if}

        {#if row.flavorVersion}
          <span class={flavorBadgeClass} title={row.flavorVersion}>
            <Tag
              aria-hidden="true"
              size={12}
              strokeWidth={2}
              class="shrink-0 text-shell-500"
            />
            <span
              class={`truncate ${
                row.flavorVersionCompact ? "max-w-none" : "max-w-[14rem]"
              }`}
            >
              {row.flavorVersion}
            </span>
          </span>
        {/if}

        <span class={versionBadgeClass}>
          <Layers3 size={13} strokeWidth={2.1} />
          <span>{row.versionLabel}</span>
        </span>
      </div>
    </div>

    <div
      class="flex flex-col gap-3 border-t border-shell-150/90 pt-3 sm:flex-row sm:items-center sm:justify-between"
    >
      {#if !row.isProtected}
        <div class="flex min-w-0 flex-wrap gap-2">
          <RefreshButton
            label={commonCopy.stop}
            icon={Square}
            variant="secondary"
            refreshing={row.terminateRunning}
            refreshingLabel={commonCopy.stopping}
            className={rowButtonClass}
            onclick={() => void callbacks.terminate(row.name)}
            disabled={row.actionsDisabled || hasActiveLongTask}
          />
          <RefreshButton
            label={rowCopy.delete}
            icon={Trash2}
            refreshIcon={RefreshCw}
            variant="danger"
            refreshing={row.unregisterBusy}
            refreshingLabel={row.deleteLabel}
            className={rowDangerButtonClass}
            onclick={() => void callbacks.unregister(row.name)}
            disabled={row.actionsDisabled || row.isDefault || hasActiveLongTask}
          />
          <RefreshButton
            label={rowCopy.setDefault}
            icon={Star}
            variant="secondary"
            refreshing={row.settingDefault}
            refreshingLabel={rowCopy.settingDefault}
            className={rowButtonClass}
            onclick={() => void callbacks.setDefault(row.name)}
            disabled={row.actionsDisabled || row.isDefault || hasActiveLongTask}
          />
        </div>
      {:else if row.protectedMessage}
        <span class="text-[12px] font-medium text-shell-500">
          {row.protectedMessage}
        </span>
      {/if}

      <div
        class="flex min-w-0 flex-wrap items-center gap-2 self-start sm:ml-auto sm:self-auto"
      >
        {#if !row.isProtected}
          <Button
            label={exportOpen ? rowCopy.collapseExportMenu : row.exportLabel}
            icon={exportToggleIcon}
            variant="secondary"
            className={rowExportButtonClass}
            ariaControls={row.panelId}
            ariaExpanded={exportOpen}
            onclick={() => void toggleExportMenu()}
            disabled={row.actionsDisabled}
          />
        {/if}

        <Button
          label={row.expandLabel}
          icon={detailToggleIcon}
          variant="secondary"
          className={rowButtonClass}
          ariaControls={row.panelId}
          ariaExpanded={row.expanded}
          onclick={() => void callbacks.toggleExpanded(row.name)}
          disabled={row.actionsDisabled}
        />
      </div>
    </div>

    {#if row.expanded || (exportOpen && !row.isProtected)}
      <section
        id={row.panelId}
        class="pt-2"
        transition:slide={{ duration: 150 }}
      >
        {#if row.expanded}
          <div class="mb-2 flex items-center justify-between gap-3">
            <h3
              class="text-[13px] font-semibold tracking-[-0.02em] text-shell-900"
            >
              {rowCopy.moreInfo}
            </h3>
            <span class="text-[11px] font-medium text-shell-500">
              {rowCopy.detailsTitle}
            </span>
          </div>

          <dl class="grid grid-cols-2 gap-2">
            {#each row.details as detail (detail.key)}
              <div
                class={`grid gap-0.5 ${detail.variant === "mono" ? "col-span-2" : ""}`}
              >
                <dt class="text-[12px] text-shell-500">{detail.label}</dt>
                <dd class={getDetailClass(detail.variant)}>
                  {detail.value}
                </dd>
              </div>
            {/each}
          </dl>
        {:else if !row.isProtected}
          <div class="mb-2 flex items-center justify-between gap-3">
            <h3
              class="text-[13px] font-semibold tracking-[-0.02em] text-shell-900"
            >
              {row.exportMenu.title}
            </h3>
          </div>

          <div
            class="grid gap-3 rounded-[10px] border border-shell-150 bg-shell-50/45 p-3"
          >
            <div class="grid gap-3 md:grid-cols-[minmax(0,1fr)_160px]">
              <div class="grid gap-1.5">
                <label
                  class="text-[14px] font-medium text-shell-600"
                  for={`${row.panelId}-export-file-name`}
                >
                  {row.exportMenu.fileNameLabel}<span class="text-rose-600"
                    >*</span
                  >
                </label>
                <input
                  id={`${row.panelId}-export-file-name`}
                  class={`${selectClass} ${exportFileNameError ? "border-rose-300 focus-visible:ring-rose-200" : ""}`}
                  aria-invalid={exportFileNameError !== null}
                  aria-describedby={exportFileNameError
                    ? `${row.panelId}-export-file-name-error`
                    : undefined}
                  disabled={exportSubmitting}
                  placeholder={row.exportMenu.fileNamePlaceholder}
                  type="text"
                  value={exportFileName}
                  oninput={(event) => {
                    exportFileNameTouched = true;
                    exportFileName = event.currentTarget.value;
                  }}
                />
                {#if exportFileNameError}
                  <p
                    id={`${row.panelId}-export-file-name-error`}
                    class="text-[12px] font-medium text-rose-600"
                  >
                    {exportFileNameError}
                  </p>
                {/if}
              </div>

              <div class="grid gap-1.5">
                <label
                  class="text-[14px] font-medium text-shell-600"
                  for={`${row.panelId}-export-format`}
                >
                  {row.exportMenu.formatLabel}<span class="text-rose-600"
                    >*</span
                  >
                </label>
                <select
                  id={`${row.panelId}-export-format`}
                  class={selectClass}
                  disabled={exportSubmitting}
                  value={exportFormat}
                  onchange={(event) => {
                    exportFormat = event.currentTarget
                      .value as DistroExportFormatOption["format"];
                  }}
                >
                  {#each row.exportMenu.formats as option (option.format)}
                    <option value={option.format}>{option.label}</option>
                  {/each}
                </select>
              </div>
            </div>

            <PathPickerField
              id={`${row.panelId}-export-directory`}
              label={row.exportMenu.directoryLabel}
              required
              value={exportDirectory}
              error={exportDirectoryError}
              placeholder={row.exportMenu.directoryPlaceholder}
              chooseLabel={commonCopy.chooseDirectory}
              disabled={exportSubmitting}
              chooseDisabled={!tauriAvailable}
              oninput={(value) => {
                exportDirectoryTouched = true;
                exportDirectory = value;
              }}
              onchoose={() => void chooseExportDirectory()}
            >
              {#snippet after()}
                {#if exportTargetFile}
                  <p class={exportTargetPathClass} title={exportTargetFile}>
                    {exportTargetFile}
                  </p>
                {/if}
              {/snippet}
            </PathPickerField>

            <Button
              label={exportSubmitting
                ? row.exportMenu.exporting
                : row.exportMenu.submit}
              icon={FileOutput}
              iconStrokeWidth={2.1}
              size="lg"
              className="justify-self-start"
              disabled={exportSubmitDisabled}
              onclick={() => void submitExport()}
            />
          </div>
        {/if}
      </section>
    {/if}
  </div>
</li>
