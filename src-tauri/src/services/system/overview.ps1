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
  gpus = $null
  storage = [ordered]@{
    totalBytes = if ($null -ne $totalStorage) { [uint64]$totalStorage } else { $null }
    usedBytes = if ($null -ne $usedStorage) { [uint64]$usedStorage } else { $null }
    freeBytes = if ($null -ne $freeStorage) { [uint64]$freeStorage } else { $null }
    volumeCount = [uint32]$volumes.Count
  }
}

if ($scope -eq 'full') {
  $currentVersion = Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'
  $operatingSystem = Get-CimInstance Win32_OperatingSystem | Select-Object -First 1
  $processors = @(Get-CimInstance Win32_Processor)
  $memoryModules = @(Get-CimInstance Win32_PhysicalMemory)
  $memoryArrays = @(Get-CimInstance Win32_PhysicalMemoryArray)
  $videoControllers = @(
    Get-CimInstance Win32_VideoController |
      Where-Object {
        -not [string]::IsNullOrWhiteSpace($_.Name) -and
        -not [string]::IsNullOrWhiteSpace($_.PNPDeviceID) -and
        (
          $_.PNPDeviceID.StartsWith('PCI\', [System.StringComparison]::OrdinalIgnoreCase) -or
          $_.PNPDeviceID.IndexOf('ACPI', [System.StringComparison]::OrdinalIgnoreCase) -ge 0
        )
      }
  )

  $gpuMemoryByMatchingId = @{}
  $displayAdapterClassPath = 'HKLM:\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}'
  if (Test-Path $displayAdapterClassPath) {
    Get-ChildItem $displayAdapterClassPath -ErrorAction SilentlyContinue | ForEach-Object {
      try {
        $adapterKey = Get-ItemProperty $_.PSPath -ErrorAction Stop
        $matchingDeviceId = $adapterKey.MatchingDeviceId
        if (-not [string]::IsNullOrWhiteSpace($matchingDeviceId)) {
          $memorySize = $null
          if ($null -ne $adapterKey.'HardwareInformation.qwMemorySize') {
            $memorySize = [uint64]$adapterKey.'HardwareInformation.qwMemorySize'
          } elseif ($null -ne $adapterKey.'HardwareInformation.MemorySize' -and
            -not ($adapterKey.'HardwareInformation.MemorySize' -is [byte[]])) {
            $memorySize = [uint64]$adapterKey.'HardwareInformation.MemorySize'
          }

          if ($null -ne $memorySize -and $memorySize -gt 0) {
            $gpuMemoryByMatchingId[$matchingDeviceId.ToUpperInvariant()] = $memorySize
          }
        }
      } catch {
      }
    }
  }

  function Get-GpuMemoryBytes {
    param(
      [Parameter(Mandatory = $true)] $Gpu,
      [Parameter(Mandatory = $true)] [hashtable] $MemoryByMatchingId
    )

    $pnpDeviceId = if ($Gpu.PNPDeviceID) { $Gpu.PNPDeviceID.ToUpperInvariant() } else { '' }
    foreach ($matchingId in $MemoryByMatchingId.Keys) {
      if ($pnpDeviceId.Contains($matchingId)) {
        return [uint64]$MemoryByMatchingId[$matchingId]
      }
    }

    if ($Gpu.AdapterRAM) {
      return [uint64]$Gpu.AdapterRAM
    }

    return $null
  }

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
  $cpu = $processors | Select-Object -First 1
  $maxCpuClock = ($processors | Measure-Object -Property MaxClockSpeed -Maximum).Maximum
  $coreCount = ($processors | Measure-Object -Property NumberOfCores -Sum).Sum
  $logicalProcessorCount = ($processors | Measure-Object -Property NumberOfLogicalProcessors -Sum).Sum
  $windowsProductName = $operatingSystem.Caption
  if (-not [string]::IsNullOrWhiteSpace($windowsProductName)) {
    $windowsProductName = $windowsProductName -replace '^Microsoft\s+', ''
  }

  $result.windows = [ordered]@{
    productName = $windowsProductName
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
    maxClockMhz = if ($maxCpuClock) { [uint32]$maxCpuClock } else { $null }
    coreCount = if ($coreCount) { [uint32]$coreCount } else { $null }
    logicalProcessorCount = if ($logicalProcessorCount) {
      [uint32]$logicalProcessorCount
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
  $result.gpus = @(
    $videoControllers | ForEach-Object {
      [ordered]@{
        name = $_.Name
        vendor = $_.AdapterCompatibility
        memoryBytes = Get-GpuMemoryBytes -Gpu $_ -MemoryByMatchingId $gpuMemoryByMatchingId
        driverVersion = $_.DriverVersion
      }
    }
  )
}

$result | ConvertTo-Json -Depth 6 -Compress
