#!/bin/bash
set -e

PROJECT_ROOT=$(pwd)
WORKSPACE_DIR="${PROJECT_ROOT}/edk2"
EDK_DIR="$WORKSPACE_DIR/edk2"

mkdir -p "$WORKSPACE_DIR"

echo "[1/3] 准备 EDK2 构建环境..."
cd "$WORKSPACE_DIR"

if [ ! -d "ToolChain/RISCV" ]; then
    echo "下载 RISCV 工具链..."
    mkdir -p ToolChain/RISCV
    wget -q https://github.com/riscv-collab/riscv-gnu-toolchain/releases/download/2025.07.16/riscv64-elf-ubuntu-24.04-gcc-nightly-2025.07.16-nightly.tar.xz
    tar -xf riscv64-elf-ubuntu-24.04-gcc-nightly-2025.07.16-nightly.tar.xz -C "${WORKSPACE_DIR}/ToolChain/RISCV"
else
    echo "RISCV 工具链已存在，跳过下载。"
fi

if ! command -v python &> /dev/null; then
    echo "Python3 未安装，请先安装 Python3。"
    exit 1
fi

if ! command -v python3 &> /dev/null; then
    echo "Python3 未安装，请先安装 Python3。"
    exit 1
fi

echo "[2/3] 克隆 EDK2 仓库..."
if [ ! -d "$EDK_DIR" ]; then
    git clone --recurse-submodule https://github.com/tianocore/edk2.git "$EDK_DIR"
else
    echo "EDK2 仓库已存在，跳过克隆。"
fi

export PATH="$WORKSPACE_DIR/ToolChain/RISCV/riscv/bin:$PATH"

echo "[3/3] 构建 EDK2..."
cd "$WORKSPACE_DIR"

export WORKSPACE=`pwd`
export GCC5_RISCV64_PREFIX=riscv64-unknown-elf-
export PACKAGES_PATH=$WORKSPACE/edk2
export EDK_TOOLS_PATH=$WORKSPACE/edk2/BaseTools
source edk2/edksetup.sh --reconfig
make -C edk2/BaseTools
source edk2/edksetup.sh BaseTools

mkdir -p "$EDK_DIR/Hello"
cp -r "$PROJECT_ROOT/tests/edk2-hello" "$EDK_DIR"
mv "$EDK_DIR/edk2-hello"/* "$EDK_DIR/Hello/"
cp -r "$EDK_DIR/MdeModulePkg/MdeModulePkg.dsc" "$EDK_DIR/Hello/Hello.dsc"
echo -e "\n[Components]\n  Hello/Hello.inf" >> "$EDK_DIR/Hello/Hello.dsc"
build -a RISCV64 -t GCC5 -p "$EDK_DIR/Hello/Hello.dsc"

echo "EDK2 构建完成。生成的镜像位于：$WORKSPACE_DIR/Build/MdeModule/DEBUG_GCC5/RISCV64/Hello/Hello/DEBUG/Hello.efi"