export function joinWindowsPath(
  base: string | null | undefined,
  child: string | null | undefined,
): string | null {
  const basePart = base?.trim().replace(/[\\/]+$/, "") ?? "";
  const childPart =
    child
      ?.trim()
      .replace(/^[\\/]+/, "")
      .replace(/[\\/]+$/, "") ?? "";

  if (basePart.length === 0 || childPart.length === 0) {
    return null;
  }

  return `${basePart}\\${childPart}`;
}

export function getWindowsParentPath(
  path: string | null | undefined,
): string | null {
  const trimmed = path?.trim().replace(/[\\/]+$/, "") ?? "";
  if (trimmed.length === 0) return null;

  const index = Math.max(trimmed.lastIndexOf("\\"), trimmed.lastIndexOf("/"));
  if (index < 0) return null;

  return index === 2 && /^[A-Za-z]:[\\/]/.test(trimmed)
    ? trimmed.slice(0, 3)
    : index <= 0
      ? null
      : trimmed.slice(0, index);
}
