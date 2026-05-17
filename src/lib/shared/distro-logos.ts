const distroLogoKeysByMatchPriority = [
  "opensuse",
  "ubuntu",
  "suse",
  "kali",
  "debian",
  "alma",
  "arch",
  "fedora",
  "elxr",
  "oracle",
  "docker",
] as const;

export type DistroLogoKey = (typeof distroLogoKeysByMatchPriority)[number];

export const GENERIC_DISTRO_LOGO_SRC = "/distro-logos/generic.ico";

const distroLogoSrcByKey: Partial<Record<DistroLogoKey, string>> = {
  ubuntu: "/distro-logos/ubuntu.ico",
  opensuse: "/distro-logos/opensuse.ico",
  suse: "/distro-logos/suse.ico",
  kali: "/distro-logos/kali.ico",
  debian: "/distro-logos/debian.ico",
  alma: "/distro-logos/alma.ico",
  arch: "/distro-logos/arch.ico",
  fedora: "/distro-logos/fedora.ico",
  elxr: "/distro-logos/elxr.ico",
  oracle: "/distro-logos/oracle.ico",
  docker: "/distro-logos/docker.ico",
};

const aliasDistroKeys = [
  ["suse", "sles"],
] as const satisfies readonly (readonly [DistroLogoKey, string])[];

export function getDistroLogoSrc(
  value: string | null | undefined,
  name?: string,
): string {
  const key = getDistroLogoKey(value, name);

  return key === null
    ? GENERIC_DISTRO_LOGO_SRC
    : (distroLogoSrcByKey[key] ?? GENERIC_DISTRO_LOGO_SRC);
}

export function getDistroLogoKey(
  value: string | null | undefined,
  name?: string,
): DistroLogoKey | null {
  return parseDistroLogoKey(value) ?? parseDistroLogoKey(name);
}

function parseDistroLogoKey(
  value: string | null | undefined,
): DistroLogoKey | null {
  const normalized = value?.trim().toLowerCase();

  if (!normalized) {
    return null;
  }

  const key = distroLogoKeysByMatchPriority.find((distroKey) =>
    normalized.includes(distroKey),
  );

  if (key !== undefined) {
    return key;
  }

  return (
    aliasDistroKeys.find(([, alias]) => normalized.includes(alias))?.[0] ?? null
  );
}
