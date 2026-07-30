# dprint-plugin-incopat

格式化 [incoPat](https://www.incopat.com/) 专利数据库检索表达式的 [dprint](https://dprint.dev/) 插件。

## 格式化效果

输入：
```
TI=(空调 OR "air condition") AND TIAB=(蒸发器 OR evaporator) OR IPC=F25B
```

输出：
```
    ti=(空调 or "air condition")
and tiab=(蒸发器 or evaporator)
 or ipc=F25B
```

- 顶级布尔运算符强制换行，操作数左对齐，运算符右对齐
- 内层表达式在行宽允许时保持单行，超出时自适应换行并对齐
- 字段名和布尔运算符默认转为小写
- 语义检索关键词 (R/RAD/RPD) 始终输出为大写

## 安装

需要先安装 [dprint CLI](https://dprint.dev/install/)。

在 `dprint.json` 中添加插件：

```json
{
  "incopat": {},
  "includes": ["**/*.incopat"],
  "plugins": [
    "/path/to/dprint_plugin_incopat.wasm"
  ]
}
```

然后运行：

```bash
dprint fmt
```

## 支持的语法

- 布尔运算符: `AND`, `OR`, `NOT`
- 邻近算符: `(Nw)`, `(Nn)`, `(s)`, `(p)` — 词距/句内/段内限定
- (w)检索的每个单词均已指定顺序。如果前加数字，则代表在两个关键词之间插入0~n个单词（n代表0~99的数字），且检索词的顺序不可颠倒的记录。如下所示：

| 运算符 | 介入词的数量 |
| --- | --- |
| (w) 或者 (1w) | 0~1 |
| (0w) | 0 |
| (2w) | 0~2 |
| (99w) | 0~99 |

示例：car (w) engine表示car和engine之间隔0~1个单词；钻 (2w) 孔表示钻和孔之间隔0~2个字；

- (n)检索包含指定检索词且词序任意的记录。如果前加数字则代表两个关键词之间插入0~n个单词（n代表0~99的数字），且检索词顺序任意的记录。

| 运算符 | 介入词的数量 |
| --- | --- |
| (n) 或者 (1n) | 0~1 |
| (0n) | 0 |
| (2n) | 0~2 |
| (99n) | 0~99 |
- 频率算符: `(Nf)` — 最少出现次数限定
- 通配符: `*`, `?`, `$`
- 方括号范围: `[20200101 to 20231231]`
- 比较范围: `(20110101<=pd<=20130101)`, `(pd>20190101)`
- 公司树: `TREE@"公司名"`
- 字段表达式: `ti=keyword`, `ipc=(A61K or B01J)`
- 语义检索: `R=(专利号)`, `RAD=(文本)`, `RPD=(文本)`
- 引号短语: `"精确匹配"`, `'单引号'`
- 注释: `# 行注释`
- CJK 及多语言关键词

完整的字段列表和检索规则说明见 [docs/incopat-search-reference.md](docs/incopat-search-reference.md)。

## 配置项

所有配置项均为可选，以下为默认值：

```json
{
  "incopat": {
    "lineWidth": 120,
    "indentWidth": 2,
    "quoteStyle": "double",
    "fieldCase": "lowercase",
    "booleanOperatorCase": "lowercase"
  }
}
```

| 配置项 | 可选值 | 默认值 | 说明 |
|-------|--------|-------|------|
| `lineWidth` | 数字 | `120` | 行宽限制 |
| `indentWidth` | 数字 | `2` | 缩进宽度 |
| `quoteStyle` | `double` / `single` / `preserve` | `double` | 引号风格 |
| `fieldCase` | `lowercase` / `uppercase` / `preserve` | `lowercase` | 字段名大小写 |
| `booleanOperatorCase` | `lowercase` / `uppercase` / `preserve` | `lowercase` | 布尔运算符大小写 |

## 忽略格式化

在不想格式化的行前加注释：

```
# incopat-ignore
ti=( 保持   原样 )
```

## 与 Patsnap 插件的区别

| 特性 | dprint-plugin-patsnap | dprint-plugin-incopat |
|------|----------------------|----------------------|
| 字段分隔符 | `:` (冒号) | `=` (等号) |
| 文件扩展名 | `.patsnap` | `.incopat` |
| GAND 运算符 | 支持 | 不支持 |
| 邻近算符语法 | `$Wn`, `$PREn` | `(Nw)`, `(Nn)`, `(s)`, `(p)` |
| 频率算符语法 | `$FREQn` | `(Nf)` |
| 比较范围 | 不支持 | 支持 `(value<=field<=value)` |
| 语义检索 | 不支持 | 支持 R/RAD/RPD |
| 通配符 | `*`, `?`, `#` | `*`, `?`, `$` |

## 从源码构建

```bash
# 运行测试
cargo test

# 构建 WASM 插件
cargo build --release --target wasm32-unknown-unknown --features wasm
```

构建前需安装 WASM 目标：

```bash
rustup target add wasm32-unknown-unknown
```

构建产物位于 `target/wasm32-unknown-unknown/release/dprint_plugin_incopat.wasm`。

## License

[MIT](LICENSE)
