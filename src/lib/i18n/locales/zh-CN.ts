import type { DeepPartial, WidenCopy } from "../types";
import type { EnUsCopy } from "./en-US";

export const zhCN = {
  acquire: {
    page: {
      eyebrow: "Acquire",
      title: "安装和导入工作台",
      description: "安装在线发行版，或导入本地 WSL 镜像。",
    },
    modes: {
      store: "商店安装",
      import: "导入",
    },
    onlineList: {
      title: "发行版列表",
      count: (count: number) => `共 ${count} 个发行版`,
      readFailedMessage:
        "可能是网络异常、`wsl.exe` 返回错误或在线发行版列表暂时不可用。",
      emptyTitle: "没有数据",
      emptyMessage: "当前没有可安装的在线发行版，请刷新列表或稍后重试。",
    },
    sections: {
      installConfig: "安装配置",
      importConfig: "导入配置",
    },
    importGuide: {
      title: "导入说明",
      items: [
        {
          title: "归档文件",
          description:
            "支持从 WSL 导出的 .tar、.tar.gz、.tar.xz。导入目录默认使用应用默认位置和发行版名称生成。",
        },
        {
          title: "VHDX 文件",
          description:
            "导入目录默认使用源 VHDX 所在目录。保持该目录时直接注册原 VHDX。",
        },
        {
          title: "其他目录",
          description:
            "选择其他导入目录时会先复制 VHDX，再执行注册；目标目录已存在时会被拒绝。",
        },
      ],
    },
    install: {
      selectedDistro: "已选发行版",
      nameLabel: "名称",
      namePlaceholder: "安装后的发行版名称",
      locationLabel: "最终安装目录",
      locationPlaceholder: "选择最终安装目录",
      vhdOptions: "VHD 选项",
      diskSize: "磁盘空间大小",
      enableFixedVhd: "启用固定大小 VHDX",
      start: "开始安装",
      installing: "安装中...",
    },
    importForm: {
      currentMode: "当前模式",
      nameLabel: "名称",
      namePlaceholder: "导入后的发行版名称",
      fileLabel: "导入文件",
      filePlaceholder: "选择 .tar / .tar.gz / .tar.xz / .vhdx",
      chooseFile: "选择文件",
      directoryLabel: "导入目录",
      directoryPlaceholder: "选择最终导入目录",
      start: (noun: string) => `开始${noun}`,
      importing: (noun: string) => `${noun}中...`,
    },
    importNouns: {
      archive: "导入归档",
      vhdx: "导入 VHDX",
      generic: "导入",
    },
    dialogs: {
      importFileTitle: "选择导入文件",
      importFileFilterName: "WSL 导入文件",
      installLocationTitle: "选择安装位置",
      importRootTitle: "选择导入目标根目录",
    },
    refresh: {
      failedTitle: "刷新失败",
      failedMessage: "在线发行版列表刷新失败，请稍后重试。",
      completedTitle: "刷新完成",
      successMessage: "在线发行版列表已更新。",
    },
    toasts: {
      cannotStartInstallTitle: "无法开始安装",
      cannotStartImportTitle: "无法导入",
      installValidationFallback: "请先完成发行版选择和安装参数校验。",
      importValidationFallback: "请先完成导入参数校验。",
    },
    validation: {
      installLocationRequired: "请填写最终安装目录。",
      installLocationProbeFailed: "安装目录检查失败。",
      installLocationExists: "安装目录已存在。",
      installSpaceEmpty: "填写最终安装目录后检查磁盘空间。",
      targetDirectoryRequired: "请填写导入目录。",
      targetDirectoryProbeFailed: "目标目录检查失败。",
      targetDirectoryExists: "目标目录已存在。",
      importSpaceEmpty: "填写目标位置和名称后检查磁盘空间。",
      nameRequired: "请填写发行版名称。",
      nameNoWhitespace: "发行版名称不能包含空白字符。",
      nameProbeFailed: "发行版名称检查失败，请刷新后重试。",
      nameExists: "发行版名称已存在。",
      importFileRequired: "请选择导入文件。",
      importFileUnsupported:
        "请选择 `.tar`、`.tar.gz`、`.tar.xz` 或 `.vhdx` 文件。",
      fixedVhdSizeRequired: "固定大小 VHD 需要填写 `--vhd-size`。",
      vhdSizeInteger: "请输入整数 GB，例如 `20`。",
      vhdSizeMinimum: "`--vhd-size` 不能小于 15 GiB。",
      diskSpaceReadFailed: "磁盘可用空间读取失败。",
      diskSpaceInsufficient: (minimum: string) =>
        `当前磁盘可用空间不足，目标卷可用空间必须大于 ${minimum}。`,
      diskSpaceChecking: "正在检查磁盘可用空间...",
      diskSpaceWaiting: "等待磁盘空间检查。",
      diskSpaceNotEnough: (free: string) =>
        `当前可用 ${free}，目标卷需要更多可用空间。`,
      diskSpaceEnough: (free: string) => `当前可用 ${free}。`,
      importFileProbeFailed: "导入文件检查失败。",
      importFileMissing: "导入文件不存在。",
      importFileNotFile: "导入路径不是文件。",
      installedDistroListReadFailed: "无法读取已安装发行版列表。",
    },
  },
  common: {
    loading: "读取中",
    missing: "未提供",
    refreshing: "刷新中...",
    chooseDirectory: "选择目录",
    refreshList: "刷新列表",
    readFailed: "读取失败",
    export: "导出",
    stop: "停止",
    stopping: "停止中...",
    installing: "安装中",
    running: "运行中",
    stopped: "已停止",
    errors: {
      operationFailed: "操作失败，请重试。",
      wslCommandFailed: (code: string, details?: string) =>
        details
          ? `WSL 命令失败：${code}。详情：${details}`
          : `WSL 命令失败：${code}。`,
      wslCommandTimedOut: "WSL 命令超时，未能获得稳定结果。",
      invalidWslArguments: "WSL 命令参数无效。",
      fileNotFound: "指定文件不存在。",
      distroNotFound: "指定发行版不存在。",
      diskResizeFailed: "发行版磁盘扩容失败。",
      wslOperationNotPermitted: (distro: string) =>
        `发行版 \`${distro}\` 不允许执行该操作。`,
      registryReadFailed: "WSL 注册表信息读取失败。",
      outputParseFailed: "WSL 命令输出解析失败。",
      processFailed: "WSL 命令启动失败。",
      processKilled: "WSL 命令退出时没有状态码。",
      cancelled: "操作已取消。",
    },
    feedback: {
      dismissConfirmDialog: "关闭确认弹窗",
      dismissToast: "关闭提示",
    },
  },
  distros: {
    page: {
      eyebrow: "Workspace",
      title: "核心功能工作台",
      description:
        "这里作为默认首屏，集中展示已安装发行版的摘要信息、状态概览和核心操作布局。",
    },
    section: {
      title: "已安装发行版",
      countLabel: "发行版数量",
      loadingTitle: "正在读取发行版列表",
      loadingMessage: "请稍候，概览状态会在这里与其他页面保持一致。",
      recoveringTitle: "发行版列表状态暂未确认",
      emptyTitle: "还没有已安装发行版",
      emptyMessage:
        "如果这不符合预期，可以先在命令行确认 `wsl --list --verbose`。",
    },
    notices: {
      listReadFailed: "发行版列表读取失败",
      listRecovering: "发行版列表状态暂未确认",
    },
    messages: {
      recovering: "发行版列表状态暂未确认，可稍后手动刷新。",
    },
    buttons: {
      shutdownWsl: "停止WSL",
      forceStopping: "强停中...",
    },
    row: {
      collapse: "收起",
      defaultBadge: "默认",
      delete: "删除",
      deleting: "删除中...",
      deleteVerifying: "校验中...",
      expand: "展开",
      collapseExportMenu: "关闭导出",
      moreInfo: "更多信息",
      protectedMessage: "该发行版仅保留详情查看。",
      setDefault: "设置为默认",
      settingDefault: "设置中...",
      unread: "暂未读取",
      probing: "正在探测中...",
      detailsTitle: "发行版详情",
      unknownState: (state: string) => `未知 (${state})`,
      defaultUser: {
        label: "默认用户",
        value: (uid: number) => `UID ${uid}`,
      },
      details: {
        installLocation: "安装位置",
        vhdSize: "VHD 大小",
      },
      exportMenu: {
        title: "导出菜单",
        fileNameLabel: "导出文件名",
        fileNamePlaceholder: "不含后缀的文件名",
        formatLabel: "后缀",
        directoryLabel: "目录",
        directoryPlaceholder: "选择导出目录",
        directoryDialogTitle: (distro: string) => `选择 ${distro} 导出目录`,
        submit: "开始导出",
        exporting: "导出中...",
        formats: {
          tar: ".tar",
          tarGz: ".tar.gz",
          tarXz: ".tar.xz",
          vhd: ".vhdx",
        },
        validation: {
          fileNameRequired: "请填写导出文件名。",
          fileNameInvalid: "文件名包含 Windows 不允许的字符。",
          fileNameSuffixNotAllowed:
            "只填写基础文件名，后缀会按所选格式自动添加。",
          directoryRequired: "请填写导出目录。",
          noTauriDirectoryPicker: "未检测到 Tauri 运行时，无法开始导出。",
        },
      },
    },
    overlays: {
      pendingShutdown: "停止命令已提交，WSL 状态同步延后，请稍后手动刷新。",
      pendingTerminate: (distro: string) =>
        `${distro} 的停止命令已提交，状态同步延后，请稍后手动刷新。`,
      pendingUnregister: (distro: string) =>
        `${distro} 的删除命令已提交，状态同步延后，请稍后手动刷新。`,
      unknownShutdown: "停止命令已返回，但暂未确认所有运行中的发行版都已停止。",
      unknownTerminate: (distro: string) =>
        `${distro} 的停止命令已返回，但暂未确认其已停止。`,
      unknownUnregister: (distro: string) =>
        `${distro} 的删除命令已返回，但暂未确认其已从列表移除。`,
    },
    actions: {
      liveReadFailedTitle: "实时状态读取失败",
      terminateFailedTitle: "停止发行版失败",
      terminateSuccessMessage: (distro: string) => `${distro} 已确认停止。`,
      terminatePendingTitle: "停止后同步延后",
      terminateUnknownTitle: "停止结果暂未确认",
      terminateUnknownFallback: (distro: string) =>
        `${distro} 的最终状态暂未确认。`,
      shutdownDialogTitle: "停止全部 WSL",
      shutdownDialogMessage: "检测到运行中的发行版。请选择停止方式。",
      cancel: "取消",
      forceStop: "强制停止",
      forceStopFailedTitle: "强制停止失败",
      stopAllFailedTitle: "停止全部失败",
      setDefaultFailedTitle: "设置默认失败",
      unregisterDialogTitle: (distro: string) => `删除 ${distro}`,
      unregisterDialogMessage: (distro: string) =>
        `这会删除 ${distro}，且无法恢复。请确认这是你要执行的操作。`,
      unregisterFailedTitle: "删除失败",
      syncUnknownTitle: "同步结果暂未确认",
      syncUnknownMessage: "命令已返回，但暂未确认最新列表状态。",
      missingDistroTitle: "发行版不存在",
      missingDistroSuccess: (distro: string) => `${distro} 已不存在。`,
      missingDistroFailed: (distro: string) =>
        `${distro} 已不存在，但列表同步失败，请手动刷新确认。`,
      listStatePendingTitle: "列表状态暂未确认",
      distroChangedTitle: "发行版状态已变化",
      distroChangedFallback: (distro: string) =>
        `${distro} 的最新状态已变化，请按当前列表重试。`,
      noStopNeededTitle: "无需停止",
      alreadyStoppedMessage: (distro: string) => `${distro} 当前已停止。`,
      alreadyStoppedFailed: (distro: string) =>
        `${distro} 当前是否已停止暂未确认，请手动刷新。`,
      stopSyncFailedTitle: "停止后同步失败",
      alreadyStoppedUnknown: (distro: string) =>
        `${distro} 的当前停止状态暂未确认，请按最新列表重试。`,
      noRunningMessage: "当前没有运行中的发行版。",
      noRunningFailed: "当前是否仍有运行中的发行版暂未确认，请手动刷新。",
      noRunningUnknown: "列表已变化，请按当前运行状态重新判断。",
      noDefaultNeededTitle: "无需设置默认",
      alreadyDefaultMessage: (distro: string) =>
        `${distro} 当前已经是默认发行版。`,
      alreadyDefaultFailed: (distro: string) =>
        `${distro} 当前是否仍为默认发行版暂未确认，请手动刷新。`,
      setDefaultPendingTitle: "默认切换同步延后",
      setDefaultUnknownTitle: "默认结果暂未确认",
      setDefaultChangedFallback: (distro: string) =>
        `${distro} 的默认状态已变化，请按当前列表重试。`,
    },
    actionSupport: {
      stopSyncFailedTitle: "停止后同步失败",
      stopSyncFailedMessage:
        "已执行停止命令，但列表同步失败，请手动刷新确认状态。",
      actionSyncFailedTitle: "操作后同步失败",
      actionSyncFailedMessage:
        "命令已返回，但列表同步失败，请手动刷新确认状态。",
      forceStopSyncFailedTitle: "强停后同步失败",
      shutdownSyncFailedMessage:
        "停止命令已返回，但列表同步失败，请手动刷新确认状态。",
      forceStoppedTitle: "已强制停止",
      stoppedAllTitle: "已停止全部",
      forceStoppedMessage: "已执行强制停止，且已确认没有运行中的发行版。",
      stoppedAllMessage: "已执行停止全部 WSL，且已确认没有运行中的发行版。",
      forceStopPendingTitle: "强停后同步延后",
      stopPendingTitle: "停止后同步延后",
      forceStopUnknownTitle: "强停结果暂未确认",
      stopUnknownTitle: "停止结果暂未确认",
      stopUnknownFallback: "停止命令已返回，但暂未确认所有发行版都已停止。",
      unregisterSyncFailedTitle: "删除后同步失败",
      unregisterSyncFailedMessage:
        "删除命令已返回，但列表同步失败，请手动刷新确认状态。",
      unregisterSuccessTitle: "删除成功",
      unregisterSuccessMessage: (distro: string) =>
        `${distro} 已从列表中移除。`,
      unregisterPendingTitle: "删除后同步延后",
      unregisterUnknownTitle: "无法确认删除结果",
      unregisterUnknownFallback: "删除命令已提交，但暂时无法确认最终状态。",
      setDefaultSyncFailedTitle: "默认切换同步失败",
      setDefaultSyncFailedMessage:
        "命令已返回，但列表同步失败，请手动刷新确认状态。",
      setDefaultSuccessTitle: "设置默认成功",
      setDefaultSuccessMessage: (distro: string) =>
        `${distro} 已确认设为默认发行版。`,
      setDefaultUnknownTitle: "默认切换结果暂未确认",
      setDefaultUnknownFallback: (distro: string) =>
        `${distro} 的默认设置命令已返回，但暂未确认其已成为默认发行版。`,
    },
  },
  longTasks: {
    operations: {
      install: "安装",
      importArchive: "导入归档",
      importVhd: "导入 VHDX",
      export: "导出",
    },
    phases: {
      waiting: "等待中",
      Copying: "复制中",
      Downloading: "下载中",
      Installing: "安装中",
      Importing: "导入中",
      Exporting: "导出中",
    },
    progress: {
      completed: "任务已完成",
      failed: "任务失败",
      interrupted: "任务已中断",
    },
    errors: {
      interrupted: "应用在该任务完成前重载或关闭，原 WSL 命令无法恢复。",
    },
    status: {
      completed: "已完成",
      failed: "失败",
      running: "进行中",
    },
    tray: {
      title: "长任务详情",
      totalTasks: "任务",
      currentTask: "当前任务",
      recentTask: "最近任务",
      failedAt: "失败时间：",
      noTasks: "当前没有长任务",
      startedAt: "开始：",
      endedAt: "结束：",
      expandDetails: "展开详情",
      collapseDetails: "折叠详情",
      total: (total: number, label: string) => `${total} 个${label}`,
      active: (active: number) => `进行中 ${active}`,
      completed: (completed: number) => `已完成 ${completed}`,
      failed: (failed: number) => `失败 ${failed}`,
    },
    card: {
      distro: "发行版",
      location: "位置",
      startedAt: "开始时间",
      endedAt: "结束时间",
      failedAt: "失败时间",
      logoAlt: (distro: string) => `${distro} logo`,
    },
    collapsed: {
      failed: (operation: string) => `${operation}失败`,
      completed: (operation: string) => `${operation}完成`,
    },
    acquireTasks: {
      activeTitle: "已有长任务正在进行",
      activeMessage: "当前已有长任务正在进行，请等待其完成。",
      failedTitle: (noun: string) => `${noun}失败`,
      nameCheckFailedTitle: "名称检查失败",
      nameExistsTitle: "名称已存在",
      nameExistsMessage: (name: string) => `${name} 已存在，请更换发行版名称。`,
      targetExistsMessage: (location: string) =>
        `${location} 已存在，请更换目标目录。`,
      directoryCheckFailedTitle: "目录检查失败",
      syncFailedTitle: (noun: string) => `${noun}后同步失败`,
      syncFailedMessage: (noun: string) =>
        `${noun}命令已完成，但已安装列表刷新失败，请手动刷新确认结果。`,
      syncDelayedTitle: (noun: string) => `${noun}后同步延后`,
      syncDelayedMessage: (displayName: string, noun: string) =>
        `${displayName} ${noun}已完成，但已安装列表同步延后，请稍后手动刷新。`,
      resultUnknownTitle: (noun: string) => `${noun}结果暂未确认`,
      resultUnknownMessage: (displayName: string, noun: string) =>
        `${displayName} ${noun}已完成，但暂未确认其已出现在已安装列表中。`,
      completedTitle: (noun: string) => `${noun}完成`,
      completedMessage: (displayName: string, noun: string) =>
        `${displayName} 已完成${noun}，并已出现在已安装列表中。`,
      installDirectoryExistsTitle: "安装目录已存在",
      targetDirectoryExistsTitle: "目标目录已存在",
      installedDistroListReadFailed: "无法读取已安装发行版列表。",
    },
    exportTasks: {
      activeTitle: "已有长任务正在进行",
      activeMessage: "当前已有长任务正在进行，请等待其完成。",
      failedTitle: (noun: string) => `${noun}失败`,
      completedTitle: "导出完成",
      completedMessage: (distro: string) => `${distro} 导出完成。`,
      targetFileExistsTitle: "导出文件已存在",
      targetFileExistsMessage: (file: string) =>
        `${file} 已存在，请更换文件名或目录。`,
      targetFileCheckFailedTitle: "导出文件检查失败",
      targetDirectoryCheckFailedTitle: "导出目录检查失败",
      targetDirectoryInvalidTitle: "导出目录不可用",
      targetDirectoryInvalidMessage: (directory: string) =>
        `${directory} 不存在或不是文件夹。`,
      vhdxSuffixRequiredMessage: "VHD 导出必须使用 .vhdx 目标文件。",
    },
  },
  overview: {
    page: {
      eyebrow: "Overview",
      title: "主机概览",
      description:
        "系统信息与 WSL 概览会在进入页面时按顺序刷新；手动刷新会补足最小等待时间，并在全部前台请求完成后才进入下一阶段。",
      refreshLabel: "刷新概览",
      logSubject: "主机概览",
    },
    refresh: {
      failedTitle: "刷新失败",
      failedMessage: "主机概览刷新失败，请稍后重试。",
      completedTitle: "刷新完成",
      recoveringMessage:
        "主机概览已更新，但部分结果暂未确认，已保留上次成功数据。",
      successMessage: "主机概览数据已更新。",
    },
    notices: {
      systemError: "系统信息读取失败",
      systemRecovering: "系统信息状态暂未确认",
      wslError: "WSL 信息读取失败",
      wslRecovering: "WSL 信息状态暂未确认",
      distrosRecovering: "发行版摘要状态暂未确认",
    },
    system: {
      title: "系统信息",
      description: "这里展示应用启动后缓存的宿主机快照；不会自动轮询刷新。",
      labels: {
        windows: "Windows 信息",
        displayVersion: "版本",
        buildNumber: "系统版本",
        cpu: "处理器",
        maxClock: "基准频率",
        cores: "核心数",
        memory: "内存",
        speed: "速度",
        slots: "插槽",
        gpuMemory: "显存",
        driverVersion: "驱动版本",
        storage: "磁盘",
        used: "已用",
        free: "可用",
        volumeCount: "卷数量",
      },
      coreCount: (cores: number) => `${cores} 核`,
      coreThreadCount: (cores: number, threads: number) =>
        `${cores} 核 / ${threads} 线程`,
      usedSlots: (used: number) => `${used} 个已占用`,
      slotUsage: (used: number, total: number) => `${used} / ${total} 已占用`,
    },
    wsl: {
      title: "WSL 信息",
      description:
        "默认发行版与发行版数量来自已安装列表缓存；点击顶部刷新会连同 WSL 版本信息一起更新。",
      copied: "已复制",
      copy: "复制",
      status: {
        notSet: "未设置",
        recovering: "状态暂未确认",
      },
      fields: {
        defaultDistro: "默认发行版",
        direct3d: "Direct3D 版本",
        distroCount: "发行版数量",
        dxcore: "DXCore 版本",
        kernel: "内核版本",
        msrdc: "MSRDC 版本",
        wsl: "WSL 版本",
        wslg: "WSLg 版本",
      },
    },
  },
  queryCache: {
    errors: {
      distros: "发行版列表读取失败。",
      wslVersion: "WSL 版本读取失败。",
      systemOverview: "系统信息读取失败。",
      onlineDistros: "在线发行版列表读取失败。",
    },
    recovering: {
      subjects: {
        distros: "发行版列表读取超时",
        wslVersion: "WSL 版本读取超时",
        systemOverview: "宿主机信息探测超时",
        onlineDistros: "在线发行版列表读取超时",
        hostCommandTimedOut: "宿主机信息探测超时",
      },
      withFallbackData: (subject: string) =>
        `${subject}，当前展示的是上次成功结果，可稍后手动刷新。`,
      withoutFallbackData: (subject: string) =>
        `${subject}，当前状态暂未确认，可稍后手动刷新。`,
    },
  },
  settings: {
    page: {
      eyebrow: "Settings",
      title: "设置",
    },
    language: {
      label: "语言",
      options: {
        "en-US": "English",
        "zh-CN": "简体中文",
      },
    },
    defaultInstallLocation: {
      label: "默认安装位置",
      dialogTitle: "选择默认安装位置",
    },
    backgroundRefresh: {
      intervalLabel: "后台刷新间隔（分钟）",
      targetsLabel: "后台刷新项目",
      targets: {
        distros: "发行版列表",
        systemOverviewStorage: "系统磁盘信息",
        wslVersion: "WSL 版本",
        onlineDistros: "在线发行版列表",
      },
    },
    validation: {
      defaultInstallLocationRequired: "默认安装位置不能为空。",
      backgroundTargetsRequired: "后台刷新项目不能为空。",
      backgroundIntervalInteger: "后台刷新间隔必须是整数分钟。",
      backgroundIntervalRange: (min: number, max: number) =>
        `后台刷新间隔必须在 ${min} 到 ${max} 分钟之间。`,
    },
    errors: {
      noTauriDirectoryPicker: "未检测到 Tauri 运行时，无法打开目录选择器。",
      noTauriSave: "未检测到 Tauri 运行时，无法保存设置。",
      saveFailed: "保存失败",
    },
    actions: {
      saving: "保存中",
      saveSettings: "保存设置",
    },
  },
  shell: {
    navigation: {
      label: "导航",
      primaryAriaLabel: "主导航",
      items: {
        overview: "概览",
        distros: "发行版",
        acquire: "安装",
        settings: "设置",
      },
    },
    sidebar: {
      collapse: "折叠侧栏",
      expand: "展开侧栏",
    },
    titlebar: {
      close: "关闭",
      maximize: "最大化",
      minimize: "最小化",
      resize: (direction: string) => `调整窗口大小 ${direction}`,
      resizeDirections: {
        North: "上边",
        South: "下边",
        East: "右边",
        West: "左边",
        NorthEast: "右上角",
        NorthWest: "左上角",
        SouthEast: "右下角",
        SouthWest: "左下角",
      },
      restore: "还原",
    },
  },
} as const satisfies DeepPartial<WidenCopy<EnUsCopy>>;

export default zhCN;
