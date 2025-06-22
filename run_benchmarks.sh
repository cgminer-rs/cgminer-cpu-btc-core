#!/bin/bash

# CPU BTC 核心完整基准测试运行脚本
# 专门用于运行 cgminer-cpu-btc-core 的全面性能基准测试

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# 打印带颜色的消息
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo -e "${PURPLE}[BENCHMARK]${NC} $1"
}

print_result() {
    echo -e "${CYAN}[RESULT]${NC} $1"
}

# 显示帮助信息
show_help() {
    echo "🚀 CPU BTC 核心完整基准测试运行脚本"
    echo ""
    echo "用法: $0 [选项]"
    echo ""
    echo "选项:"
    echo "  --all                    运行所有基准测试"
    echo "  --sha256                 只运行 SHA256 哈希测试"
    echo "  --device                 只运行设备创建和管理测试"
    echo "  --work                   只运行工作处理测试"
    echo "  --factory                只运行核心工厂测试"
    echo "  --performance            只运行性能监控测试"
    echo "  --temperature            只运行温度监控测试"
    echo "  --memory                 只运行内存效率测试"
    echo "  --concurrency            只运行并发性能测试"
    echo "  --quick                  快速测试模式"
    echo "  --detailed               详细测试模式"
    echo "  --save-baseline <name>   保存基准测试结果为基线"
    echo "  --compare <baseline>     与指定基线比较"
    echo "  --report                 生成详细报告"
    echo "  --open-report            运行后打开HTML报告"
    echo "  --help                   显示此帮助信息"
    echo ""
    echo "示例:"
    echo "  $0 --all                                    # 运行所有基准测试"
    echo "  $0 --sha256 --detailed                      # 详细运行 SHA256 测试"
    echo "  $0 --quick --report                         # 快速测试并生成报告"
    echo "  $0 --all --open-report                      # 运行所有测试并打开报告"
    echo "  $0 --save-baseline m4_baseline              # 保存基线"
    echo "  $0 --compare m4_baseline                    # 与基线比较"
}

# 获取系统信息
get_system_info() {
    print_header "系统信息收集"

    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        echo "🍎 检测到 macOS 系统"
        system_profiler SPHardwareDataType | grep -E "(Model Name|Chip|Total Number of Cores|Memory)"
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Linux
        echo "🐧 检测到 Linux 系统"
        echo "CPU: $(lscpu | grep 'Model name' | cut -d':' -f2 | xargs)"
        echo "Cores: $(nproc)"
        echo "Memory: $(free -h | grep '^Mem:' | awk '{print $2}')"
    else
        echo "❓ 未知系统类型: $OSTYPE"
    fi

    echo "🦀 Rust 版本: $(rustc --version)"
    echo "📦 Cargo 版本: $(cargo --version)"
    echo ""
}

# 运行所有基准测试
run_all_benchmarks() {
    print_header "运行完整基准测试套件"

    local args=""
    local mode="$1"
    local baseline_save="$2"
    local baseline_compare="$3"

    if [ "$mode" = "quick" ]; then
        args="-- --quick"
        print_info "使用快速模式运行基准测试"
    elif [ "$mode" = "detailed" ]; then
        args="-- --sample-size 1000"
        print_info "使用详细模式运行基准测试"
    fi

    if [ -n "$baseline_save" ]; then
        args="$args --save-baseline $baseline_save"
        print_info "将保存基线到: $baseline_save"
    fi

    if [ -n "$baseline_compare" ]; then
        args="$args --load-baseline $baseline_compare"
        print_info "将与基线比较: $baseline_compare"
    fi

    print_info "开始运行基准测试..."
    cargo bench --bench cpu_btc_core_benchmark $args
}

# 运行特定基准测试
run_specific_benchmark() {
    local benchmark_name="$1"
    local mode="$2"

    print_header "运行特定基准测试: ${benchmark_name}"

    local args=""
    if [ "$mode" = "quick" ]; then
        args="-- --quick"
    elif [ "$mode" = "detailed" ]; then
        args="-- --sample-size 1000"
    fi

    case "$benchmark_name" in
        "sha256")
            cargo bench --bench cpu_btc_core_benchmark bench_double_sha256 $args
            ;;
        "device")
            cargo bench --bench cpu_btc_core_benchmark bench_device_creation $args
            ;;
        "work")
            cargo bench --bench cpu_btc_core_benchmark bench_work_processing $args
            ;;
        "factory")
            cargo bench --bench cpu_btc_core_benchmark bench_core_factory $args
            ;;
        "performance")
            cargo bench --bench cpu_btc_core_benchmark bench_performance_monitoring $args
            ;;
        "temperature")
            cargo bench --bench cpu_btc_core_benchmark bench_temperature_monitoring $args
            ;;
        "memory")
            cargo bench --bench cpu_btc_core_benchmark bench_memory_efficiency $args
            ;;
        "concurrency")
            cargo bench --bench cpu_btc_core_benchmark bench_concurrency $args
            ;;
        *)
            print_error "未知的基准测试: $benchmark_name"
            return 1
            ;;
    esac
}

# 生成基准测试报告
generate_report() {
    print_header "生成基准测试报告"

    local report_dir="target/criterion"
    local report_file="benchmark_report_$(date +%Y%m%d_%H%M%S).md"

    if [ ! -d "$report_dir" ]; then
        print_error "未找到基准测试结果目录: $report_dir"
        return 1
    fi

    print_info "正在生成报告: $report_file"

    cat > "$report_file" << EOF
# CGMiner CPU BTC Core 基准测试报告

## 测试时间
$(date '+%Y年%m月%d日 %H:%M:%S')

## 系统信息
$(get_system_info)

## 基准测试结果

### 测试概览
- 测试框架: Criterion.rs
- 编译模式: Release (优化级别3)
- 测试类型: 完整基准测试套件

### 详细结果
请查看以下HTML报告获取详细的性能图表和统计数据：
- 主报告: target/criterion/report/index.html
- SHA256测试: target/criterion/sha256_double_hash/report/index.html
- 设备测试: target/criterion/device_creation/report/index.html
- 工作处理: target/criterion/work_processing/report/index.html
- 性能监控: target/criterion/performance_monitoring/report/index.html

### 建议
1. 查看HTML报告获取详细的性能分析
2. 关注性能回归和改进趋势
3. 根据结果优化关键路径

---
*报告由 run_benchmarks.sh 自动生成*
EOF

    print_success "报告已生成: $report_file"

    if [ -f "target/criterion/report/index.html" ]; then
        print_info "HTML报告位置: target/criterion/report/index.html"
    fi
}

# 打开HTML报告
open_report() {
    local report_path="target/criterion/report/index.html"

    if [ ! -f "$report_path" ]; then
        print_error "未找到HTML报告: $report_path"
        print_info "请先运行基准测试生成报告"
        return 1
    fi

    print_info "正在打开HTML报告..."

    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS
        open "$report_path"
    elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
        # Linux
        if command -v xdg-open > /dev/null; then
            xdg-open "$report_path"
        elif command -v firefox > /dev/null; then
            firefox "$report_path"
        elif command -v chromium-browser > /dev/null; then
            chromium-browser "$report_path"
        else
            print_warning "无法自动打开浏览器，请手动打开: $report_path"
        fi
    else
        print_warning "未知系统，请手动打开: $report_path"
    fi
}

# 主函数
main() {
    local command="$1"
    local param="$2"
    local param3="$3"
    local mode=""
    local generate_report_flag=false
    local open_report_flag=false

    # 解析参数
    while [[ $# -gt 0 ]]; do
        case $1 in
            --quick)
                mode="quick"
                shift
                ;;
            --detailed)
                mode="detailed"
                shift
                ;;
            --report)
                generate_report_flag=true
                shift
                ;;
            --open-report)
                open_report_flag=true
                shift
                ;;
            *)
                break
                ;;
        esac
    done

    case "$command" in
        "--all")
            get_system_info
            if [ "$param" = "--save-baseline" ]; then
                run_all_benchmarks "$mode" "$param3"
            elif [ "$param" = "--compare" ]; then
                run_all_benchmarks "$mode" "" "$param3"
            else
                run_all_benchmarks "$mode"
            fi
            ;;
        "--sha256")
            get_system_info
            run_specific_benchmark "sha256" "$mode"
            ;;
        "--device")
            get_system_info
            run_specific_benchmark "device" "$mode"
            ;;
        "--work")
            get_system_info
            run_specific_benchmark "work" "$mode"
            ;;
        "--factory")
            get_system_info
            run_specific_benchmark "factory" "$mode"
            ;;
        "--performance")
            get_system_info
            run_specific_benchmark "performance" "$mode"
            ;;
        "--temperature")
            get_system_info
            run_specific_benchmark "temperature" "$mode"
            ;;
        "--memory")
            get_system_info
            run_specific_benchmark "memory" "$mode"
            ;;
        "--concurrency")
            get_system_info
            run_specific_benchmark "concurrency" "$mode"
            ;;
        "--quick")
            get_system_info
            run_all_benchmarks "quick"
            ;;
        "--detailed")
            get_system_info
            run_all_benchmarks "detailed"
            ;;
        "--save-baseline")
            if [ -z "$param" ]; then
                print_error "请指定基线名称"
                exit 1
            fi
            get_system_info
            run_all_benchmarks "$mode" "$param"
            ;;
        "--compare")
            if [ -z "$param" ]; then
                print_error "请指定要比较的基线名称"
                exit 1
            fi
            get_system_info
            run_all_benchmarks "$mode" "" "$param"
            ;;
        "--report")
            generate_report
            ;;
        "--open-report")
            open_report
            ;;
        "--help"|"-h"|"")
            show_help
            ;;
        *)
            print_error "未知选项: $command"
            show_help
            exit 1
            ;;
    esac

    # 后处理
    if [ "$generate_report_flag" = true ]; then
        generate_report
    fi

    if [ "$open_report_flag" = true ]; then
        open_report
    fi
}

# 检查环境和依赖
check_environment() {
    print_header "环境检查"

    # 检查是否在正确的目录中
    if [ ! -f "Cargo.toml" ] || [ ! -d "benches" ]; then
        print_error "请在 cgminer-cpu-btc-core 目录中运行此脚本"
        print_info "当前目录: $(pwd)"
        exit 1
    fi

    # 检查 Rust 工具链
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo 未安装，请先安装 Rust 工具链"
        print_info "请访问 https://rustup.rs/ 安装 Rust"
        exit 1
    fi

    # 检查基准测试文件
    if [ ! -f "benches/cpu_btc_core_benchmark.rs" ]; then
        print_error "未找到基准测试文件: benches/cpu_btc_core_benchmark.rs"
        exit 1
    fi

    # 检查依赖
    print_info "检查项目依赖..."
    if ! cargo check --benches &> /dev/null; then
        print_warning "项目依赖检查失败，尝试更新依赖..."
        cargo update
    fi

    print_success "环境检查通过"
}

# 清理旧的基准测试结果
clean_old_results() {
    if [ -d "target/criterion" ]; then
        print_info "清理旧的基准测试结果..."
        rm -rf target/criterion
    fi
}

# 显示基准测试总结
show_summary() {
    print_header "基准测试总结"

    if [ -d "target/criterion" ]; then
        local test_count=$(find target/criterion -name "*.json" | wc -l)
        print_result "✅ 完成 $test_count 个基准测试"

        if [ -f "target/criterion/report/index.html" ]; then
            print_result "📊 HTML报告: target/criterion/report/index.html"
        fi

        # 显示主要测试结果目录
        print_info "主要测试结果:"
        for dir in target/criterion/*/; do
            if [ -d "$dir" ] && [ "$(basename "$dir")" != "report" ]; then
                echo "  - $(basename "$dir")"
            fi
        done
    else
        print_warning "未找到基准测试结果"
    fi
}

# 主程序入口
print_header "🚀 CGMiner CPU BTC Core 基准测试套件"
print_info "📍 当前目录: $(pwd)"
print_info "⏰ 开始时间: $(date '+%Y-%m-%d %H:%M:%S')"

# 检查环境
check_environment

# 如果没有参数，显示帮助
if [ $# -eq 0 ]; then
    show_help
    exit 0
fi

# 运行主函数
main "$@"

# 检查执行结果
if [ $? -eq 0 ]; then
    show_summary
    print_success "🎉 基准测试执行完成！"
    print_info "⏰ 结束时间: $(date '+%Y-%m-%d %H:%M:%S')"

    # 提示用户查看结果
    if [ -f "target/criterion/report/index.html" ]; then
        print_info "💡 提示: 运行 '$0 --open-report' 可直接打开HTML报告"
    fi
else
    print_error "❌ 基准测试执行失败！"
    print_info "💡 提示: 检查错误信息并确保所有依赖都已正确安装"
    exit 1
fi
