#!/bin/bash
set -e

LOG_FILE="qemu.log"
TARGET_STRING="EFI Output: Hello, World!"

if [ ! -f "$LOG_FILE" ]; then
    echo "❌ $LOG_FILE 不存在"
    exit 1
fi

if grep -qE "\[\s*[0-9]+\.[0-9]+\s+[0-9]+\s+arceboot::runtime::protocol::simple_text_output:[0-9]+\] $TARGET_STRING" "$LOG_FILE"; then
    echo "✅ 找到匹配日志行"
    exit 0
else
    echo "❌ 未找到匹配日志行"
    exit 2
fi
