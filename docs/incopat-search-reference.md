# incoPat 专利数据库检索字段与规则参考

> 本文档整理自 incoPat 官方帮助文档，作为离线参考使用。
> 来源：incoPat 帮助中心 - 字段代码表 (`helpcode.html`) 和检索规则 (`principal.html`)

---

## 一、检索规则

### 1. 布尔运算符

| 运算符 | 说明 | 示例 |
|--------|------|------|
| `AND` | 逻辑与，两侧条件必须同时满足 | `ti=空调 and ab=蒸发器` |
| `OR` | 逻辑或，满足任一条件即可 | `ti=空调 or ti=冰箱` |
| `NOT` | 逻辑非，排除右侧条件 | `ti=空调 not ab=冰箱` |

**优先级**：`NOT` > `AND` > `OR`

**隐式 AND**：两个相邻的表达式之间如果没有运算符，默认为 AND 连接。

### 2. 通配符

| 符号 | 说明 | 示例 |
|------|------|------|
| `*` | 匹配零个或多个字符 | `comput*` 匹配 computer, computing 等 |
| `?` | 匹配恰好一个字符 | `m?tor` 匹配 motor, meter 等 |
| `$` | 匹配零个或多个字符（同 `*`） | `comput$` |

### 3. 邻近算符 (Proximity Operators)

邻近算符用于指定两个检索词在文档中出现的距离关系，格式为 `(Nx)` 或 `(x)`。

| 算符 | 格式 | 说明 |
|------|------|------|
| `(Nw)` | `词1 (Nw) 词2` | 两词之间最多间隔 N 个词（N为0-99），**有序**（词1 必须在词2 前面） |
| `(w)` | `词1 (w) 词2` | 等同于 `(1w)` |
| `(Nn)` | `词1 (Nn) 词2` | 两词之间最多间隔 N 个词（N为0-99），**无序**（不限前后顺序） |
| `(n)` | `词1 (n) 词2` | 等同于 `(1n)` |
| `(s)` | `词1 (s) 词2` | 两词出现在同一句子中 |
| `(p)` | `词1 (p) 词2` | 两词出现在同一段落中 |

**示例**：
```
空调 (0w) 蒸发器    -- "空调"紧邻"蒸发器"
空调 (2w) 蒸发器    -- "空调"在"蒸发器"前面，间隔不超过2个词
data (3n) line      -- "data"和"line"间隔不超过3个词，不限顺序
空调 (s) 蒸发器     -- 两词出现在同一句子中
```

### 4. 频率算符 (Frequency Operator)

频率算符限定某个词在文档中出现的最少次数。

| 算符 | 格式 | 说明 |
|------|------|------|
| `(Nf)` | `词 (Nf)` | 该词至少出现 N 次 |

**示例**：
```
tiab=("机器人" (3f))   -- "机器人"在标题或摘要中至少出现3次
```

### 5. 字段表达式

使用 `字段名=检索值` 的格式指定在特定字段中检索。

```
ti=空调                  -- 在标题中检索"空调"
tiab=(空调 or 蒸发器)   -- 在标题或摘要中检索
ipc=F25B                -- 检索IPC分类号
```

- 字段名不区分大小写：`TI=`, `ti=`, `Ti=` 等效
- 括号内可使用布尔运算符组合多个条件

### 6. 范围表达式

#### 方括号范围

```
pd=[20200101 to 20241231]   -- 公开日在2020-01-01到2024-12-31之间
```

#### 比较范围

```
(20110101<=pd<=20130101)    -- 双侧比较：公开日在指定范围内
(pd>20190101)               -- 单侧比较：公开日晚于2019-01-01
(20190101<=pd)              -- 单侧比较：同上
```

支持的比较运算符：`<`, `<=`, `>`, `>=`

### 7. TREE@ 运算符

用于公司名检索时展开公司树（包含子公司、关联公司等）。

```
ap=(TREE@"清华大学")    -- 检索"清华大学"及其关联机构的申请
```

### 8. 语义检索 (R / RAD / RPD)

语义检索关键词用于基于语义相似度进行检索。

| 关键词 | 说明 |
|--------|------|
| `R` | 通用语义检索，通过专利号或文本进行相似检索 |
| `RAD` | 基于申请日（Application Date）排序的语义检索 |
| `RPD` | 基于公开日（Publication Date）排序的语义检索 |

**使用规则**：
- 每个检索式中只能使用一次语义检索关键词
- 必须出现在表达式的开头或结尾
- 不能放在括号内
- 必须使用显式 AND 与其他条件连接

**示例**：
```
R=(CN101850473B)                     -- 通过专利号进行语义检索
RAD=(CN101850473B) AND tiab=(发动机)  -- 语义检索 + 字段检索
tiab=(空调) AND RPD=(蒸发器)          -- 字段检索 + 语义检索
```

### 9. 引号短语

使用双引号或单引号将多词短语括起来作为精确匹配：

```
ti="air condition"    -- 精确匹配短语"air condition"
ti='空气调节器'        -- 单引号同样有效
```

### 10. 注释

以 `#` 开头的行为注释，不参与检索：

```
# 这是一条注释
ti=空调
```

---

## 二、字段代码完整列表

### 技术字段 (Technical Fields)

| 字段代码 | 说明 |
|----------|------|
| `TI` | 标题 |
| `TI-CN` | 中文标题 |
| `TI-OTLANG` | 原始语言标题 |
| `TI-EN` | 英文标题 |
| `TI-DWPI` | DWPI 标题 |
| `AB` | 摘要 |
| `AB-CN` | 中文摘要 |
| `AB-OTLANG` | 原始语言摘要 |
| `AB-EN` | 英文摘要 |
| `USE-DWPI` | DWPI 用途 |
| `ADV-DWPI` | DWPI 优势 |
| `NOVELTY-DWPI` | DWPI 新颖性 |
| `ABSTRACT-DWPI` | DWPI 摘要 |
| `DTD-DWPI` | DWPI 详细描述 |
| `ACTIVITY-DWPI` | DWPI 活动 |
| `MEC-DWPI` | DWPI 机理 |
| `FOC-DWPI` | DWPI 焦点 |
| `DRAW-DWPI` | DWPI 附图 |
| `TIAB` | 标题+摘要 |
| `TIAB-DWPI` | DWPI 标题+摘要 |
| `CLAIM` | 权利要求 |
| `FIRST-CLAIM` | 第一权利要求 |
| `FIRST-CLAIM-OR` | 第一权利要求（原文） |
| `INDEPCLAIMS-CN` | 中文独立权利要求 |
| `DEPCLAIMS-CN` | 中文从属权利要求 |
| `NO-INDEPCLAIMS` | 独立权利要求数量 |
| `NO-DEPCLAIMS` | 从属权利要求数量 |
| `FIRST-CLAIM-TS` | 第一权利要求翻译状态 |
| `LEN-FIRST-CLAIM` | 第一权利要求长度 |
| `CLAIM-EN` | 英文权利要求 |
| `CLAIM-CN` | 中文权利要求 |
| `CLAIM-OT` | 原文权利要求 |
| `NO-CLAIM` | 权利要求数量 |
| `TIABC` | 标题+摘要+权利要求 |
| `DES` | 说明书 |
| `DES-OT` | 原文说明书 |
| `DES-EN` | 英文说明书 |
| `DES-CN` | 中文说明书 |
| `TECHNICAL-FIELD` | 技术领域 |
| `BACKGROUND-ART` | 背景技术 |
| `DISCLOSURE` | 发明内容 |
| `MODE-FOR-INVENTION` | 实施方式 |
| `NO-IMAGE` | 附图数量 |
| `EFFECT-S-CN` | 技术效果（句子） |
| `USE-CN` | 用途（中文） |
| `USE-EN` | 用途（英文） |
| `EFFECT-PH-CN` | 技术效果（短语） |
| `EFFECT-CN` | 技术效果 |
| `EFFECT-CN-3` | 技术效果 L3 |
| `EFFECT-CN-2` | 技术效果 L2 |
| `EFFECT-CN-1` | 技术效果 L1 |
| `EFFECT-TRIZ` | TRIZ 效果 |
| `ALL` | 全部文本字段 |
| `FULL` | 全文 |
| `FILING-LANG` | 申请语言 |
| `PRD-FLAG` | PRD 标志 |
| `PRD` | 优先权文献 |
| `PRD-DWPI` | DWPI 优先权文献 |
| `PAGE` | 页数 |
| `VLSTAR` | 价值度 |
| `VLSTAR-1` | 价值度 L1 |
| `VLSTAR-2` | 价值度 L2 |
| `VLSTAR-3` | 价值度 L3 |
| `REWARD-LEVEL` | 奖励等级 |
| `REWARD-NAME` | 奖励名称 |
| `REWARD-SESSION` | 奖励届次 |
| `STD-TYPE` | 标准类型 |
| `STD-PROJECT` | 标准项目 |
| `STD-NUM` | 标准号 |
| `STD-COMPANY` | 标准制定公司 |
| `STD-FLAG` | 标准标志 |
| `CAS-NO` | CAS 号 |
| `DRUG-NAME-CN` | 药物名称（中文） |
| `DRUG-NAME-EN` | 药物名称（英文） |
| `COMPANY` | 公司 |
| `BRAND-NAME` | 品牌名 |
| `ACTIVE-INGREDIENT` | 活性成分 |
| `TARGET` | 靶点 |
| `INDICATION` | 适应症 |
| `PATENT-EXPIRATION` | 专利到期日 |
| `PED-PATENT-EXPIRATION` | PED 专利到期日 |

### 公司&人物字段 (Company & People Fields)

| 字段代码 | 说明 |
|----------|------|
| `WHO` | 所有人/公司/发明人 |
| `AP-ALL` | 所有申请人 |
| `AP-GROUP` | 申请人集团 |
| `AP-GROUPTT` | 申请人集团（翻译） |
| `AP` | 申请人 |
| `APTT` | 申请人（翻译） |
| `AP-LITE` | 申请人（简称） |
| `AP-OR` | 申请人（原文） |
| `AP-OT` | 申请人（其他语言） |
| `AP-TS` | 申请人翻译状态 |
| `APNOR` | 申请人（标准化） |
| `APNORTT` | 申请人（标准化翻译） |
| `AP-ROOT` | 申请人（根名称） |
| `AP-FIRST` | 第一申请人 |
| `AP-NEW-NAME` | 申请人新名称 |
| `AP-OTADD` | 申请人其他地址 |
| `NO-AP` | 申请人数量 |
| `PATENTEE` | 专利权人 |
| `PATENTEETT` | 专利权人（翻译） |
| `PATENTEENOR` | 专利权人（标准化） |
| `PATENTEENORTT` | 专利权人（标准化翻译） |
| `ASSIGN-PARTY` | 转让方 |
| `AOR` | 转让人 |
| `AOR-TYPE` | 转让人类型 |
| `AEE` | 受让人 |
| `AEETT` | 受让人（翻译） |
| `AEENOR` | 受让人（标准化） |
| `AEENORTT` | 受让人（标准化翻译） |
| `AEE-TYPE` | 受让人类型 |
| `IN` | 发明人 |
| `INTT` | 发明人（翻译） |
| `IN-OR` | 发明人（原文） |
| `IN-OT` | 发明人（其他语言） |
| `IN-TS` | 发明人翻译状态 |
| `IN-FIRST` | 第一发明人 |
| `NO-IN` | 发明人数量 |
| `IN-NEW-NAME` | 发明人新名称 |
| `IN-CURRENT` | 当前发明人 |
| `LOR` | 许可方 |
| `LEE` | 被许可方 |
| `LOR-TYPE` | 许可方类型 |
| `LEE-TYPE` | 被许可方类型 |
| `OPPONENT` | 异议人 |
| `AT` | 代理人 |
| `AGC` | 代理机构 |
| `LGI-PARTY` | 诉讼当事人 |
| `RE-AP` | 复审申请人 |
| `IN-AP` | 无效申请人 |
| `RI-ME` | 审查员 |
| `RI-AE` | 复审委员 |
| `RI-LEADER` | 审查组长 |
| `POR` | 质押出质人 |
| `PEE` | 质押质权人 |
| `EX` | 审查员 |
| `AP-TYPE` | 申请人类型 |
| `PATENTEE-TYPE` | 专利权人类型 |
| `CO-DWPI` | DWPI 公司代码 |
| `CK-DWPI` | DWPI 公司关键词 |
| `CK-TYPE-DWPI` | DWPI 公司关键词类型 |
| `AP-AS` | 申请人简称 |
| `AP-EN` | 申请人（英文） |
| `AP-REG-LOCATION` | 申请人注册地 |
| `AP-COMPANY-ORG-TYPE` | 申请人组织类型 |
| `AP-ESTIBLISH-TIME` | 申请人成立时间 |
| `AP-USC` | 申请人统一社会信用代码 |
| `AP-REG-NUMBER` | 申请人注册号 |
| `AP-REG-STATUS` | 申请人注册状态 |
| `AP-LIST-CODE` | 申请人上市代码 |

### 分类字段 (Classification Fields)

| 字段代码 | 说明 |
|----------|------|
| `IPC` | IPC 分类号 |
| `IPC-MAIN` | IPC 主分类号 |
| `IPC-SECTION` | IPC 部 |
| `IPC-CLASS` | IPC 大类 |
| `IPC-SUBCLASS` | IPC 小类 |
| `IPC-GROUP` | IPC 大组 |
| `IPC-SUBGROUP` | IPC 小组 |
| `IPCM-SECTION` | 主分类 IPC 部 |
| `IPCM-CLASS` | 主分类 IPC 大类 |
| `IPCM-SUBCLASS` | 主分类 IPC 小类 |
| `IPCM-GROUP` | 主分类 IPC 大组 |
| `IPC-LOW` | IPC 低分类 |
| `IPC-HIGH` | IPC 高分类 |
| `IPCM-LOW` | 主分类 IPC 低分类 |
| `IPCM-HIGH` | 主分类 IPC 高分类 |
| `IPC-DWPI` | DWPI IPC |
| `IPC-SECTION-DWPI` | DWPI IPC 部 |
| `IPC-CLASS-DWPI` | DWPI IPC 大类 |
| `IPC-SUBCLASS-DWPI` | DWPI IPC 小类 |
| `IPC-GROUP-DWPI` | DWPI IPC 大组 |
| `IPC-SUBGROUP-DWPI` | DWPI IPC 小组 |
| `IPC-F-DWPI` | DWPI IPC (F) |
| `IPC-SECTION-F-DWPI` | DWPI IPC 部 (F) |
| `IPC-CLASS-F-DWPI` | DWPI IPC 大类 (F) |
| `IPC-SUBCLASS-F-DWPI` | DWPI IPC 小类 (F) |
| `IPC-GROUP-F-DWPI` | DWPI IPC 大组 (F) |
| `IPC-SUBGROUP-F-DWPI` | DWPI IPC 小组 (F) |
| `DC-DWPI` | DWPI DC 分类 |
| `DC-SECTION-DWPI` | DWPI DC 部 |
| `DC-CLASS-DWPI` | DWPI DC 大类 |
| `MC-DWPI` | DWPI MC 分类 |
| `MC-SECTION-DWPI` | DWPI MC 部 |
| `MC-CLASS-DWPI` | DWPI MC 大类 |
| `MC-GROUP-DWPI` | DWPI MC 大组 |
| `MC-SUBGROUP-DWPI` | DWPI MC 小组 |
| `MC-SUBGROUPD-DWPI` | DWPI MC 小组 (D) |
| `MC-FULLMC-DWPI` | DWPI 完整 MC |
| `MC-FULLMCX-DWPI` | DWPI 完整 MC (X) |
| `LOC` | 洛迦诺分类 |
| `LOC-CLASS` | 洛迦诺大类 |
| `LOC-SUBCLASS` | 洛迦诺小类 |
| `ECLA` | ECLA 分类 |
| `ECLA-SECTION` | ECLA 部 |
| `ECLA-CLASS` | ECLA 大类 |
| `ECLA-SUBCLASS` | ECLA 小类 |
| `ECLA-GROUP` | ECLA 大组 |
| `ECLA-SUBGROUP` | ECLA 小组 |
| `UC` | US 分类 |
| `UC-MAIN` | US 主分类 |
| `CPC` | CPC 分类号 |
| `CPC-SECTION` | CPC 部 |
| `CPC-CLASS` | CPC 大类 |
| `CPC-SUBCLASS` | CPC 小类 |
| `CPC-GROUP` | CPC 大组 |
| `CPC-SUBGROUP` | CPC 小组 |
| `CPC-MAIN` | CPC 主分类 |
| `CPCM-SECTION` | 主分类 CPC 部 |
| `CPCM-CLASS` | 主分类 CPC 大类 |
| `CPCM-SUBCLASS` | 主分类 CPC 小类 |
| `CPCM-GROUP` | 主分类 CPC 大组 |
| `CPCM-SUBGROUP` | 主分类 CPC 小组 |
| `FI` | 日本 FI 分类 |
| `FT` | 日本 F-Term |
| `CLASS` | 所有分类号 |
| `BCLASS` | 产业分类 |
| `MBCLAS1` ~ `MBCLAS4` | 主产业分类 L1-L4 |
| `MBCLASS` | 主产业分类 |
| `BCLAS1` ~ `BCLAS4` | 产业分类 L1-L4 |
| `INDUSTRY1` | 产业 L1 |
| `MINDUSTRY1` | 主产业 L1 |
| `MINDUSTRY2` | 主产业 L2 |
| `INDUSTRY2` | 产业 L2 |
| `INDUSTRY-TYPE` | 产业类型 |
| `MKCLAS1` ~ `MKCLAS2` | 市场分类 L1-L2 |
| `SC-MAIN` | SC 主分类 |
| `SC-SECTION` | SC 部 |
| `SC-CLASS` | SC 大类 |
| `SC-SUBCLASS` | SC 小类 |
| `LNGCLAS1` ~ `LNGCLAS3` | 语种分类 L1-L3 |
| `CPCLAS1` ~ `CPCLAS3` | 计算分类 L1-L3 |
| `DIGCLAS1` ~ `DIGCLAS3` | 数字分类 L1-L3 |

### 地域字段 (Region Fields)

| 字段代码 | 说明 |
|----------|------|
| `AP-COUNTRY` | 申请人国家 |
| `IN-COUNTRY` | 发明人国家 |
| `AUTH` | 公开机构/专利局 |
| `PNC` | 公开国家代码 |
| `AP-ADD` | 申请人地址 |
| `PR-AU` | 优先权国家 |
| `PR-AU-DWPI` | DWPI 优先权国家 |
| `ORIPRC-DWPI` | DWPI 原始优先权国家 |
| `AP-PROVINCE` | 申请人省份 |
| `PC-CN` | 中国省份代码 |
| `AP-PC` | 申请人邮编 |
| `CITY` | 城市 |
| `COUNTY` | 区县 |
| `PATENTEE-ADD` | 专利权人地址 |
| `PATENTEE-PROVINCE` | 专利权人省份 |
| `PATENTEE-CITY` | 专利权人城市 |
| `PATENTEE-COUNTY` | 专利权人区县 |
| `IN-ADD` | 发明人地址 |
| `IN-ADD-OTH` | 发明人其他地址 |
| `IN-OR-ADD` | 发明人原文地址 |
| `IN-CITY` | 发明人城市 |
| `IN-STATE` | 发明人州/省 |
| `ASSIGN-COUNTRY` | 转让国家 |
| `ASSIGNEE-ADD` | 受让人地址 |
| `ASSIGNEE-CADD` | 受让人公司地址 |
| `ASSIGN-STATE` | 转让州/省 |
| `ASSIGN-CITY` | 转让城市 |
| `AEE-PROVINCE` | 受让人省份 |
| `AEE-CITY` | 受让人城市 |
| `AEE-COUNTY` | 受让人区县 |
| `ASSIGNOR-ADD` | 转让人地址 |
| `AOR-PROVINCE` | 转让人省份 |
| `AOR-CITY` | 转让人城市 |
| `AOR-COUNTY` | 转让人区县 |
| `AT-COUNTRY` | 代理人国家 |
| `AT-ADD` | 代理人地址 |
| `AT-CITY` | 代理人城市 |
| `AT-STATE` | 代理人州/省 |
| `LGI-REGION` | 诉讼地域 |
| `WHERE` | 所有地域 |
| `DE-COUNTRY` | 指定国家 |

### 号码字段 (Number Fields)

| 字段代码 | 说明 |
|----------|------|
| `AN` | 申请号 |
| `ANN` | 申请号（标准化） |
| `PN` | 公开号/公告号 |
| `PNN` | 公开号（标准化） |
| `PU-PN` | 公布号 |
| `GRANT-PN` | 授权号 |
| `RPND-DWPI` | DWPI 公开号 |
| `PR` | 优先权号 |
| `PR-DWPI` | DWPI 优先权号 |
| `PRN` | 优先权号（标准化） |
| `PT` | 专利类型 |
| `PAT` | 专利类型 |
| `PNK` | 文献类型代码 |
| `MF` | 同族号 |
| `CF` | 同族号 |
| `MFN` | 同族号（标准化） |
| `CFN` | 同族号（标准化） |
| `IF` | INPADOC 族 |
| `IFN` | INPADOC 族（标准化） |
| `F-DWPI` | DWPI 族号 |
| `FN-DWPI` | DWPI 族号 |
| `FA-COUNTRY` | 族首申请国 |
| `FA-COUNTRY-DWPI` | DWPI 族首申请国 |
| `FCN-DWPI` | DWPI 族首国家 |
| `NUMBER` | 所有号码 |
| `RI-NUM` | 复审/无效号 |
| `RI-INERNAL` | 内部编号 |
| `LICENSE-NO` | 许可编号 |
| `PLEDGE-NO` | 质押编号 |
| `IAN` | 国际申请号 |
| `IPN` | 国际公开号 |
| `SAN` | 分案申请号 |
| `SUBSAN` | 子分案号 |
| `ESM` | 同日申请标记 |
| `CONTINUATION-PARENT` | 延续案母案 |
| `CONTINUATION-INPART-PARENT` | 部分延续案母案 |

### 日期字段 (Date Fields)

| 字段代码 | 说明 |
|----------|------|
| `AD` | 申请日 |
| `RADD-DWPI` | DWPI 申请日 |
| `ADM` | 申请月 |
| `ADY` | 申请年 |
| `PD` | 公开日/公告日 |
| `PU-DATE` | 公布日 |
| `PU-YEAR` | 公布年 |
| `PU-MONTH` | 公布月 |
| `PDY` | 公开年 |
| `PDM` | 公开月 |
| `PR-DATE` | 优先权日 |
| `PR-DATE-DWPI` | DWPI 优先权日 |
| `PRYEAR` | 优先权年 |
| `ORI-PRDATE` | 原始优先权日 |
| `ORI-PRYEAR` | 原始优先权年 |
| `ORI-PRYEAR-DWPI` | DWPI 原始优先权年 |
| `CT-AD` | 引证申请日 |
| `CT-PD` | 引证公开日 |
| `CTFW-AD` | 被引证申请日 |
| `CTFW-PD` | 被引证公开日 |
| `CTYEAR` | 引证年 |
| `SUBEX-DATE` | 实审日 |
| `GRANT-DATE` | 授权日 |
| `GRANT-YEAR` | 授权年 |
| `GRANT-MONTH` | 授权月 |
| `EXDT` | 期限届满日 |
| `EXDT-YEAR` | 期限届满年 |
| `EXDT-MONTH` | 期限届满月 |
| `EXPIRY-DATE` | 专利到期日 |
| `EXPIRY-YEAR` | 专利到期年 |
| `ECD` | 最早公开日 |
| `PLEDGEYEAR` | 质押年 |
| `ASSIGNYEAR` | 转让年 |
| `LICENSEYEAR` | 许可年 |
| `ASSIGN-DATE` | 转让日 |
| `ASSIGN-RD` | 转让登记日 |
| `RI-DATE` | 复审/无效日期 |
| `LGI-DATE` | 诉讼日期 |
| `LGI-FD` | 诉讼立案日 |
| `LGI-CD` | 诉讼结案日 |
| `LGD` | 诉讼日 |
| `PLEDGE-DATE` | 质押日期 |
| `LICENSE-DATE` | 许可日期 |
| `LICENSE-SD` | 许可起始日 |
| `LICENSE-TD` | 许可终止日 |
| `PLEDGE-CD` | 质押登记日 |
| `PLEDGE-RD` | 质押解除日 |
| `LGIYEAR` | 诉讼年 |
| `LGI-FY` | 诉讼立案年 |
| `LGI-CY` | 诉讼结案年 |
| `PATENT-LIFE` | 专利寿命 |
| `EX-TIME` | 审查周期 |
| `PFEX-TIME` | 优先权至授权周期 |
| `RE-DATE` | 复审日期 |
| `IN-DATE` | 无效日期 |
| `OR-DATE` | 口审日期 |
| `REAPP-DATE` | 复审申请日 |
| `INAPP-DATE` | 无效申请日 |

### 法律字段 (Legal Fields)

| 字段代码 | 说明 |
|----------|------|
| `STATUS` | 法律状态 |
| `STATUS-LITE` | 法律状态（简要） |
| `LG` | 法律事件 |
| `LGE` | 法律事件（英文） |
| `LGF` | 法律事件（法文） |
| `LGC` | 法律事件（中文） |
| `RI-TYPE` | 复审/无效类型 |
| `RI-TEXT` | 复审/无效文本 |
| `RI-AP` | 复审/无效申请人 |
| `RE-DECISION` | 复审决定 |
| `RI-BASIS` | 复审/无效依据 |
| `RI-POINT` | 复审/无效要点 |
| `LGI-COURT` | 诉讼法院 |
| `LGI-JUDGE` | 审判长 |
| `LGI-FIRM` | 律师事务所 |
| `LAWYER` | 律师 |
| `LGI-CAUSE` | 案由 |
| `ASSIGN-TEXT` | 转让文本 |
| `LGI-TI` | 诉讼标题 |
| `LGI-TEXT` | 诉讼文本 |
| `LGI-TYPE` | 诉讼类型 |
| `LGI-NO` | 诉讼案号 |
| `LGI-PROCEDURE` | 诉讼程序 |
| `LGI-PLAINTIFF` | 原告 |
| `LGI-DEFENDANT` | 被告 |
| `LICENSE-TYPE` | 许可类型 |
| `LICENSE-STAGE` | 许可阶段 |
| `LICENSE-CS` | 许可状态 |
| `LEE-CURRENT` | 当前被许可方 |
| `PEE-CURRENT` | 当前质权人 |
| `PLEDGE-TYPE` | 质押类型 |
| `PLEDGE-STAGE` | 质押阶段 |
| `LAWTXT` | 法律文本 |
| `ASSIGN-FLAG` | 转让标志 |
| `ASSIGN-TIMES` | 转让次数 |
| `ASSIGN-NO` | 转让编号 |
| `ASSIGN-TYPE` | 转让类型 |
| `LICENCE-FLAG` | 许可标志 |
| `LICENCE-TIMES` | 许可次数 |
| `PLEGE-FLAG` | 质押标志 |
| `PLEDGE-TIMES` | 质押次数 |
| `REE-FLAG` | 复审标志 |
| `LGI-FLAG` | 诉讼标志 |
| `LGI-TIMES` | 诉讼次数 |
| `ACTION-TYPES` | 动作类型 |
| `CUSTOMS-FLAG` | 海关备案标志 |
| `ALL-FLAG` | 所有标志 |
| `TOVALIDE-DATE` | 有效日期 |
| `FLAG-337` | 337 调查标志 |

### 引证字段 (Citation Fields)

| 字段代码 | 说明 |
|----------|------|
| `CT` | 引证文献 |
| `CTFW` | 被引证文献 |
| `CT-SELF` | 自引文献 |
| `CT-OTH` | 他引文献 |
| `CTFW-SELF` | 自被引文献 |
| `CTFW-OTH` | 他被引文献 |
| `CT-TIMES` | 引证次数 |
| `CTFW-TIMES` | 被引证次数 |
| `CT-SELF-TIMES` | 自引次数 |
| `CT-OTH-TIMES` | 他引次数 |
| `CTFW-SELF-TIMES` | 自被引次数 |
| `CTFW-OTH-TIMES` | 他被引次数 |
| `FCT` | 族引证 |
| `FCTFW` | 族被引证 |
| `CT-AP` | 引证申请人 |
| `CTFW-AP` | 被引证申请人 |
| `FCT-AP` | 族引证申请人 |
| `FCTFW-AP` | 族被引证申请人 |
| `CT-NO` | 引证号 |
| `CTFW-NO` | 被引证号 |
| `CT-AUTH` | 引证机构 |
| `CTFW-AUTH` | 被引证机构 |
| `CT-CODE` | 引证代码 |
| `CT-X` | X 类引证 |
| `FCT-TIMES` | 族引证次数 |
| `FCTFW-TIMES` | 族被引证次数 |
| `CTNP` | 非专利引证 |
| `CT-SOURCE` | 引证来源 |
| `CTFW-SOURCE` | 被引证来源 |

### 其他字段

| 字段代码 | 说明 |
|----------|------|
| `DOC-DC` | 文档分类 |
| `RAND-DWPI` | DWPI 随机 |

### 语义检索关键词

| 关键词 | 说明 |
|--------|------|
| `R` | 通用语义检索 |
| `RAD` | 按申请日排序的语义检索 |
| `RPD` | 按公开日排序的语义检索 |

---

## 三、检索式示例

### 基本检索
```
ti=空调
tiab=(空调 or "air conditioner" or 空气调节)
```

### 多字段布尔组合
```
    ti=(空调 or "air conditioner")
and ab=(蒸发器 or evaporator)
and ipc=F25B
not ap=格力
```

### 日期范围
```
pd=[20200101 to 20241231]
(20110101<=pd<=20130101)
(pd>20190101)
```

### 邻近检索
```
tiab=(空调 (2w) 蒸发器)
```

### 频率检索
```
tiab=("机器人" (3f))
```

### 公司树检索
```
ap=(TREE@"清华大学")
```

### 语义检索
```
R=(CN101850473B)
    RAD=(CN101850473B)
and tiab=(发动机 or engine)
```

### 通配符检索
```
ti=comput$ and ab=m?chine
```
