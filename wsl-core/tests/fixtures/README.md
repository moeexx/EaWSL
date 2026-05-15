# wsl-core 测试 fixture

本目录存放 parser fixture，用于覆盖真实或代表性的 `wsl.exe` parser 输出差异。

## 目录约定

```text
tests/fixtures/parser/<locale>.json
```

- `locale`：BCP 47 风格代码，例如 `zh-CN`、`en-US`、`ja-JP`。
- 一个 locale 一个 JSON，文件名必须与 JSON 内的 `locale` 字段一致。
- 只为 locale 真实差异建 case；`zh-CN` 实测只有 `wsl --version` 的字段 label 需要 locale fixture。
- `list_verbose` / `list_online` 的表头和数据列实测为英文，不因 localized 提示语、本机 distro 列表或 online catalog 内容重复。
- 文件保存为 UTF-8。`wsl.exe` 实际输出可能是 UTF-16LE；fixture 保存为已解码文本，测试按 UTF-8 读取后传给 parser。

## JSON 结构

```json
{
  "locale": "zh-CN",
  "version": [
    {
      "name": "wsl_2_6_3",
      "source": "captured",
      "output_lines": ["WSL 版本: 2.6.3.0", "Windows: 10.0.26200.8039"],
      "expected": {
        "wsl": "2.6.3.0",
        "kernel": null,
        "wslg": null,
        "msrdc": null,
        "direct3d": null,
        "dxcore": null,
        "windows": "10.0.26200.8039"
      }
    }
  ]
}
```

- `name`：同一 command 下唯一、可读的 case 名称。
- `source`：`captured` 表示完整真实命令输出；`synthetic` 表示手写、改写、缩减、跨 locale 复用或为覆盖边界而构造。
- `output_lines`：原始输出按行存放；空行用 `""` 表示。
- `expected`：对应 parser 的结构化结果。
- command 数组可省略；例如 `en-US.json` 只验证英文 `wsl --version` label，不代表完整英文环境抓取。

## 添加新 locale 或版本

1. 抓取真实输出并确认已正确解码为文本，例如：
   ```pwsh
   wsl --version | Out-File -Encoding utf8 zh-CN.txt
   ```
   若文本包含 null byte 或乱码，先按 UTF-16LE 解码再转为 UTF-8。
2. 把输出内容转成对应 locale JSON 的一个 case。
3. 若是新语言，新增 `tests/fixtures/parser/<locale>.json`。只添加该 locale 相对现有 fixture 有真实差异的 command case。
4. 跑 `cargo test -p wsl-core`。

测试入口 [fixture_tests.rs](../../src/infrastructure/wsl_cli/parser/fixture_tests.rs) 会自动扫描全部 locale JSON；新增 fixture 不需要修改 Rust 测试代码。

## 目的

- 减少文件数量：一个语言一个 JSON，而不是一个 command 一个文件。
- 避免重复：相同表头、相同数据结构、相同 distro rows 不跨 locale 复制。
- 多版本回归：同一 locale JSON 内可并存多个 `name` 不同且确有输出差异的 case。
- 证据可追踪：用 `source` 区分真实抓取和合成边界样例。
- 不替代 inline 测试：inline `mod tests` 仍负责错误路径、边界条件和小粒度分支。
