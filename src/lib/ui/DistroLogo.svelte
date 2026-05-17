<script lang="ts">
  import { GENERIC_DISTRO_LOGO_SRC } from "$lib/shared/distro-logos";

  const sizeClassBySize = {
    sm: "h-9 w-9 rounded-[9px]",
    md: "h-11 w-11 rounded-[11px]",
    lg: "h-13 w-13 rounded-[13px]",
  } as const;

  type DistroLogoSize = keyof typeof sizeClassBySize;

  type Props = {
    src: string;
    alt?: string;
    size?: DistroLogoSize;
  };

  let { src, alt = "", size = "md" }: Props = $props();

  function handleError(event: Event): void {
    const image = event.currentTarget as HTMLImageElement | null;

    if (image?.getAttribute("src") !== GENERIC_DISTRO_LOGO_SRC) {
      image?.setAttribute("src", GENERIC_DISTRO_LOGO_SRC);
    }
  }
</script>

<img
  {alt}
  class={`shrink-0 border-[0.5px] border-shell-200 bg-shell-50 object-contain p-[3px] ${sizeClassBySize[size]}`}
  src={src || GENERIC_DISTRO_LOGO_SRC}
  onerror={handleError}
/>
