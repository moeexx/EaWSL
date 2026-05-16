<script lang="ts">
  import Check from "@lucide/svelte/icons/check";
  import Download from "@lucide/svelte/icons/download";
  import HardDrive from "@lucide/svelte/icons/hard-drive";
  import { i18nState } from "$lib/i18n";
  import Button from "$lib/ui/Button.svelte";
  import PathPickerField from "$lib/ui/PathPickerField.svelte";
  import type { AcquireWorkspaceViewModel } from "../view-model/workspace.svelte";
  import AcquireSpaceNotice from "./AcquireSpaceNotice.svelte";
  import AcquireTextField from "./AcquireTextField.svelte";

  type Props = { model: AcquireWorkspaceViewModel };
  let { model }: Props = $props();
  const copy = $derived($i18nState.copy.acquire.install);
  const commonCopy = $derived($i18nState.copy.common);
  const fieldDisabled = $derived(model.selectedDistro === null);
  const vhdClass = $derived(`min-h-[36px] w-full rounded-[10px] border-[0.5px] bg-white px-3 pr-12 text-[14px] text-shell-900 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-200 ${model.validation.vhdSizeError ? "border-rose-200" : "border-shell-200/80"} ${fieldDisabled ? "cursor-not-allowed opacity-60" : ""}`);
</script>

<div class="min-w-0 grid gap-3">
  {#if model.selectedDistro}
    <div class="flex items-center gap-3 rounded-[10px] border-[0.5px] border-shell-200/80 bg-white px-3.5 py-3">
      <img alt="" class="h-10 w-10 shrink-0 rounded-[9px] border-[0.5px] border-shell-200 bg-shell-50 object-contain p-1.5" src={model.selectedDistro.logoSrc} />
      <div class="min-w-0 flex-1"><p class="text-[12px] font-medium text-shell-500">{copy.selectedDistro}</p><strong class="mt-1 block truncate text-[15px] leading-5 text-shell-950">{model.selectedDistro.friendly_name}</strong><p class="mt-0.5 truncate text-[12px] leading-4 text-shell-500">{model.selectedDistro.name}</p></div>
      <div aria-hidden="true" class="flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-accent-700 text-white"><Check size={16} strokeWidth={2.4} /></div>
    </div>
  {/if}

  <AcquireTextField id="install-name" label={copy.nameLabel} required value={model.draft.name} invalid={model.validation.nameError !== null} error={model.validation.nameError} placeholder={copy.namePlaceholder} disabled={fieldDisabled} oninput={model.callbacks.updateName} />
  <PathPickerField id="install-location" label={copy.locationLabel} required value={model.draft.location} error={model.validation.locationError} placeholder={copy.locationPlaceholder} chooseLabel={commonCopy.chooseDirectory} disabled={fieldDisabled} oninput={model.callbacks.updateLocation} onchoose={() => void model.callbacks.chooseLocation()}>
    {#snippet after()}
      {#if model.spaceNotice}<AcquireSpaceNotice tone={model.spaceNotice.tone} message={model.spaceNotice.message} />{/if}
    {/snippet}
  </PathPickerField>

  <div class="rounded-[10px] border-[0.5px] border-shell-200/80 bg-white px-3.5 py-3">
    <div class="flex items-center justify-between gap-3"><strong class="text-[15px] font-semibold text-shell-950">{copy.vhdOptions}</strong><HardDrive size={17} strokeWidth={1.9} class="text-shell-600" /></div>
    <div class="mt-3 grid gap-3">
      <div class="grid gap-1.5">
        <label class="text-[14px] font-medium text-shell-600" for="install-vhd-size">{copy.diskSize}{#if model.draft.fixedVhd}<span class="text-rose-600">*</span>{/if}</label>
        <div class="relative">
          <input id="install-vhd-size" aria-invalid={model.validation.vhdSizeError !== null} class={vhdClass} disabled={fieldDisabled} inputmode="numeric" min={15} oninput={(event) => { const normalized = event.currentTarget.value.replace(/\D/g, ""); if (event.currentTarget.value !== normalized) event.currentTarget.value = normalized; model.callbacks.updateVhdSize(normalized); }} placeholder="20" required={model.draft.fixedVhd} step={1} type="number" value={model.draft.vhdSize} />
          <span class="pointer-events-none absolute inset-y-0 right-3 flex items-center text-[13px] font-medium text-shell-500">GB</span>
        </div>
        {#if model.validation.vhdSizeError}<p class="text-[12px] leading-5 text-rose-700">{model.validation.vhdSizeError}</p>{/if}
      </div>
      <label class="inline-flex min-h-[36px] items-center gap-2 text-[14px] text-shell-700"><input checked={model.draft.fixedVhd} class="h-4 w-4 accent-[#1d78d7]" disabled={fieldDisabled} onchange={(event) => model.callbacks.setFixedVhd(event.currentTarget.checked)} type="checkbox" /><span>{copy.enableFixedVhd}</span></label>
    </div>
  </div>

  <Button label={model.installSubmitting ? copy.installing : copy.start} icon={Download} iconStrokeWidth={2.1} className="min-h-[38px] w-full text-[14px]" disabled={model.installSubmitDisabled} onclick={() => void model.callbacks.startInstall()} />
</div>
