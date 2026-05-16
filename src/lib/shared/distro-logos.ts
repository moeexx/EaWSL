export type DistroLogoKey =
  | "ubuntu"
  | "opensuse"
  | "suse"
  | "kali"
  | "debian"
  | "alma"
  | "arch"
  | "fedora"
  | "elxr"
  | "oracle";

export const GENERIC_DISTRO_LOGO_SRC = "/distro-logos/generic.ico";

const distroLogoSrcByKey: Partial<Record<DistroLogoKey, string>> = {
  ubuntu: "/distro-logos/ubuntu.ico",
  debian: "/distro-logos/debian.ico",
  arch: "/distro-logos/arch.ico",
};

export function getDistroLogoSrc(value: string | null | undefined): string {
  const key = getDistroLogoKey(value);
  return key === null
    ? GENERIC_DISTRO_LOGO_SRC
    : distroLogoSrcByKey[key] ?? GENERIC_DISTRO_LOGO_SRC;
}

export function getDistroLogoKey(
  value: string | null | undefined,
): DistroLogoKey | null {
  const normalized = value?.trim().toLowerCase() ?? "";

  if (normalized.length === 0) {
    return null;
  }

  const tokens = normalized.split(/[^a-z0-9]+/).filter(Boolean);
  const compact = tokens.join("");

  if (
    tokens.includes("opensuse") ||
    compact.startsWith("opensuse") ||
    hasAdjacentTokens(tokens, "open", "suse")
  ) {
    return "opensuse";
  }

  if (tokens.includes("suse")) return "suse";
  if (tokens.includes("ubuntu")) return "ubuntu";
  if (tokens.includes("kali")) return "kali";
  if (tokens.includes("debian")) return "debian";
  if (tokens.includes("alma") || compact.startsWith("almalinux")) return "alma";
  if (tokens.includes("arch") || compact.startsWith("archlinux")) return "arch";
  if (tokens.includes("fedora")) return "fedora";
  if (tokens.includes("elxr")) return "elxr";
  if (tokens.includes("oracle") || compact.startsWith("oraclelinux")) {
    return "oracle";
  }

  return null;
}

function hasAdjacentTokens(tokens: string[], first: string, second: string): boolean {
  return tokens.some(
    (token, index) => token === first && tokens[index + 1] === second,
  );
}
