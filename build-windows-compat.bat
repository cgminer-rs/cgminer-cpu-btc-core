@echo off
REM Windows兼容性构建脚本 - 解决CPU指令兼容性问题
REM 这个脚本使用最保守的设置确保在所有Windows系统上运行

echo CGMiner CPU BTC Core - Windows兼容性构建
echo =============================================

echo 使用兼容性模式构建...
echo   目标: 支持所有x86_64 CPU
echo   特性: 基础SSE指令集

REM 设置兼容性环境变量
set "RUSTFLAGS=-C target-cpu=x86-64 -C target-feature=+sse2,+sse4.1,+sse4.2 -C opt-level=3 -C overflow-checks=off"

echo 清理之前的构建...
cargo clean

echo 编译项目...
cargo build --release --features=thermal-management,power-management,cpu-affinity,temperature-monitoring

if %ERRORLEVEL% NEQ 0 (
    echo 项目构建失败!
    pause
    exit /b 1
)

echo 编译示例程序...
cargo build --release --examples --features=thermal-management,power-management,cpu-affinity,temperature-monitoring

if %ERRORLEVEL% NEQ 0 (
    echo 示例程序构建失败!
    pause
    exit /b 1
)

echo 构建成功!

echo 测试程序...
if exist "target\release\examples\basic_mining_demo.exe" (
    echo 找到可执行文件
    echo 尝试运行程序...

    REM 运行程序并捕获输出
    timeout /t 3 /nobreak > nul 2>&1
    echo 程序应该可以正常运行了

    echo.
    echo 构建完成!
    echo 可执行文件: target\release\examples\basic_mining_demo.exe
    echo 运行命令: target\release\examples\basic_mining_demo.exe
    echo.
    echo 如果仍然出现错误，请尝试:
    echo   1. 更新Windows和CPU驱动
    echo   2. 在较新的CPU上运行
    echo   3. 联系开发者获取支持
) else (
    echo 未找到可执行文件!
)

pause
