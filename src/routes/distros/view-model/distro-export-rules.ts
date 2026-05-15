import type { AppCopy } from "$lib/i18n";
import { joinWindowsPath } from "$lib/shared/windows-path";

import type { DistroExportFormatOption, DistroExportMenuView } from "./distro-workspace-types";

const knownExportSuffixPattern = /\.(tar|tar\.gz|tar\.xz|vhd|vhdx)$/i;

export function buildDistroExportFormats(
  copy: AppCopy,
): DistroExportFormatOption[] {
  return [
    {
      label: copy.distros.row.exportMenu.formats.tar,
      suffix: ".tar",
      format: "Tar",
    },
    {
      label: copy.distros.row.exportMenu.formats.tarGz,
      suffix: ".tar.gz",
      format: "TarGz",
    },
    {
      label: copy.distros.row.exportMenu.formats.tarXz,
      suffix: ".tar.xz",
      format: "TarXz",
    },
    {
      label: copy.distros.row.exportMenu.formats.vhd,
      suffix: ".vhdx",
      format: "Vhd",
    },
  ];
}

export function getExportFileNameError(
  value: string,
  errors: DistroExportMenuView["errors"],
): string | null {
  const fileName = value.trim();

  if (fileName.length === 0) {
    return errors.fileNameRequired;
  }

  if (!isValidWindowsFileName(fileName)) {
    return errors.fileNameInvalid;
  }

  if (knownExportSuffixPattern.test(fileName)) {
    return errors.fileNameSuffixNotAllowed;
  }

  return null;
}

export function getExportTargetFile(
  directory: string,
  fileName: string,
  format: DistroExportFormatOption,
): string | null {
  const name = fileName.trim();
  if (name.length === 0) {
    return null;
  }

  return joinWindowsPath(directory, `${name}${format.suffix}`);
}

function isValidWindowsFileName(fileName: string): boolean {
  if (
    fileName === "." ||
    fileName === ".." ||
    /[<>:"/\\|?*\x00-\x1F]/.test(fileName) ||
    /[ .]$/.test(fileName)
  ) {
    return false;
  }

  return !/^(con|prn|aux|nul|com[1-9]|lpt[1-9])(\..*)?$/i.test(fileName);
}
