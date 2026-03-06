# GraphDB C API 测试构建脚本
# 用于 Windows PowerShell 环境

param(
    [string]$BuildMode = "debug",
    [switch]$Clean = $false,
    [switch]$Run = $false
)

# 设置错误处理
$ErrorActionPreference = "Stop"

# 获取脚本所在目录
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectRoot = Split-Path -Parent (Split-Path -Parent $ScriptDir)

# 设置路径
$IncludeDir = Join-Path $ProjectRoot "include"
$SourceDir = $ScriptDir
$BuildDir = Join-Path $ScriptDir "build"
$OutputDir = Join-Path $BuildDir "bin"
$LibDir = Join-Path $ProjectRoot "target\$BuildMode"

# 创建构建目录
if ($Clean -and (Test-Path $BuildDir)) {
    Write-Host "清理构建目录..." -ForegroundColor Yellow
    Remove-Item -Path $BuildDir -Recurse -Force
}

New-Item -ItemType Directory -Path $BuildDir -Force | Out-Null
New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  GraphDB C API 测试构建脚本" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

Write-Host "配置信息:" -ForegroundColor Green
Write-Host "  项目根目录: $ProjectRoot"
Write-Host "  包含目录: $IncludeDir"
Write-Host "  源文件目录: $SourceDir"
Write-Host "  构建目录: $BuildDir"
Write-Host "  输出目录: $OutputDir"
Write-Host "  库目录: $LibDir"
Write-Host "  构建模式: $BuildMode"
Write-Host ""

# 检查编译器
$Compiler = $null
if (Get-Command gcc -ErrorAction SilentlyContinue) {
    $Compiler = "gcc"
    Write-Host "检测到编译器: GCC" -ForegroundColor Green
} elseif (Get-Command clang -ErrorAction SilentlyContinue) {
    $Compiler = "clang"
    Write-Host "检测到编译器: Clang" -ForegroundColor Green
} elseif (Get-Command cl -ErrorAction SilentlyContinue) {
    $Compiler = "cl"
    Write-Host "检测到编译器: MSVC" -ForegroundColor Green
} else {
    Write-Host "错误: 未找到 C 编译器 (GCC, Clang 或 MSVC)" -ForegroundColor Red
    exit 1
}

Write-Host ""

# 检查库文件
$LibName = if ($IsWindows) { "graphdb.lib" } else { "libgraphdb.a" }
$LibPath = Join-Path $LibDir $LibName

if (-not (Test-Path $LibPath)) {
    Write-Host "警告: 未找到 GraphDB 库文件: $LibPath" -ForegroundColor Yellow
    Write-Host "请先构建 GraphDB 项目: cargo build --$BuildMode" -ForegroundColor Yellow
    Write-Host ""
    
    # 询问是否继续
    $Continue = Read-Host "是否继续构建测试? (y/n)"
    if ($Continue -ne "y" -and $Continue -ne "Y") {
        exit 1
    }
}

# 编译选项
$SourceFile = Join-Path $SourceDir "tests.c"
$OutputFile = Join-Path $OutputDir "graphdb_c_api_tests.exe"

Write-Host "编译测试程序..." -ForegroundColor Cyan

if ($Compiler -eq "gcc" -or $Compiler -eq "clang") {
    # GCC/Clang 编译命令
    $CompileCmd = @(
        $Compiler,
        "-Wall", "-Wextra",
        "-I$IncludeDir",
        "-L$LibDir",
        "-o", $OutputFile,
        $SourceFile,
        "-lgraphdb"
    )
    
    if ($IsWindows) {
        $CompileCmd += "-lws2_32"
    } else {
        $CompileCmd += @("-lpthread", "-ldl", "-lm")
    }
    
    Write-Host "执行命令: $($CompileCmd -join ' ')" -ForegroundColor DarkGray
    & $Compiler $CompileCmd[1..$CompileCmd.Length]
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "编译失败!" -ForegroundColor Red
        exit $LASTEXITCODE
    }
} elseif ($Compiler -eq "cl") {
    # MSVC 编译命令
    $CompileCmd = @(
        "cl.exe",
        "/W4",
        "/I$IncludeDir",
        "/Fe$OutputFile",
        $SourceFile,
        "/link",
        "/LIBPATH:$LibDir",
        "graphdb.lib",
        "ws2_32.lib"
    )
    
    Write-Host "执行命令: $($CompileCmd -join ' ')" -ForegroundColor DarkGray
    & cl.exe $CompileCmd[1..$CompileCmd.Length]
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host "编译失败!" -ForegroundColor Red
        exit $LASTEXITCODE
    }
}

Write-Host "编译成功!" -ForegroundColor Green
Write-Host "输出文件: $OutputFile" -ForegroundColor Green
Write-Host ""

# 运行测试
if ($Run) {
    Write-Host "运行测试..." -ForegroundColor Cyan
    Write-Host ""
    
    # 设置库路径
    if ($IsWindows) {
        $env:PATH = "$LibDir;$env:PATH"
    }
    
    & $OutputFile
    
    if ($LASTEXITCODE -ne 0) {
        Write-Host ""
        Write-Host "测试失败!" -ForegroundColor Red
        exit $LASTEXITCODE
    }
    
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Green
    Write-Host "  所有测试通过!" -ForegroundColor Green
    Write-Host "========================================" -ForegroundColor Green
} else {
    Write-Host "提示: 使用 -Run 参数运行测试" -ForegroundColor Yellow
    Write-Host "示例: .\build.ps1 -Run" -ForegroundColor Yellow
}

Write-Host ""
