# Windows构建脚本 - 解决CPU指令兼容性问题
# 使用方法: .\build-windows.ps1 [compat|perf|auto]

param(
    [Parameter(Position=0)]
    [ValidateSet("compat", "perf", "auto")]
    [string]$Mode = "auto"
)

Write-Host "🚀 CGMiner CPU BTC Core - Windows构建脚本" -ForegroundColor Green
Write-Host "============================================" -ForegroundColor Green

# 检测CPU特性
function Test-CpuFeatures {
    Write-Host "🔍 检测CPU特性..." -ForegroundColor Yellow
    
    try {
        # 使用wmic获取CPU信息
        $cpu = Get-WmiObject -Class Win32_Processor | Select-Object -First 1
        Write-Host "  CPU: $($cpu.Name)" -ForegroundColor Cyan
        
        # 检查是否支持AVX2 (简化检测)
        $cpuFeatures = $cpu.Name.ToLower()
        $supportsModernFeatures = $cpuFeatures -match "intel.*core.*i[3-9]" -or 
                                  $cpuFeatures -match "amd.*ryzen" -or
                                  $cpuFeatures -match "intel.*xeon"
        
        return $supportsModernFeatures
    }
    catch {
        Write-Host "  ⚠️  无法检测CPU特性，使用兼容模式" -ForegroundColor Yellow
        return $false
    }
}

# 设置环境变量
function Set-BuildEnvironment {
    param([string]$BuildMode)
    
    Write-Host "⚙️  配置构建环境: $BuildMode" -ForegroundColor Yellow
    
    if ($BuildMode -eq "compat") {
        # 兼容模式 - 适用于所有x86_64 CPU
        $env:RUSTFLAGS = "-C target-cpu=x86-64 -C target-feature=+sse2,+sse4.1,+sse4.2 -C opt-level=3 -C overflow-checks=off"
        Write-Host "  📊 模式: 兼容性优先 (支持所有x86_64 CPU)" -ForegroundColor Green
    }
    elseif ($BuildMode -eq "perf") {
        # 性能模式 - 需要现代CPU
        $env:RUSTFLAGS = "-C target-cpu=native -C target-feature=+aes,+sha,+sse4.2,+avx2,+bmi2,+popcnt -C opt-level=3 -C overflow-checks=off"
        Write-Host "  🚀 模式: 性能优先 (需要现代CPU支持)" -ForegroundColor Green
    }
    
    # 禁用SIMD优化以避免指令冲突
    if ($BuildMode -eq "compat") {
        $env:CARGO_FEATURES = "thermal-management,power-management,cpu-affinity,temperature-monitoring"
    } else {
        $env:CARGO_FEATURES = "simd-optimizations,thermal-management,power-management,cpu-affinity,temperature-monitoring"
    }
}

# 主构建逻辑
function Start-Build {
    param([string]$BuildMode)
    
    Write-Host "🔨 开始构建..." -ForegroundColor Yellow
    
    # 清理之前的构建
    Write-Host "  🧹 清理之前的构建..." -ForegroundColor Cyan
    cargo clean
    
    # 设置构建环境
    Set-BuildEnvironment -BuildMode $BuildMode
    
    # 构建项目
    Write-Host "  🔧 编译项目..." -ForegroundColor Cyan
    $buildCmd = "cargo build --release --features=$env:CARGO_FEATURES"
    Write-Host "  执行: $buildCmd" -ForegroundColor Gray
    
    Invoke-Expression $buildCmd
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "  ✅ 构建成功!" -ForegroundColor Green
        
        # 构建示例
        Write-Host "  🔧 编译示例程序..." -ForegroundColor Cyan
        cargo build --release --examples --features=$env:CARGO_FEATURES
        
        if ($LASTEXITCODE -eq 0) {
            Write-Host "  ✅ 示例程序构建成功!" -ForegroundColor Green
            return $true
        } else {
            Write-Host "  ❌ 示例程序构建失败!" -ForegroundColor Red
            return $false
        }
    } else {
        Write-Host "  ❌ 构建失败!" -ForegroundColor Red
        return $false
    }
}

# 测试构建结果
function Test-Build {
    Write-Host "🧪 测试构建结果..." -ForegroundColor Yellow
    
    $exePath = "target\release\examples\basic_mining_demo.exe"
    if (Test-Path $exePath) {
        Write-Host "  📁 找到可执行文件: $exePath" -ForegroundColor Green
        
        # 尝试运行程序（限时5秒）
        Write-Host "  🚀 测试运行程序..." -ForegroundColor Cyan
        try {
            $job = Start-Job -ScriptBlock {
                param($path)
                & $path
            } -ArgumentList $exePath
            
            Wait-Job $job -Timeout 5 | Out-Null
            $output = Receive-Job $job
            Stop-Job $job -ErrorAction SilentlyContinue
            Remove-Job $job -ErrorAction SilentlyContinue
            
            if ($output -match "CGMiner CPU BTC Core") {
                Write-Host "  ✅ 程序运行正常!" -ForegroundColor Green
                return $true
            } else {
                Write-Host "  ⚠️  程序可能存在问题" -ForegroundColor Yellow
                return $false
            }
        }
        catch {
            Write-Host "  ❌ 程序运行失败: $($_.Exception.Message)" -ForegroundColor Red
            return $false
        }
    } else {
        Write-Host "  ❌ 未找到可执行文件!" -ForegroundColor Red
        return $false
    }
}

# 主程序
try {
    if ($Mode -eq "auto") {
        $supportsModern = Test-CpuFeatures
        if ($supportsModern) {
            Write-Host "🎯 自动选择: 性能模式" -ForegroundColor Green
            $selectedMode = "perf"
        } else {
            Write-Host "🎯 自动选择: 兼容模式" -ForegroundColor Green
            $selectedMode = "compat"
        }
    } else {
        $selectedMode = $Mode
        Write-Host "🎯 手动选择: $selectedMode 模式" -ForegroundColor Green
    }
    
    # 尝试构建
    $buildSuccess = Start-Build -BuildMode $selectedMode
    
    if ($buildSuccess) {
        $testSuccess = Test-Build
        
        if ($testSuccess) {
            Write-Host "`n🎉 构建和测试完成!" -ForegroundColor Green
            Write-Host "📁 可执行文件位置: target\release\examples\basic_mining_demo.exe" -ForegroundColor Cyan
            Write-Host "🚀 运行命令: .\target\release\examples\basic_mining_demo.exe" -ForegroundColor Cyan
        } else {
            Write-Host "`n⚠️  构建成功但测试失败，尝试兼容模式..." -ForegroundColor Yellow
            if ($selectedMode -ne "compat") {
                $buildSuccess = Start-Build -BuildMode "compat"
                if ($buildSuccess) {
                    Test-Build
                }
            }
        }
    } else {
        Write-Host "`n❌ 构建失败!" -ForegroundColor Red
        if ($selectedMode -ne "compat") {
            Write-Host "🔄 尝试兼容模式..." -ForegroundColor Yellow
            Start-Build -BuildMode "compat"
        }
    }
}
catch {
    Write-Host "❌ 脚本执行出错: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

Write-Host "`n📖 使用说明:" -ForegroundColor Cyan
Write-Host "  compat模式: 最大兼容性，适用于所有x86_64 CPU" -ForegroundColor Gray
Write-Host "  perf模式:   最高性能，需要现代CPU支持" -ForegroundColor Gray
Write-Host "  auto模式:   自动检测CPU并选择合适模式" -ForegroundColor Gray
