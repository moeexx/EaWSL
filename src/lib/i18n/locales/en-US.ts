export const enUS = {
  acquire: {
    page: {
      eyebrow: "Acquire",
      title: "Install and import workspace",
      description: "Install online distros or import local WSL images.",
    },
    modes: {
      store: "Store install",
      import: "Import",
    },
    onlineList: {
      title: "Distro list",
      count: (count: number) => `${count} distros`,
      readFailedMessage:
        "The network, wsl.exe, or the online distro list may be unavailable.",
      emptyTitle: "No data",
      emptyMessage:
        "There are no installable online distros. Refresh the list or try again later.",
    },
    sections: {
      installConfig: "Install configuration",
      importConfig: "Import configuration",
    },
    importGuide: {
      title: "Import notes",
      items: [
        {
          title: "Archive files",
          description:
            "Use .tar, .tar.gz, or .tar.xz files exported from WSL. The import directory defaults to the app default location plus the distro name.",
        },
        {
          title: "VHDX files",
          description:
            "The import directory defaults to the source VHDX folder. Keeping that directory registers the original VHDX directly.",
        },
        {
          title: "Other directory",
          description:
            "Choosing a different import directory copies the VHDX there before registration. Existing target directories are rejected.",
        },
      ],
    },
    install: {
      selectedDistro: "Selected distro",
      nameLabel: "Name",
      namePlaceholder: "Distro name after install",
      locationLabel: "Final install directory",
      locationPlaceholder: "Choose the final install directory",
      vhdOptions: "VHD options",
      diskSize: "Disk size",
      enableFixedVhd: "Enable fixed-size VHDX",
      start: "Start install",
      installing: "Installing...",
    },
    importForm: {
      currentMode: "Current mode",
      nameLabel: "Name",
      namePlaceholder: "Distro name after import",
      fileLabel: "Import file",
      filePlaceholder: "Choose .tar / .tar.gz / .tar.xz / .vhdx",
      chooseFile: "Choose file",
      directoryLabel: "Import directory",
      directoryPlaceholder: "Choose the final import directory",
      start: (noun: string) => `Start ${noun}`,
      importing: (noun: string) => `Importing ${noun}...`,
    },
    importNouns: {
      archive: "archive import",
      vhdx: "VHDX import",
      generic: "import",
    },
    dialogs: {
      importFileTitle: "Choose import file",
      importFileFilterName: "WSL import file",
      installLocationTitle: "Choose install location",
      importRootTitle: "Choose import target root",
    },
    refresh: {
      failedTitle: "Refresh failed",
      failedMessage: "Online distro list refresh failed. Please try again later.",
      completedTitle: "Refresh complete",
      successMessage: "Online distro list was updated.",
    },
    toasts: {
      cannotStartInstallTitle: "Cannot start install",
      cannotStartImportTitle: "Cannot import",
      installValidationFallback:
        "Complete distro selection and install parameter validation first.",
      importValidationFallback: "Complete import parameter validation first.",
    },
    validation: {
      installLocationRequired: "Fill the final install directory.",
      installLocationProbeFailed: "Install directory check failed.",
      installLocationExists: "Install directory already exists.",
      installSpaceEmpty: "Fill the final install directory to check disk space.",
      targetDirectoryRequired: "Fill the import directory.",
      targetDirectoryProbeFailed: "Target directory check failed.",
      targetDirectoryExists: "Target directory already exists.",
      importSpaceEmpty:
        "Fill the target location and name to check disk space.",
      nameRequired: "Fill the distro name.",
      nameNoWhitespace: "Distro name cannot contain whitespace.",
      nameProbeFailed: "Distro name check failed. Refresh and try again.",
      nameExists: "Distro name already exists.",
      importFileRequired: "Choose an import file.",
      importFileUnsupported:
        "Choose a `.tar`, `.tar.gz`, `.tar.xz`, or `.vhdx` file.",
      fixedVhdSizeRequired: "Fixed-size VHD requires `--vhd-size`.",
      vhdSizeInteger: "Enter an integer GB value, for example `20`.",
      vhdSizeMinimum: "`--vhd-size` cannot be smaller than 15 GiB.",
      diskSpaceReadFailed: "Failed to read disk free space.",
      diskSpaceInsufficient: (minimum: string) =>
        `Not enough free disk space. The target volume must have more than ${minimum} available.`,
      diskSpaceChecking: "Checking disk free space...",
      diskSpaceWaiting: "Waiting for disk space check.",
      diskSpaceNotEnough: (free: string) =>
        `Available ${free}. The target volume needs more free space.`,
      diskSpaceEnough: (free: string) => `Available ${free}.`,
      importFileProbeFailed: "Import file check failed.",
      importFileMissing: "Import file does not exist.",
      importFileNotFile: "Import path is not a file.",
      installedDistroListReadFailed:
        "Failed to read installed distro list.",
    },
  },
  common: {
    loading: "Loading",
    missing: "Not provided",
    refreshing: "Refreshing...",
    chooseDirectory: "Choose folder",
    refreshList: "Refresh list",
    readFailed: "Read failed",
    export: "Export",
    stop: "Stop",
    stopping: "Stopping...",
    installing: "Installing",
    running: "Running",
    stopped: "Stopped",
    errors: {
      operationFailed: "Operation failed. Please try again.",
    },
    feedback: {
      dismissConfirmDialog: "Close confirmation dialog",
      dismissToast: "Dismiss notification",
    },
  },
  distros: {
    page: {
      eyebrow: "Workspace",
      title: "Core workspace",
      description:
        "This default workspace shows installed distro summaries, state, and core actions.",
    },
    section: {
      title: "Installed distros",
      countLabel: "Distro count",
      loadingTitle: "Reading distro list",
      loadingMessage:
        "Please wait. Overview state stays aligned with the rest of the app.",
      recoveringTitle: "Distro list state is not confirmed",
      emptyTitle: "No installed distros yet",
      emptyMessage:
        "If this is unexpected, confirm with `wsl --list --verbose` in a terminal.",
    },
    notices: {
      listReadFailed: "Failed to read distro list",
      listRecovering: "Distro list state is not confirmed",
    },
    messages: {
      recovering:
        "Distro list state is not confirmed. You can refresh manually later.",
    },
    buttons: {
      shutdownWsl: "Stop WSL",
      forceStopping: "Force stopping...",
    },
    row: {
      collapse: "Collapse",
      defaultBadge: "Default",
      delete: "Delete",
      deleting: "Deleting...",
      deleteVerifying: "Verifying...",
      expand: "Expand",
      collapseExportMenu: "Close export",
      moreInfo: "More information",
      protectedMessage: "Only details are available for this distro.",
      setDefault: "Set as default",
      settingDefault: "Setting...",
      unread: "Not read yet",
      probing: "Probing...",
      detailsTitle: "Distro details",
      unknownState: (state: string) => `Unknown (${state})`,
      defaultUser: {
        label: "Default user",
        value: (uid: number) => `UID ${uid}`,
      },
      details: {
        installLocation: "Install location",
        vhdSize: "VHD size",
      },
      exportMenu: {
        title: "Export menu",
        fileNameLabel: "Export file name",
        fileNamePlaceholder: "File name without suffix",
        formatLabel: "Suffix",
        directoryLabel: "Directory",
        directoryPlaceholder: "Choose export directory",
        directoryDialogTitle: (distro: string) => `Choose ${distro} export directory`,
        submit: "Start export",
        exporting: "Exporting...",
        formats: {
          tar: ".tar",
          tarGz: ".tar.gz",
          tarXz: ".tar.xz",
          vhd: ".vhdx",
        },
        validation: {
          fileNameRequired: "Fill the export file name.",
          fileNameInvalid: "File name contains characters Windows does not allow.",
          fileNameSuffixNotAllowed:
            "Enter the base file name only. The selected suffix is added automatically.",
          directoryRequired: "Fill the export directory.",
          noTauriDirectoryPicker:
            "Tauri runtime was not detected. Export cannot start.",
        },
      },
    },
    overlays: {
      pendingShutdown:
        "The stop command was submitted, but WSL state sync is delayed. Refresh manually later.",
      pendingTerminate: (distro: string) =>
        `${distro} stop was submitted, but state sync is delayed. Refresh manually later.`,
      pendingUnregister: (distro: string) =>
        `${distro} delete was submitted, but state sync is delayed. Refresh manually later.`,
      unknownShutdown:
        "The stop command returned, but not all running distros have been confirmed stopped.",
      unknownTerminate: (distro: string) =>
        `${distro} stop returned, but it has not been confirmed stopped.`,
      unknownUnregister: (distro: string) =>
        `${distro} delete returned, but removal from the list has not been confirmed.`,
    },
    actions: {
      liveReadFailedTitle: "Failed to read live state",
      terminateFailedTitle: "Failed to stop distro",
      terminateSuccessMessage: (distro: string) =>
        `${distro} has been confirmed stopped.`,
      terminatePendingTitle: "Stop sync delayed",
      terminateUnknownTitle: "Stop result is not confirmed",
      terminateUnknownFallback: (distro: string) =>
        `${distro} final state is not confirmed.`,
      shutdownDialogTitle: "Stop all WSL",
      shutdownDialogMessage:
        "Running distros were detected. Choose how to stop them.",
      cancel: "Cancel",
      forceStop: "Force stop",
      forceStopFailedTitle: "Failed to force stop",
      stopAllFailedTitle: "Failed to stop all",
      setDefaultFailedTitle: "Failed to set default",
      unregisterDialogTitle: (distro: string) => `Delete ${distro}`,
      unregisterDialogMessage: (distro: string) =>
        `This will delete ${distro} and cannot be undone. Confirm this is what you want.`,
      unregisterFailedTitle: "Delete failed",
      syncUnknownTitle: "Sync result is not confirmed",
      syncUnknownMessage:
        "The command returned, but the latest list state is not confirmed.",
      missingDistroTitle: "Distro does not exist",
      missingDistroSuccess: (distro: string) =>
        `${distro} no longer exists.`,
      missingDistroFailed: (distro: string) =>
        `${distro} no longer exists, but list sync failed. Refresh manually to confirm.`,
      listStatePendingTitle: "List state is not confirmed",
      distroChangedTitle: "Distro state changed",
      distroChangedFallback: (distro: string) =>
        `${distro} latest state changed. Retry using the current list.`,
      noStopNeededTitle: "No stop needed",
      alreadyStoppedMessage: (distro: string) =>
        `${distro} is already stopped.`,
      alreadyStoppedFailed: (distro: string) =>
        `${distro} stopped state is not confirmed. Refresh manually.`,
      stopSyncFailedTitle: "Stop sync failed",
      alreadyStoppedUnknown: (distro: string) =>
        `${distro} current stopped state is not confirmed. Retry using the latest list.`,
      noRunningMessage: "There are no running distros.",
      noRunningFailed:
        "Whether any distros are still running is not confirmed. Refresh manually.",
      noRunningUnknown: "The list changed. Re-check using the current running state.",
      noDefaultNeededTitle: "No default change needed",
      alreadyDefaultMessage: (distro: string) =>
        `${distro} is already the default distro.`,
      alreadyDefaultFailed: (distro: string) =>
        `${distro} current default state is not confirmed. Refresh manually.`,
      setDefaultPendingTitle: "Default sync delayed",
      setDefaultUnknownTitle: "Default result is not confirmed",
      setDefaultChangedFallback: (distro: string) =>
        `${distro} default state changed. Retry using the current list.`,
    },
    actionSupport: {
      stopSyncFailedTitle: "Stop sync failed",
      stopSyncFailedMessage:
        "The stop command ran, but list sync failed. Refresh manually to confirm state.",
      actionSyncFailedTitle: "Action sync failed",
      actionSyncFailedMessage:
        "The command returned, but list sync failed. Refresh manually to confirm state.",
      forceStopSyncFailedTitle: "Force stop sync failed",
      shutdownSyncFailedMessage:
        "The stop command returned, but list sync failed. Refresh manually to confirm state.",
      forceStoppedTitle: "Force stopped",
      stoppedAllTitle: "Stopped all",
      forceStoppedMessage:
        "Force stop ran and no running distros remain.",
      stoppedAllMessage:
        "Stop all WSL ran and no running distros remain.",
      forceStopPendingTitle: "Force stop sync delayed",
      stopPendingTitle: "Stop sync delayed",
      forceStopUnknownTitle: "Force stop result is not confirmed",
      stopUnknownTitle: "Stop result is not confirmed",
      stopUnknownFallback:
        "The stop command returned, but not all distros have been confirmed stopped.",
      unregisterSyncFailedTitle: "Delete sync failed",
      unregisterSyncFailedMessage:
        "The delete command returned, but list sync failed. Refresh manually to confirm state.",
      unregisterSuccessTitle: "Deleted",
      unregisterSuccessMessage: (distro: string) =>
        `${distro} was removed from the list.`,
      unregisterPendingTitle: "Delete sync delayed",
      unregisterUnknownTitle: "Cannot confirm delete result",
      unregisterUnknownFallback:
        "The delete command was submitted, but the final state is not confirmed.",
      setDefaultSyncFailedTitle: "Default sync failed",
      setDefaultSyncFailedMessage:
        "The command returned, but list sync failed. Refresh manually to confirm state.",
      setDefaultSuccessTitle: "Default set",
      setDefaultSuccessMessage: (distro: string) =>
        `${distro} has been confirmed as the default distro.`,
      setDefaultUnknownTitle: "Default result is not confirmed",
      setDefaultUnknownFallback: (distro: string) =>
        `${distro} default command returned, but it has not been confirmed as the default distro.`,
    },
  },
  longTasks: {
    operations: {
      install: "Install",
      importArchive: "Archive import",
      importVhd: "VHDX import",
      export: "Export",
    },
    phases: {
      waiting: "Waiting",
      Copying: "Copying",
      Downloading: "Downloading",
      Installing: "Installing",
      Importing: "Importing",
      Exporting: "Exporting",
    },
    progress: {
      completed: "Task completed",
      failed: "Task failed",
      interrupted: "Task interrupted",
    },
    errors: {
      interrupted:
        "The app was reloaded or closed before this task finished. The original WSL command cannot be resumed.",
    },
    status: {
      completed: "Completed",
      failed: "Failed",
      running: "Running",
    },
    tray: {
      title: "Long task details",
      totalTasks: "tasks",
      currentTask: "Current task",
      recentTask: "Recent task",
      failedAt: "Failed at: ",
      noTasks: "No long tasks",
      startedAt: "Started: ",
      endedAt: "Ended: ",
      expandDetails: "Expand details",
      collapseDetails: "Collapse details",
      total: (total: number, label: string) => `${total} ${label}`,
      active: (active: number) => `Running ${active}`,
      completed: (completed: number) => `Completed ${completed}`,
      failed: (failed: number) => `Failed ${failed}`,
    },
    card: {
      distro: "Distro",
      location: "Location",
      startedAt: "Start time",
      endedAt: "End time",
      failedAt: "Failure time",
      logoAlt: (distro: string) => `${distro} logo`,
    },
    collapsed: {
      failed: (operation: string) => `${operation} failed`,
      completed: (operation: string) => `${operation} completed`,
    },
    acquireTasks: {
      activeTitle: "A long task is already running",
      activeMessage: "A long task is already running. Wait for it to finish.",
      failedTitle: (noun: string) => `${noun} failed`,
      nameCheckFailedTitle: "Name check failed",
      nameExistsTitle: "Name already exists",
      nameExistsMessage: (name: string) =>
        `${name} already exists. Choose another distro name.`,
      targetExistsMessage: (location: string) =>
        `${location} already exists. Choose another target directory.`,
      directoryCheckFailedTitle: "Directory check failed",
      syncFailedTitle: (noun: string) => `${noun} sync failed`,
      syncFailedMessage: (noun: string) =>
        `${noun} command completed, but the installed list refresh failed. Refresh manually to confirm the result.`,
      syncDelayedTitle: (noun: string) => `${noun} sync delayed`,
      syncDelayedMessage: (displayName: string, noun: string) =>
        `${displayName} ${noun} completed, but installed list sync is delayed. Refresh manually later.`,
      resultUnknownTitle: (noun: string) =>
        `${noun} result is not confirmed`,
      resultUnknownMessage: (displayName: string, noun: string) =>
        `${displayName} ${noun} completed, but it has not been confirmed in the installed list.`,
      completedTitle: (noun: string) => `${noun} completed`,
      completedMessage: (displayName: string, noun: string) =>
        `${displayName} completed ${noun} and is now in the installed list.`,
      installDirectoryExistsTitle: "Install directory already exists",
      targetDirectoryExistsTitle: "Target directory already exists",
      installedDistroListReadFailed:
        "Failed to read installed distro list.",
    },
    exportTasks: {
      activeTitle: "A long task is already running",
      activeMessage: "A long task is already running. Wait for it to finish.",
      failedTitle: (noun: string) => `${noun} failed`,
      completedTitle: "Export completed",
      completedMessage: (distro: string) => `${distro} export completed.`,
      targetFileExistsTitle: "Export file already exists",
      targetFileExistsMessage: (file: string) =>
        `${file} already exists. Choose another file name or directory.`,
      targetFileCheckFailedTitle: "Export file check failed",
      targetDirectoryCheckFailedTitle: "Export directory check failed",
      targetDirectoryInvalidTitle: "Export directory is unavailable",
      targetDirectoryInvalidMessage: (directory: string) =>
        `${directory} does not exist or is not a folder.`,
      vhdxSuffixRequiredMessage:
        "VHD export must use a .vhdx target file.",
    },
  },
  overview: {
    page: {
      eyebrow: "Overview",
      title: "Host overview",
      description:
        "System information and WSL overview refresh sequentially when you enter the page. Manual refresh waits for the minimum duration and finishes foreground requests before advancing.",
      refreshLabel: "Refresh overview",
      logSubject: "Host overview",
    },
    refresh: {
      failedTitle: "Refresh failed",
      failedMessage: "Host overview refresh failed. Please try again later.",
      completedTitle: "Refresh complete",
      recoveringMessage:
        "Host overview was updated, but some results are not confirmed. Last successful data is kept.",
      successMessage: "Host overview data was updated.",
    },
    notices: {
      systemError: "Failed to read system information",
      systemRecovering: "System information state is not confirmed",
      wslError: "Failed to read WSL information",
      wslRecovering: "WSL information state is not confirmed",
      distrosRecovering: "Distro summary state is not confirmed",
    },
    system: {
      title: "System information",
      description:
        "Shows the host snapshot cached after app startup. It does not refresh automatically.",
      labels: {
        windows: "Windows information",
        displayVersion: "Version",
        buildNumber: "Build",
        cpu: "Processor",
        maxClock: "Base clock",
        cores: "Cores",
        memory: "Memory",
        speed: "Speed",
        slots: "Slots",
        gpuMemory: "GPU memory",
        driverVersion: "Driver version",
        storage: "Disk",
        used: "Used",
        free: "Free",
        volumeCount: "Volumes",
      },
      coreCount: (cores: number) => `${cores} cores`,
      coreThreadCount: (cores: number, threads: number) =>
        `${cores} cores / ${threads} threads`,
      usedSlots: (used: number) => `${used} used`,
      slotUsage: (used: number, total: number) => `${used} / ${total} used`,
    },
    wsl: {
      title: "WSL information",
      description:
        "Default distro and distro count come from the installed distro cache. Refreshing updates them with WSL version information.",
      copied: "Copied",
      copy: "Copy",
      status: {
        failed: "Read failed",
        notSet: "Not set",
        recovering: "State not confirmed",
      },
      fields: {
        defaultDistro: "Default distro",
        direct3d: "Direct3D version",
        distroCount: "Distro count",
        dxcore: "DXCore version",
        kernel: "Kernel version",
        msrdc: "MSRDC version",
        wsl: "WSL version",
        wslg: "WSLg version",
      },
    },
  },
  queryCache: {
    errors: {
      distros: "Failed to read distro list.",
      wslVersion: "Failed to read WSL version.",
      systemOverview: "Failed to read host information.",
      onlineDistros: "Failed to read online distro list.",
    },
    recovering: {
      subjects: {
        distros: "Distro list read timed out",
        wslVersion: "WSL version read timed out",
        systemOverview: "Host information probe timed out",
        onlineDistros: "Online distro list read timed out",
        hostCommandTimedOut: "Host information probe timed out",
      },
      withFallbackData: (subject: string) =>
        `${subject}. Showing the last successful result; you can refresh manually later.`,
      withoutFallbackData: (subject: string) =>
        `${subject}. Current state is not confirmed; you can refresh manually later.`,
    },
  },
  settings: {
    page: {
      eyebrow: "Settings",
      title: "Settings",
    },
    language: {
      label: "Language",
      options: {
        "en-US": "English",
        "zh-CN": "Simplified Chinese",
      },
    },
    defaultInstallLocation: {
      label: "Default install location",
      dialogTitle: "Choose default install location",
    },
    backgroundRefresh: {
      intervalLabel: "Background refresh interval (minutes)",
      targetsLabel: "Background refresh targets",
      targets: {
        distros: "Distro list",
        systemOverviewStorage: "System storage information",
        wslVersion: "WSL version",
        onlineDistros: "Online distro list",
      },
    },
    validation: {
      defaultInstallLocationRequired:
        "Default install location cannot be empty.",
      backgroundTargetsRequired: "Background refresh targets cannot be empty.",
      backgroundIntervalInteger:
        "Background refresh interval must be an integer number of minutes.",
      backgroundIntervalRange: (min: number, max: number) =>
        `Background refresh interval must be between ${min} and ${max} minutes.`,
    },
    errors: {
      noTauriDirectoryPicker:
        "Tauri runtime was not detected. The folder picker cannot be opened.",
      noTauriSave: "Tauri runtime was not detected. Settings cannot be saved.",
      saveFailed: "Save failed",
    },
    actions: {
      saving: "Saving",
      saveSettings: "Save settings",
    },
  },
  shell: {
    navigation: {
      label: "Navigation",
      primaryAriaLabel: "Primary navigation",
      items: {
        overview: "Overview",
        distros: "Distros",
        acquire: "Install",
        settings: "Settings",
      },
    },
    sidebar: {
      collapse: "Collapse sidebar",
      expand: "Expand sidebar",
    },
    titlebar: {
      close: "Close",
      maximize: "Maximize",
      minimize: "Minimize",
      resize: (direction: string) => `Resize ${direction}`,
      resizeDirections: {
        North: "north",
        South: "south",
        East: "east",
        West: "west",
        NorthEast: "north-east",
        NorthWest: "north-west",
        SouthEast: "south-east",
        SouthWest: "south-west",
      },
      restore: "Restore",
    },
  },
} as const;

export type EnUsCopy = typeof enUS;

export default enUS;
