# pickup 位点探针留档（P1-a2 批4 账②）

> DDDSA P1-a2 位点探针实证（sprite 表扫描——临时探针跑后即删，本档=留痕）。
> 探针形：`game_data::sprite::SpriteTable::parse(assets/raw/sprite{n})` 扫
> records（type_code/x>>5/y>>5）。

| 型 | 位点（spriteN 格） | 用于夹具 |
|---|---|---|
| 13 | sprite52 (3,8) | fx-item13 |
| 26 | sprite2 (3,4)/(4,4)/(3,5)（三格） | fx-pickup-26（双拾实况） |
| 27 | sprite8 (2,2) | fx-pickup-27 |
| 28 | sprite3 (5,3) | fx-pickup-28 |
| 29 | sprite3 (1,9)、sprite12 (6,1) 等 37 处 | fx-pickup-29 / 29f12 |
| 30 | sprite3 (2,1) | fx-pickup-30 |
| 31 | sprite3 (2,2) | fx-pickup-31 |
| 33 | sprite5 (11,11) | fx-pickup-33 |
| 37 | sprite9 (9,7) | fx-pickup-37 |
| 34/35/36/38 | sprite17 (2,2)/sprite33 (10,10)/sprite13 (6,3)/sprite11 (10,2) | 备选 |
| 81-88 | 仅 t83（sprite1/2/3 等·触发型） | —（无 87/88 图面实体） |

**floor1 剧情触发面注记**：sprite1（floor1）位点备选弃——嫦娥戏（case83
到达触发 script19）致 236 拍止（fx-pickup-26 首版实证）。

**default no-op 臂可达性分析**（批4 递延注记）：pickup 入口守卫=renderCategory
2|4（scene.rs:4123 区）→ cat 2/4 型域=13..=40|79|80|85|86 **全被 case 覆盖**
——图拾路径 **default 臂不可达**。可行夹具车=脚本 GIN 面（op_gin→
pickup_item_type(value) 裸型注入——scene.rs:2765 同式）；default 臂夹具随
脚本面批次落（显式递延，非遗漏）。
