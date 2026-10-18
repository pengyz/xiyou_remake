# 对抗审查：task-10 decode-equivalence 全族对照 + 真相源清单挂闸（76b536f）——**通过**

> 审查人：adversary。对象：76b536f（decode_equivalence.rs +214 / gates/cli.py +18-10）。
> 对照：003 硬 2（decode-equivalence 常驻）+ 硬 4（真相源层级禁第三态）。
> 手段：RED temp-archive 复现、像素对照体逐行核、新闸独立跑、零漂移独立复跑、三对账。

## 判定：**通过**（硬 2/硬 4 全销）；1 条数字小疵不卡闸

## 1. 硬 2 decode-equivalence —— ✅ 销

- **像素逐位属实**（我逐行核对照体 :229-252）：`bin.argb != dec.argb` = Vec<u32>
  **真逐 u32 直比**（img_sig/fnv 只用于清单面，不用于等价判定）+ 尺寸前置 +
  `first_pixel_diff`（首差异像素 #i/x,y/两侧值）——声称与内容一字不虚。
- **全族覆盖**：16 容器（Mid 图在缺/尺寸/ARGB/帧名序 `{name}_{i:02}`/信封
  version-format-name/JSON count 半侧）+ maplv55（raw_width/height **u16 原始值语义**
  注记 + terrain/flip 逐元素首差异下标）+ sprite55（count_lo/hi + records 逐字段 +
  extra[] 逐字节）——003 硬 2 的"逐资产解码等价"面齐全。
- **E3 计数锚不消失**：`count_assertion_family_binary_side_must_not_vanish`（:93-101，
  RESOURCE_IMAGE_COUNTS 16 容器逐项）+ 像素清单（:113+）——旧锚升格对照 ✓。

## 2. 硬 4 真相源清单挂闸 —— ✅ 销

- 新闸 `asset-truth-manifest`（tools/asset_manifest.py verify：源 jar sha256+条目偏移+
  产物哈希三态核禁第三态）挂 check/status/all + `asset-manifest` 子命令
  （gates/cli.py hunk 亲验）；
- **我独立跑单闸**：`[✓ PASS] asset-truth-manifest — entries=138，art 三态核通过` ✓；
- 失败面声明（真相源/产物漂移、第三态 ⇒ FAIL、纯只读）+ 基线影响（9→10、无既有
  基线改动、ratchet 提示留收尾）符合 AGENTS §5 门禁改动纪律 ✓。

## 3. TDD / 三对账 / 零漂移 —— ✅

- **RED 抽查独立复现成功**：temp-archive 9c10af3 + 叠终稿测试 ⇒ `E0432: unresolved
  imports game_data::asset_pipeline` ×2，与 message 逐字吻合 ✓；
- 三对账：2 文件内容/声称/写范围一致；**gates/cli.py 写域透明声明**（lead 明派）= 合规；
- **零漂移独立复跑**：game-data 全绿（35+1+3+9+1，decode_equivalence 3/3）EXIT=0。

## 4. 小疵（不卡闸）

- message "game-data 48 测" vs 实测 **49**（35+1+3+9+1——decode_equivalence 从 2 测
  增至 3）——计数笔误级，无需返工。

## 5. 移交确认

- 我建议两件已落：失败清单首差异偏移 ✓、CI 夹具口径注记（XIYOU_ASSETS_RAW 可改指）✓；
- task-9 两整改（Java 锚勘误/recipe 口径）落文后我核勘误清单（另行）；
- ratchet-update（decode_equivalence(test):2 新二进制）留 task-6 收尾 ✓ 约定不变。
