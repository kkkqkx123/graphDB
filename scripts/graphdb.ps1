# GraphDB 服务管理脚本
# 用于单实例个人使用的数据库服务管理
# 功能：启动、停止、状态检查、日志查看

param(
    [Parameter(Mandatory=$false)]
    [ValidateSet("start", "stop", "restart", "status", "logs", "cli", "info")]
    [string]$Command = "status",
    
    [Parameter(Mandatory=$false)]
    [string]$ConfigPath = "config.toml",
    
    [Parameter(Mandatory=$false)]
    [string]$DataDir = "data",
    
    [Parameter(Mandatory=$false)]
    [string]$LogDir = "logs",
    
    [Parameter(Mandatory=$false)]
    [switch]$Foreground = $false,
    
    [Parameter(Mandatory=$false)]
    [int]$Lines = 50
)

# 配置
$Script:ServiceName = "graphdb"
$Script:PidFile = "$DataDir\graphdb.pid"
$Script:DefaultPort = 9758
$Script:ExeName = "graphdb.exe"

# 颜色输出配置
$Colors = @{
    Info = "Cyan"
    Success = "Green"
    Warning = "Yellow"
    Error = "Red"
    Normal = "White"
}

function Write-ColorOutput {
    param(
        [string]$Message,
        [string]$Color = "White",
        [switch]$NoNewline
    )
    $params = @{
        Object = $Message
        ForegroundColor = $Colors[$Color]
        NoNewline = $NoNewline
    }
    Write-Host @params
}

function Get-GraphDBProcess {
    <#
    .SYNOPSIS
        获取 GraphDB 进程信息
    #>
    # 首先尝试通过 PID 文件查找
    if (Test-Path $Script:PidFile) {
        try {
            $pid = Get-Content $Script:PidFile -Raw
            $process = Get-Process -Id $pid -ErrorAction SilentlyContinue
            if ($process -and $process.ProcessName -match "graphdb") {
                return $process
            }
        } catch {
            # PID 文件存在但进程不存在，清理 PID 文件
            Remove-Item $Script:PidFile -Force -ErrorAction SilentlyContinue
        }
    }
    
    # 通过进程名查找
    $process = Get-Process | Where-Object { 
        $_.ProcessName -match "graphdb" -or 
        $_.Path -match "graphdb\.exe$" 
    } | Select-Object -First 1
    
    return $process
}

function Test-PortInUse {
    param([int]$Port)
    <#
    .SYNOPSIS
        检查端口是否被占用
    #>
    try {
        $listener = [System.Net.Sockets.TcpListener]::new([System.Net.IPAddress]::Any, $Port)
        $listener.Start()
        $listener.Stop()
        return $false
    } catch {
        return $true
    }
}

function Get-ServiceStatus {
    <#
    .SYNOPSIS
        获取服务状态信息
    #>
    $process = Get-GraphDBProcess
    $config = @{}
    
    # 读取配置
    if (Test-Path $ConfigPath) {
        try {
            $content = Get-Content $ConfigPath -Raw
            # 简单解析 TOML
            if ($content -match 'port\s*=\s*(\d+)') {
                $config.Port = [int]$matches[1]
            } else {
                $config.Port = $Script:DefaultPort
            }
            if ($content -match 'host\s*=\s*"([^"]+)"') {
                $config.Host = $matches[1]
            } else {
                $config.Host = "127.0.0.1"
            }
        } catch {
            $config.Port = $Script:DefaultPort
            $config.Host = "127.0.0.1"
        }
    } else {
        $config.Port = $Script:DefaultPort
        $config.Host = "127.0.0.1"
    }
    
    $status = @{
        Running = $null -ne $process
        Process = $process
        Pid = if ($process) { $process.Id } else { $null }
        Config = $config
        PortInUse = Test-PortInUse -Port $config.Port
        DataDir = Resolve-Path $DataDir -ErrorAction SilentlyContinue
        LogDir = Resolve-Path $LogDir -ErrorAction SilentlyContinue
    }
    
    return $status
}

function Start-GraphDBService {
    <#
    .SYNOPSIS
        启动 GraphDB 服务
    #>
    Write-ColorOutput "正在启动 GraphDB 服务..." -Color "Info"
    
    # 检查是否已运行
    $existingProcess = Get-GraphDBProcess
    if ($existingProcess) {
        Write-ColorOutput "GraphDB 服务已在运行 (PID: $($existingProcess.Id))" -Color "Warning"
        return
    }
    
    # 检查端口占用
    $status = Get-ServiceStatus
    if ($status.PortInUse) {
        Write-ColorOutput "端口 $($status.Config.Port) 已被占用" -Color "Error"
        return
    }
    
    # 检查配置文件
    if (-not (Test-Path $ConfigPath)) {
        Write-ColorOutput "配置文件不存在: $ConfigPath" -Color "Warning"
        Write-ColorOutput "将使用默认配置启动" -Color "Info"
    }
    
    # 创建数据目录
    if (-not (Test-Path $DataDir)) {
        New-Item -ItemType Directory -Path $DataDir -Force | Out-Null
        Write-ColorOutput "创建数据目录: $DataDir" -Color "Info"
    }
    
    # 创建日志目录
    if (-not (Test-Path $LogDir)) {
        New-Item -ItemType Directory -Path $LogDir -Force | Out-Null
        Write-ColorOutput "创建日志目录: $LogDir" -Color "Info"
    }
    
    # 查找可执行文件
    $exePaths = @(
        ".\$Script:ExeName"
        ".\target\release\$Script:ExeName"
        ".\target\debug\$Script:ExeName"
    )
    
    $exePath = $null
    foreach ($path in $exePaths) {
        if (Test-Path $path) {
            $exePath = Resolve-Path $path
            break
        }
    }
    
    if (-not $exePath) {
        Write-ColorOutput "找不到 GraphDB 可执行文件" -Color "Error"
        Write-ColorOutput "请确保 graphdb.exe 在当前目录或 target/release/ 目录中" -Color "Normal"
        return
    }
    
    Write-ColorOutput "使用可执行文件: $exePath" -Color "Info"
    
    if ($Foreground) {
        # 前台运行
        Write-ColorOutput "前台启动 GraphDB 服务..." -Color "Info"
        Write-ColorOutput "按 Ctrl+C 停止服务" -Color "Warning"
        & $exePath serve --config $ConfigPath
    } else {
        # 后台运行
        $job = Start-Job -ScriptBlock {
            param($exe, $config)
            & $exe serve --config $config
        } -ArgumentList $exePath, $ConfigPath
        
        # 等待服务启动
        Start-Sleep -Seconds 2
        
        # 获取实际进程
        $process = Get-GraphDBProcess
        if ($process) {
            # 保存 PID
            $process.Id | Out-File $Script:PidFile -Force
            Write-ColorOutput "GraphDB 服务已启动" -Color "Success"
            Write-ColorOutput "  PID: $($process.Id)" -Color "Normal"
            Write-ColorOutput "  地址: http://$($status.Config.Host):$($status.Config.Port)" -Color "Normal"
            Write-ColorOutput "  数据目录: $(Resolve-Path $DataDir)" -Color "Normal"
            Write-ColorOutput "  日志目录: $(Resolve-Path $LogDir)" -Color "Normal"
            Write-ColorOutput "  查看日志: .\scripts\graphdb.ps1 logs" -Color "Info"
            Write-ColorOutput "  停止服务: .\scripts\graphdb.ps1 stop" -Color "Info"
        } else {
            Write-ColorOutput "服务启动失败，请检查日志" -Color "Error"
            Receive-Job $job
            Remove-Job $job -Force
        }
    }
}

function Stop-GraphDBService {
    <#
    .SYNOPSIS
        停止 GraphDB 服务
    #>
    Write-ColorOutput "正在停止 GraphDB 服务..." -Color "Info"
    
    $process = Get-GraphDBProcess
    
    if (-not $process) {
        Write-ColorOutput "GraphDB 服务未运行" -Color "Warning"
        # 清理 PID 文件
        if (Test-Path $Script:PidFile) {
            Remove-Item $Script:PidFile -Force -ErrorAction SilentlyContinue
        }
        return
    }
    
    Write-ColorOutput "找到进程 PID: $($process.Id)" -Color "Info"
    
    try {
        # 优雅终止
        $process.CloseMainWindow() | Out-Null
        Start-Sleep -Seconds 2
        
        # 检查是否仍在运行
        if (-not $process.HasExited) {
            # 强制终止
            Stop-Process -Id $process.Id -Force -ErrorAction Stop
        }
        
        Write-ColorOutput "GraphDB 服务已停止" -Color "Success"
    } catch {
        Write-ColorOutput "停止服务时出错: $_" -Color "Error"
    } finally {
        # 清理 PID 文件
        if (Test-Path $Script:PidFile) {
            Remove-Item $Script:PidFile -Force -ErrorAction SilentlyContinue
        }
    }
}

function Show-ServiceStatus {
    <#
    .SYNOPSIS
        显示服务状态
    #>
    $status = Get-ServiceStatus
    
    Write-ColorOutput "GraphDB 服务状态" -Color "Info"
    Write-ColorOutput "==================" -Color "Info"
    
    if ($status.Running) {
        Write-ColorOutput "状态: 运行中" -Color "Success"
        Write-ColorOutput "PID: $($status.Pid)" -Color "Normal"
        
        $process = $status.Process
        Write-ColorOutput "启动时间: $($process.StartTime)" -Color "Normal"
        Write-ColorOutput "内存使用: $([math]::Round($process.WorkingSet64 / 1MB, 2)) MB" -Color "Normal"
        Write-ColorOutput "CPU时间: $($process.TotalProcessorTime)" -Color "Normal"
    } else {
        Write-ColorOutput "状态: 未运行" -Color "Warning"
    }
    
    Write-ColorOutput "" -Color "Normal"
    Write-ColorOutput "配置信息" -Color "Info"
    Write-ColorOutput "--------" -Color "Info"
    Write-ColorOutput "监听地址: http://$($status.Config.Host):$($status.Config.Port)" -Color "Normal"
    Write-ColorOutput "端口占用: $(if ($status.PortInUse) { '是' } else { '否' })" -Color $(if ($status.PortInUse) { "Warning" } else { "Normal" })
    Write-ColorOutput "数据目录: $($status.DataDir)" -Color "Normal"
    Write-ColorOutput "日志目录: $($status.LogDir)" -Color "Normal"
    Write-ColorOutput "配置文件: $(Resolve-Path $ConfigPath -ErrorAction SilentlyContinue)" -Color "Normal"
}

function Show-Logs {
    <#
    .SYNOPSIS
        显示日志
    #>
    $logFile = "$LogDir\graphdb.log"
    $rCURRENTLogFile = "$LogDir\graphdb.rCURRENT.log"
    
    # 检查日志文件
    $targetLog = $null
    if (Test-Path $rCURRENTLogFile) {
        $targetLog = $rCURRENTLogFile
    } elseif (Test-Path $logFile) {
        $targetLog = $logFile
    }
    
    if (-not $targetLog) {
        Write-ColorOutput "未找到日志文件" -Color "Warning"
        Write-ColorOutput "预期路径: $logFile" -Color "Normal"
        return
    }
    
    Write-ColorOutput "显示日志: $targetLog (最后 $Lines 行)" -Color "Info"
    Write-ColorOutput "==================" -Color "Info"
    
    if ($Lines -gt 0) {
        Get-Content $targetLog -Tail $Lines | ForEach-Object {
            # 根据日志级别着色
            if ($_ -match "ERROR") {
                Write-ColorOutput $_ -Color "Error"
            } elseif ($_ -match "WARN") {
                Write-ColorOutput $_ -Color "Warning"
            } elseif ($_ -match "INFO") {
                Write-ColorOutput $_ -Color "Info"
            } else {
                Write-ColorOutput $_ -Color "Normal"
            }
        }
    } else {
        # 实时跟踪日志
        Write-ColorOutput "正在跟踪日志 (按 Ctrl+C 退出)..." -Color "Warning"
        Get-Content $targetLog -Wait -Tail 10
    }
}

function Show-ServiceInfo {
    <#
    .SYNOPSIS
        显示服务详细信息
    #>
    Write-ColorOutput "GraphDB 服务信息" -Color "Info"
    Write-ColorOutput "==================" -Color "Info"
    Write-ColorOutput "" -Color "Normal"
    
    Write-ColorOutput "版本信息" -Color "Info"
    Write-ColorOutput "--------" -Color "Info"
    
    # 尝试获取版本
    $exePaths = @(
        ".\$Script:ExeName"
        ".\target\release\$Script:ExeName"
        ".\target\debug\$Script:ExeName"
    )
    
    foreach ($path in $exePaths) {
        if (Test-Path $path) {
            try {
                $version = & $path --version 2>&1
                Write-ColorOutput $version -Color "Normal"
            } catch {
                Write-ColorOutput "GraphDB 0.1.0" -Color "Normal"
            }
            break
        }
    }
    
    Write-ColorOutput "" -Color "Normal"
    Write-ColorOutput "数据存储" -Color "Info"
    Write-ColorOutput "--------" -Color "Info"
    
    $status = Get-ServiceStatus
    if ($status.DataDir) {
        $dataSize = (Get-ChildItem $DataDir -Recurse -ErrorAction SilentlyContinue | 
                    Measure-Object -Property Length -Sum).Sum
        Write-ColorOutput "数据目录: $($status.DataDir)" -Color "Normal"
        Write-ColorOutput "数据大小: $([math]::Round($dataSize / 1MB, 2)) MB" -Color "Normal"
        
        # 统计文件数量
        $fileCount = (Get-ChildItem $DataDir -Recurse -File -ErrorAction SilentlyContinue).Count
        Write-ColorOutput "文件数量: $fileCount" -Color "Normal"
    }
    
    Write-ColorOutput "" -Color "Normal"
    Write-ColorOutput "日志信息" -Color "Info"
    Write-ColorOutput "--------" -Color "Info"
    
    if ($status.LogDir) {
        $logFiles = Get-ChildItem $LogDir -Filter "*.log*" -ErrorAction SilentlyContinue
        $logSize = ($logFiles | Measure-Object -Property Length -Sum).Sum
        Write-ColorOutput "日志目录: $($status.LogDir)" -Color "Normal"
        Write-ColorOutput "日志大小: $([math]::Round($logSize / 1MB, 2)) MB" -Color "Normal"
        Write-ColorOutput "日志文件: $($logFiles.Count) 个" -Color "Normal"
    }
    
    Write-ColorOutput "" -Color "Normal"
    Write-ColorOutput "使用帮助" -Color "Info"
    Write-ColorOutput "--------" -Color "Info"
    Write-ColorOutput ".\scripts\graphdb.ps1 start    # 启动服务" -Color "Normal"
    Write-ColorOutput ".\scripts\graphdb.ps1 stop     # 停止服务" -Color "Normal"
    Write-ColorOutput ".\scripts\graphdb.ps1 restart  # 重启服务" -Color "Normal"
    Write-ColorOutput ".\scripts\graphdb.ps1 status   # 查看状态" -Color "Normal"
    Write-ColorOutput ".\scripts\graphdb.ps1 logs     # 查看日志" -Color "Normal"
    Write-ColorOutput ".\scripts\graphdb.ps1 info     # 详细信息" -Color "Normal"
}

function Start-GraphDBCli {
    <#
    .SYNOPSIS
        启动交互式 CLI
    #>
    $status = Get-ServiceStatus
    
    if (-not $status.Running) {
        Write-ColorOutput "GraphDB 服务未运行，请先启动服务" -Color "Error"
        Write-ColorOutput "使用: .\scripts\graphdb.ps1 start" -Color "Info"
        return
    }
    
    Write-ColorOutput "GraphDB 交互式 CLI" -Color "Info"
    Write-ColorOutput "输入 SQL 查询或命令 (exit 退出)" -Color "Info"
    Write-ColorOutput "==================" -Color "Info"
    
    while ($true) {
        Write-Host "graphdb> " -NoNewline -ForegroundColor Cyan
        $input = Read-Host
        
        if ($input -eq "exit") {
            break
        }
        
        if ([string]::IsNullOrWhiteSpace($input)) {
            continue
        }
        
        # 执行查询
        $exePaths = @(
            ".\$Script:ExeName"
            ".\target\release\$Script:ExeName"
            ".\target\debug\$Script:ExeName"
        )
        
        foreach ($path in $exePaths) {
            if (Test-Path $path) {
                & $path query --query $input
                break
            }
        }
    }
}

# 主入口
switch ($Command) {
    "start" {
        Start-GraphDBService
    }
    "stop" {
        Stop-GraphDBService
    }
    "restart" {
        Stop-GraphDBService
        Start-Sleep -Seconds 1
        Start-GraphDBService
    }
    "status" {
        Show-ServiceStatus
    }
    "logs" {
        Show-Logs
    }
    "info" {
        Show-ServiceInfo
    }
    "cli" {
        Start-GraphDBCli
    }
    default {
        Show-ServiceStatus
    }
}
