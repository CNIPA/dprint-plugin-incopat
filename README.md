# dprint-plugin-incopat

格式化 [incoPat](https://www.incopat.com/) 专利数据库检索表达式的 [dprint](https://dprint.dev/) 插件。

## 格式化效果

输入：
```
TI=(空调 OR "air condition") AND TIAB=(蒸发器 OR evaporator) OR IPC=F25B
```

输出：
```
(
        ti = (空调 or "air condition")
    and tiab = (蒸发器 or evaporator)
     or ipc = (F25B)
)
```

- 字段名和布尔运算符默认转为小写
- 字段值统一用括号包裹,语义检索关键词 (R/RAD/RPD) 始终输出为大写

## 输出规范

- **字段值始终用括号包裹**:`ti=压缩机` → `ti = (压缩机)`;多余括号层自动去除,如 `ti = ((压缩机))` → `ti = (压缩机)`
- **顶层含多个字段的检索式默认包一层括号**(单个字段不包),方便与其它检索式片断拼接而无歧义
- **多余的括号层自动折叠**:多字段内容只保留一层括号,单个字段全部移除;该规则适用于整个检索式及其中任意部分,目的是精简检索式
- **同句 (s) / 同段 (p) 运算符前后的值片断、频率运算符 (Nf) 前的值片断自动加括号**,方便并列近义词:`温差 (s) 蒸发器` → `(温差) (s) (蒸发器)`,`"机器人" (3f)` → `("机器人") (3f)`(`(w)`/`(n)` 等词距运算符不处理)
- 方括号范围(`pd=[20200101 to 20241231]`)和比较范围(`(20110101<=pd<=20130101)`)自带定界符,不额外包裹

## 换行与缩进

- **一个字段一行**:顶层二进制链的每个字段单独占一行,运算符 (`and`/`or`/`not`) 右对齐,字段列对齐
- **括号表示层级**:遇到括号时,其内部的字段缩进 8 个字符(每层括号递增),与下一行连接符后的字段对齐;闭合括号单独占一行
- 字段值内部仍保持自适应换行:行宽内保持单行,超出 `lineWidth` 才换行

例如：

```
(
        tiabc = (压缩机 or compressor)
    and des = (比热容)
)
and R = (一种空调水系统水容量自测工具)
```

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

## 语义检索字段的规则

incoPat 对语义检索字段 (`R` / `RAD` / `RPD`) 有严格的语法约束,格式化器会自动修正可无损修复的写法,其余违规会直接报错:

- 每个检索式中最多出现一次(R、RAD、RPD 之间也不能同时出现)
- 只能位于检索式的**最顶层**,且只能在**开头或结尾**(出现在中间位置会报错)
- 必须与检索式的其它部分用 `and` 连接(`or` 会自动改正为 `and`)
- 其它部分含**多个字段**时,自动用括号包裹;只有一个字段时不包裹
- 语义检索字段本身不能被括号包裹(多余的括号会被自动移除)
- 不能出现在 `not`、普通字段值、邻近/频率/公司树运算符中(会报错)

例如,下面的输入缺了包裹括号,会被自动修正:

```
输入:
tiabc=(压缩机 or compressor) or des=(比热容) and R=(一种空调水系统水容量自测工具)

输出:
(
        tiabc = (压缩机 or compressor)
     or des = (比热容)
)
and R = (一种空调水系统水容量自测工具)
```

而下面这些写法会直接报错并保持原样:

```
R = (a) and R = (b)          # 语义检索字段出现两次
 tiabc = (a) and R = (b) and des = (c)   # R 在中间位置
```

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
