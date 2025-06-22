#!/bin/bash

# CGMiner CPU BTC Core 示例运行脚本
# 用于快速运行各种示例程序

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
    echo -e "${PURPLE}[EXAMPLE]${NC} $1"
}

# 显示帮助信息
show_help() {
    echo "🚀 CGMiner CPU BTC Core 示例运行脚本"
    echo ""
    echo "用法: $0 [选项] [示例名称]"
    echo ""
    echo "示例列表:"
    echo "  basic              基本挖矿演示"
    echo "  multi              多设备挖矿演示"
    echo "  performance        性能监控演示"
    echo "  temperature        温度管理演示"
    echo "  affinity           CPU亲和性演示"
    echo "  simulation         真实挖矿模拟"
    echo "  benchmark          基准测试演示"
    echo ""
    echo "选项:"
    echo "  --all              运行所有示例"
    echo "  --release          使用发布模式运行"
    echo "  --features <list>  启用特定特性"
    echo "  --verbose          显示详细输出"
    echo "  --help             显示此帮助信息"
    echo ""
    echo "示例:"
    echo "  $0 basic                           # 运行基本挖矿演示"
    echo "  $0 --all                           # 运行所有示例"
    echo "  $0 --release performance           # 发布模式运行性能演示"
    echo "  $0 --features experimental basic   # 启用实验特性运行基本演示"
}

# 检查环境
check_environment() {
    print_header "环境检查"
    
    # 检查是否在正确的目录中
    if [ ! -f "Cargo.toml" ] || [ ! -d "examples" ]; then
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
    
    # 检查示例文件
    local examples_dir="examples"
    if [ ! -d "$examples_dir" ]; then
        print_error "未找到示例目录: $examples_dir"
        exit 1
    fi
    
    print_success "环境检查通过"
    print_info "Rust 版本: $(rustc --version)"
    print_info "Cargo 版本: $(cargo --version)"
}

# 运行单个示例
run_example() {
    local example_name="$1"
    local release_mode="$2"
    local features="$3"
    local verbose="$4"
    
    local example_file=""
    local description=""
    
    # 映射示例名称到文件名
    case "$example_name" in
        "basic")
            example_file="basic_mining_demo"
            description="基本挖矿演示"
            ;;
        "multi")
            example_file="multi_device_demo"
            description="多设备挖矿演示"
            ;;
        "performance")
            example_file="performance_monitoring_demo"
            description="性能监控演示"
            ;;
        "temperature")
            example_file="temperature_demo"
            description="温度管理演示"
            ;;
        "affinity")
            example_file="cpu_affinity_demo"
            description="CPU亲和性演示"
            ;;
        "simulation")
            example_file="real_mining_simulation"
            description="真实挖矿模拟"
            ;;
        "benchmark")
            example_file="benchmark_demo"
            description="基准测试演示"
            ;;
        *)
            print_error "未知的示例: $example_name"
            print_info "使用 --help 查看可用示例"
            return 1
            ;;
    esac
    
    # 检查示例文件是否存在
    if [ ! -f "examples/${example_file}.rs" ]; then
        print_error "示例文件不存在: examples/${example_file}.rs"
        return 1
    fi
    
    print_header "运行示例: $description"
    print_info "文件: examples/${example_file}.rs"
    
    # 构建命令
    local cmd="cargo run"
    
    if [ "$release_mode" = "true" ]; then
        cmd="$cmd --release"
        print_info "使用发布模式"
    fi
    
    if [ -n "$features" ]; then
        cmd="$cmd --features \"$features\""
        print_info "启用特性: $features"
    fi
    
    cmd="$cmd --example $example_file"
    
    if [ "$verbose" = "true" ]; then
        print_info "执行命令: $cmd"
    fi
    
    # 运行示例
    local start_time=$(date +%s)
    
    if [ "$verbose" = "true" ]; then
        eval $cmd
    else
        eval $cmd 2>/dev/null
    fi
    
    local exit_code=$?
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    if [ $exit_code -eq 0 ]; then
        print_success "示例运行完成 (耗时: ${duration}秒)"
    else
        print_error "示例运行失败 (退出码: $exit_code)"
        return $exit_code
    fi
}

# 运行所有示例
run_all_examples() {
    local release_mode="$1"
    local features="$2"
    local verbose="$3"
    
    print_header "运行所有示例"
    
    local examples=("basic" "multi" "performance" "temperature" "affinity" "simulation")
    local total=${#examples[@]}
    local success_count=0
    local failed_examples=()
    
    for i in "${!examples[@]}"; do
        local example="${examples[$i]}"
        local current=$((i + 1))
        
        echo ""
        print_info "进度: $current/$total - 运行示例: $example"
        echo "========================================"
        
        if run_example "$example" "$release_mode" "$features" "$verbose"; then
            success_count=$((success_count + 1))
        else
            failed_examples+=("$example")
        fi
        
        echo "========================================"
    done
    
    echo ""
    print_header "所有示例运行完成"
    print_info "总计: $total 个示例"
    print_success "成功: $success_count 个示例"
    
    if [ ${#failed_examples[@]} -gt 0 ]; then
        print_error "失败: ${#failed_examples[@]} 个示例"
        print_info "失败的示例: ${failed_examples[*]}"
        return 1
    else
        print_success "🎉 所有示例都运行成功!"
    fi
}

# 主函数
main() {
    local example_name=""
    local run_all=false
    local release_mode=false
    local features=""
    local verbose=false
    
    # 解析参数
    while [[ $# -gt 0 ]]; do
        case $1 in
            --all)
                run_all=true
                shift
                ;;
            --release)
                release_mode=true
                shift
                ;;
            --features)
                features="$2"
                shift 2
                ;;
            --verbose)
                verbose=true
                shift
                ;;
            --help|-h)
                show_help
                exit 0
                ;;
            -*)
                print_error "未知选项: $1"
                show_help
                exit 1
                ;;
            *)
                if [ -z "$example_name" ]; then
                    example_name="$1"
                else
                    print_error "只能指定一个示例名称"
                    exit 1
                fi
                shift
                ;;
        esac
    done
    
    # 检查环境
    check_environment
    
    echo ""
    
    # 运行示例
    if [ "$run_all" = true ]; then
        run_all_examples "$release_mode" "$features" "$verbose"
    elif [ -n "$example_name" ]; then
        run_example "$example_name" "$release_mode" "$features" "$verbose"
    else
        print_error "请指定要运行的示例或使用 --all 运行所有示例"
        echo ""
        show_help
        exit 1
    fi
}

# 脚本入口
print_header "🚀 CGMiner CPU BTC Core 示例运行器"
print_info "📍 当前目录: $(pwd)"
print_info "⏰ 开始时间: $(date '+%Y-%m-%d %H:%M:%S')"

# 如果没有参数，显示帮助
if [ $# -eq 0 ]; then
    echo ""
    show_help
    exit 0
fi

# 运行主函数
main "$@"

# 检查执行结果
if [ $? -eq 0 ]; then
    echo ""
    print_success "🎉 脚本执行完成！"
    print_info "⏰ 结束时间: $(date '+%Y-%m-%d %H:%M:%S')"
else
    echo ""
    print_error "❌ 脚本执行失败！"
    print_info "💡 提示: 使用 --verbose 选项查看详细错误信息"
    exit 1
fi
