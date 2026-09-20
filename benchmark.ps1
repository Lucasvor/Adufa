param(
    [ValidateRange(3, 3600)]
    [int]$HoldSeconds = 10,
    [ValidateRange(1, 50)]
    [int]$Runs = 7,
    [ValidateRange(0, 10)]
    [int]$WarmupRuns = 1,
    [ValidateSet('language', 'dotnet')]
    [string]$Suite = 'language',
    [switch]$SkipBuild,
    [switch]$VerboseOutput
)

$ErrorActionPreference = 'Stop'
$root = $PSScriptRoot
$artifacts = Join-Path $root 'artifacts'
$csharpOutput = Join-Path $artifacts 'csharp'
$csharpNet10Output = Join-Path $artifacts 'csharp-net10'
$csharpNet11Output = Join-Path $artifacts 'csharp-net11'
$rustOutput = Join-Path $artifacts 'rust'

if (-not (Get-Command dotnet -ErrorAction SilentlyContinue)) {
    throw '.NET SDK was not found.'
}
$cargoCommand = Get-Command cargo -ErrorAction SilentlyContinue
$cargoPath = if ($cargoCommand) { $cargoCommand.Source } else { $null }
if (-not $cargoPath) {
    $cargoFallback = Join-Path $env:USERPROFILE '.cargo\bin\cargo.exe'
    if (Test-Path $cargoFallback) { $cargoPath = $cargoFallback }
}
if (-not $cargoPath) {
    throw 'Rust was not found. Install it with: winget install Rustlang.Rustup'
}

New-Item -ItemType Directory -Force -Path $csharpOutput, $csharpNet10Output, $csharpNet11Output, $rustOutput | Out-Null

if ($Suite -eq 'language') {
    if (-not $SkipBuild) {
        dotnet publish (Join-Path $root 'src\csharp\AudioProbe.csproj') -f net11.0 -c Release -r win-x64 --self-contained true -p:NuGetAudit=false -o $csharpOutput
        if ($LASTEXITCODE -ne 0) { throw 'C# NativeAOT build failed.' }

        & $cargoPath build --manifest-path (Join-Path $root 'src\rust\Cargo.toml') --release
        if ($LASTEXITCODE -ne 0) { throw 'Rust release build failed.' }
    }
    Copy-Item (Join-Path $root 'src\rust\target\release\audiorouter-rust.exe') $rustOutput -Force

    $probes = @(
        [pscustomobject]@{ Language = 'csharp-net11'; Executable = Join-Path $csharpOutput 'AudioProbeCSharp.exe' }
        [pscustomobject]@{ Language = 'rust'; Executable = Join-Path $rustOutput 'audiorouter-rust.exe' }
    )
    $runsFile = 'benchmark-runs.csv'
    $resultsFile = 'benchmark-results.csv'
} else {
    if (-not $SkipBuild) {
        dotnet publish (Join-Path $root 'src\csharp\AudioProbe.csproj') -f net10.0 -c Release -r win-x64 --self-contained true -p:NuGetAudit=false -o $csharpNet10Output
        if ($LASTEXITCODE -ne 0) { throw '.NET 10 NativeAOT build failed.' }
        dotnet publish (Join-Path $root 'src\csharp\AudioProbe.csproj') -f net11.0 -c Release -r win-x64 --self-contained true -p:NuGetAudit=false -o $csharpNet11Output
        if ($LASTEXITCODE -ne 0) { throw '.NET 11 NativeAOT build failed.' }
    }
    $probes = @(
        [pscustomobject]@{ Language = 'csharp-net10'; Executable = Join-Path $csharpNet10Output 'AudioProbeCSharp.exe' }
        [pscustomobject]@{ Language = 'csharp-net11'; Executable = Join-Path $csharpNet11Output 'AudioProbeCSharp.exe' }
    )
    $runsFile = 'benchmark-runs-dotnet.csv'
    $resultsFile = 'benchmark-results-dotnet.csv'
}

function Measure-Probe {
    param(
        [string]$Language,
        [string]$Executable,
        [int]$Seconds,
        [int]$Run,
        [string]$Phase
    )

    $stdout = Join-Path $artifacts "$Phase-$Language-$Run.stdout.txt"
    $stderr = Join-Path $artifacts "$Phase-$Language-$Run.stderr.txt"
    $process = Start-Process -FilePath $Executable -ArgumentList '--hold', $Seconds -PassThru -RedirectStandardOutput $stdout -RedirectStandardError $stderr
    Start-Sleep -Seconds 2
    $process.Refresh()
    if ($process.HasExited) {
        throw "$Language exited early: $(Get-Content -Raw $stderr)"
    }

    $cpuBefore = $process.TotalProcessorTime.TotalMilliseconds
    $workingSet = $process.WorkingSet64
    $privateMemory = $process.PrivateMemorySize64
    $threads = $process.Threads.Count
    $handles = $process.HandleCount
    $sampleSeconds = [Math]::Min(5, [Math]::Max(1, $Seconds - 2))
    Start-Sleep -Seconds $sampleSeconds
    $process.Refresh()
    $cpuIdleMsPerSecond = ($process.TotalProcessorTime.TotalMilliseconds - $cpuBefore) / $sampleSeconds
    $process.WaitForExit()

    if ($VerboseOutput) {
        Get-Content $stdout -Encoding utf8 | ForEach-Object { Write-Host $_ }
        if ((Get-Item $stderr).Length -gt 0) {
            Get-Content $stderr -Encoding utf8 | ForEach-Object { Write-Host $_ }
        }
    }
    $ready = Get-Content $stdout -Encoding utf8 | Where-Object { $_ -like 'READY *' } | Select-Object -Last 1
    if (-not $ready) { throw "$Language did not emit READY." }
    $startup = if ($ready -match 'startup_ms=([0-9.]+)') { [double]$Matches[1] } else { [double]::NaN }
    $scan = if ($ready -match 'scan_ms=([0-9.]+)') { [double]$Matches[1] } else { [double]::NaN }

    [pscustomobject]@{
        Language = $Language
        Run = $Run
        ExeMB = [Math]::Round((Get-Item $Executable).Length / 1MB, 3)
        StartupMs = $startup
        ScanMs = $scan
        WorkingSetMB = [Math]::Round($workingSet / 1MB, 3)
        PrivateMemoryMB = [Math]::Round($privateMemory / 1MB, 3)
        IdleCpuMsPerSecond = [Math]::Round($cpuIdleMsPerSecond, 3)
        Threads = $threads
        Handles = $handles
    }
}

function Get-Percentile {
    param([double[]]$Values, [double]$Percentile)
    $sorted = @($Values | Sort-Object)
    $index = [Math]::Max(0, [Math]::Ceiling($Percentile * $sorted.Count) - 1)
    return $sorted[$index]
}

function Get-Median {
    param([double[]]$Values)
    $sorted = @($Values | Sort-Object)
    $middle = [Math]::Floor($sorted.Count / 2)
    if ($sorted.Count % 2) { return $sorted[$middle] }
    return ($sorted[$middle - 1] + $sorted[$middle]) / 2
}

if ($WarmupRuns -gt 0) {
    Write-Host "Warming up ($WarmupRuns run(s) per executable)..."
    for ($warmup = 1; $warmup -le $WarmupRuns; $warmup++) {
        foreach ($probe in $probes) {
            $null = Measure-Probe $probe.Language $probe.Executable 3 $warmup 'warmup'
        }
    }
}

$samples = @()
for ($run = 1; $run -le $Runs; $run++) {
    $order = if ($run % 2) { $probes } else { @($probes[1], $probes[0]) }
    foreach ($probe in $order) {
        Write-Host "Measured run $run/${Runs}: $($probe.Language)"
        $samples += Measure-Probe $probe.Language $probe.Executable $HoldSeconds $run 'measured'
    }
}

$summary = foreach ($probe in $probes) {
    $rows = @($samples | Where-Object Language -eq $probe.Language)
    [pscustomobject]@{
        Language = $probe.Language
        Runs = $rows.Count
        ExeMB = $rows[0].ExeMB
        StartupMedianMs = [Math]::Round((Get-Median $rows.StartupMs), 3)
        StartupP95Ms = [Math]::Round((Get-Percentile $rows.StartupMs 0.95), 3)
        ScanMedianMs = [Math]::Round((Get-Median $rows.ScanMs), 3)
        WorkingSetMedianMB = [Math]::Round((Get-Median $rows.WorkingSetMB), 3)
        PrivateMemoryMedianMB = [Math]::Round((Get-Median $rows.PrivateMemoryMB), 3)
        IdleCpuMedianMsPerSecond = [Math]::Round((Get-Median $rows.IdleCpuMsPerSecond), 3)
        ThreadsMedian = [Math]::Round((Get-Median $rows.Threads), 1)
        HandlesMedian = [Math]::Round((Get-Median $rows.Handles), 1)
    }
}

$samples | Sort-Object Run, Language | Export-Csv (Join-Path $artifacts $runsFile) -NoTypeInformation
$summary | Export-Csv (Join-Path $artifacts $resultsFile) -NoTypeInformation
$summary | Format-Table -AutoSize
