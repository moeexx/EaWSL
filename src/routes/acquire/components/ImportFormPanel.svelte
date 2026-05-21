<script lang="ts">
  import Archive from "@lucide/svelte/icons/archive";
  import Download from "@lucide/svelte/icons/download";
  import FileUp from "@lucide/svelte/icons/file-up";
  import HardDrive from "@lucide/svelte/icons/hard-drive";
  import { i18nState } from "$lib/i18n";
  import Button from "$lib/ui/Button.svelte";
  import PathPickerField from "$lib/ui/PathPickerField.svelte";
  import type { AcquireWorkspaceViewModel } from "../view-model/workspace.svelte";
  import AcquireSpaceNotice from "./AcquireSpaceNotice.svelte";
  import AcquireTextField from "./AcquireTextField.svelte";

  type Props = { model: AcquireWorkspaceViewModel };
  let { model }: Props = $props();
  const copy = $derived($i18nState.copy.acquire.importForm);
  const commonCopy = $derived($i18nState.copy.common);
  const draft = $derived(model.importDraft);
  const validation = $derived(model.importValidation);
  const isArchive = $derived(model.detectedImportKind === "archive");
  const isVhdx = $derived(model.detectedImportKind === "vhdx");
</script>

<div class="grid min-w-0 gap-3">
  <div
    class="flex items-center gap-3 rounded-[8px] border-[0.5px] border-shell-200/80 bg-white px-3.5 py-3"
  >
    <div
      class="flex h-10 w-10 shrink-0 items-center justify-center rounded-[8px] border-[0.5px] border-shell-200 bg-shell-50 text-shell-600"
    >
      {#if isArchive}<Archive
          size={17}
          strokeWidth={1.95}
        />{:else if isVhdx}<HardDrive
          size={17}
          strokeWidth={1.95}
        />{:else}<FileUp size={17} strokeWidth={1.95} />{/if}
    </div>
    <div class="min-w-0 flex-1">
      <p class="text-[12px] font-medium text-shell-500">{copy.currentMode}</p>
      <strong class="mt-1 block truncate text-[15px] leading-5 text-shell-950"
        >{model.importNoun}</strong
      >
    </div>
  </div>

  <AcquireTextField
    id="import-name"
    label={copy.nameLabel}
    required
    value={draft.name}
    invalid={validation.nameError !== null}
    error={validation.nameError}
    placeholder={copy.namePlaceholder}
    oninput={model.callbacks.updateImportName}
  />
  <PathPickerField
    id="import-file"
    label={copy.fileLabel}
    required
    value={draft.file}
    error={validation.fileError}
    placeholder={copy.filePlaceholder}
    chooseLabel={copy.chooseFile}
    chooseIcon={FileUp}
    oninput={model.callbacks.updateImportFile}
    onchoose={() => void model.callbacks.chooseImportFile()}
  />
  <PathPickerField
    id="import-root"
    label={copy.directoryLabel}
    required
    value={draft.location}
    error={validation.locationError}
    placeholder={copy.directoryPlaceholder}
    chooseLabel={commonCopy.chooseDirectory}
    oninput={model.callbacks.updateImportRoot}
    onchoose={() => void model.callbacks.chooseImportRoot()}
  >
    {#snippet after()}
      {#if model.importSpaceNotice}<AcquireSpaceNotice
          tone={model.importSpaceNotice.tone}
          message={model.importSpaceNotice.message}
        />{/if}
    {/snippet}
  </PathPickerField>
  <Button
    label={model.importSubmitting
      ? copy.importing(model.importNoun)
      : copy.start(model.importNoun)}
    icon={Download}
    iconStrokeWidth={2.1}
    size="lg"
    className="w-full"
    disabled={model.importSubmitDisabled}
    onclick={() => void model.callbacks.startImport()}
  />
</div>
