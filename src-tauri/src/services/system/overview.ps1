$ErrorActionPreference = 'Stop'
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$OutputEncoding = [System.Text.UTF8Encoding]::new($false)

$scope = if ([string]::IsNullOrWhiteSpace($env:EAWSL_SYSTEM_OVERVIEW_SCOPE)) {
  'full'
} else {
  $env:EAWSL_SYSTEM_OVERVIEW_SCOPE.Trim().ToLowerInvariant()
}

$volumes = @(Get-CimInstance Win32_LogicalDisk -Filter "DriveType = 3")

$totalStorage = ($volumes | Measure-Object -Property Size -Sum).Sum
$freeStorage = ($volumes | Measure-Object -Property FreeSpace -Sum).Sum
$usedStorage = if ($null -ne $totalStorage -and $null -ne $freeStorage) {
  [uint64]$totalStorage - [uint64]$freeStorage
} else {
  $null
}

$result = [ordered]@{
  windows = [ordered]@{
    productName = $null
    displayVersion = $null
    buildNumber = $null
  }
  cpu = [ordered]@{
    model = $null
    maxClockMhz = $null
    coreCount = $null
    logicalProcessorCount = $null
  }
  memory = [ordered]@{
    totalBytes = $null
    speedMts = $null
    usedSlots = $null
    totalSlots = $null
  }
  gpu = $null
  storage = [ordered]@{
    totalBytes = if ($null -ne $totalStorage) { [uint64]$totalStorage } else { $null }
    usedBytes = if ($null -ne $usedStorage) { [uint64]$usedStorage } else { $null }
    freeBytes = if ($null -ne $freeStorage) { [uint64]$freeStorage } else { $null }
    volumeCount = [uint32]$volumes.Count
  }
}

if ($scope -eq 'full') {
  $currentVersion = Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'
  $cpu = Get-CimInstance Win32_Processor | Select-Object -First 1
  $memoryModules = @(Get-CimInstance Win32_PhysicalMemory)
  $memoryArrays = @(Get-CimInstance Win32_PhysicalMemoryArray)
  $videoControllers = @(
    Get-CimInstance Win32_VideoController |
      Where-Object { -not [string]::IsNullOrWhiteSpace($_.Name) }
  )
  $gpu = $videoControllers |
    Sort-Object -Property @{ Expression = { [uint64]($_.AdapterRAM) }; Descending = $true } |
    Select-Object -First 1

  $totalMemory = ($memoryModules | Measure-Object -Property Capacity -Sum).Sum
  $memorySpeed = (
    $memoryModules |
      ForEach-Object {
        if ($_.ConfiguredClockSpeed -and $_.ConfiguredClockSpeed -gt 0) {
          [uint32]$_.ConfiguredClockSpeed
        } elseif ($_.Speed -and $_.Speed -gt 0) {
          [uint32]$_.Speed
        }
      } |
      Measure-Object -Maximum
  ).Maximum
  $usedSlots = ($memoryModules | Where-Object { $_.Capacity -gt 0 }).Count
  $totalSlots = ($memoryArrays | Measure-Object -Property MemoryDevices -Sum).Sum

  $result.windows = [ordered]@{
    productName = $currentVersion.ProductName
    displayVersion = if ($currentVersion.DisplayVersion) {
      $currentVersion.DisplayVersion
    } else {
      $currentVersion.ReleaseId
    }
    buildNumber = if ($currentVersion.CurrentBuildNumber -and $null -ne $currentVersion.UBR) {
      "$($currentVersion.CurrentBuildNumber).$($currentVersion.UBR)"
    } else {
      $currentVersion.CurrentBuildNumber
    }
  }
  $result.cpu = [ordered]@{
    model = $cpu.Name
    maxClockMhz = if ($cpu.MaxClockSpeed) { [uint32]$cpu.MaxClockSpeed } else { $null }
    coreCount = if ($cpu.NumberOfCores) { [uint32]$cpu.NumberOfCores } else { $null }
    logicalProcessorCount = if ($cpu.NumberOfLogicalProcessors) {
      [uint32]$cpu.NumberOfLogicalProcessors
    } else {
      $null
    }
  }
  $result.memory = [ordered]@{
    totalBytes = if ($null -ne $totalMemory) { [uint64]$totalMemory } else { $null }
    speedMts = if ($null -ne $memorySpeed) { [uint32]$memorySpeed } else { $null }
    usedSlots = if ($null -ne $usedSlots) { [uint32]$usedSlots } else { $null }
    totalSlots = if ($null -ne $totalSlots) { [uint32]$totalSlots } else { $null }
  }
  $result.gpu = if ($null -ne $gpu) {
    [ordered]@{
      name = $gpu.Name
      memoryBytes = if ($gpu.AdapterRAM) { [uint64]$gpu.AdapterRAM } else { $null }
      driverVersion = $gpu.DriverVersion
    }
  } else {
    $null
  }
}

$result | ConvertTo-Json -Depth 6 -Compress
