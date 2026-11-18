//! boss 击杀**掉落表**与**触发索引**——引擎与求解器的单一数据源。
//!
//! 为什么住在这里：掉落原先只以扁平 `i32` 数组内联在 `scene.rs`（`BOSS_EVENT_SPAWNS`），
//! 求解器要建「击杀 boss → 掉落物」边只能另抄一份；抄的那份必然与引擎漂移
//! （同 [`crate::rules::gate`] 的教训）。现在：表只此一份（[`event_spawns`]），
//! 「这只怪死了触发哪一组」只此一份（[`event_of`] / [`post_victory_event_of`]）。
//!
//! 证据（A 级，逐行可复现）：
//! - `reference/src/deobf/a.java:2041-2197`：`bossEventSpawns` 8 组 `byte[][]`，
//!   每组每 4 字节 `(型, 格x, 格y, extra)`；组号 = 击杀钩子的实参。
//! - `a.java:2198`：`bossTypeOrder = {75, 74, 68, 67, 69, 71}`（**组号 = 该表下标**，
//!   不是型号；`case 67 → (3)`、`case 71 → (5)` 与表对得上）。
//! - `a.java:8912-8921`：`spawnBossEvent(idx)` 逐 4 元组
//!   `spawnEntity(型, x<<5, y<<5, extra)` + `markEntityDesRemoved`（DES 态，登场动画后
//!   变可交互实体——不是直接发道具）。
//! - `a.java:10059-10093`：击杀拍 `while (--i >= 0) if (型 == bossTypeOrder[i]) spawnBossEvent(i)`，
//!   整段被 `currentFloor > 50` 跳过；随后 `switch(型)` 是**剧本臂**（73/74/75，见
//!   [`event_of`] 注释），与掉落臂独立、可叠加。
//! - `a.java:10144-10149`：胜利动画收尾 `case 69` 且 `currentFloor == 35` →
//!   `spawnBossEvent(7)`（post-anim 的独立钩子，见 [`post_victory_event_of`]）。
//!
//! 全表值域 `{16,17,24,26,27,29,30,32}` 全是 cat2 拾取物/钥匙/丹药
//! （`docs/spec/evidence-boss.md` §R-boss-3，A 级行号同表）⇒ 掉落**只生成实体**，
//! 「踩上去才 pickup」由引擎/已移植的拾取臂负责；本模块**不发钥匙、不进背包**。

/// 一条掉落生成：`(型, 格x, 格y, extra)`。
///
/// 对应 Java `byte[4]`；像素坐标由调用方换算（`x << 5`, `y << 5`，a.java:8918）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BossSpawn {
    pub ty: i32,
    pub x: i32,
    pub y: i32,
    /// Java `spawnEntity` 的 `param`（本表全 0）。
    pub extra: i32,
}

const fn s(ty: i32, x: i32, y: i32, extra: i32) -> BossSpawn {
    BossSpawn { ty, x, y, extra }
}

/// `bossEventSpawns`（a.java:2041-2197）逐组转写，组号 = 下标。
///
/// 每组注释给出 `a.java` 行区间与触发者，便于逐条 diff。
static EVENT_SPAWNS: [&[BossSpawn]; 8] = [
    // [0] a.java:2042-2091 — 75 哪吒击杀：威力丹29×3 / 黄钥匙26×3 / 金刚丹30×3 / 长寿丹32×3
    &[
        s(29, 1, 3, 0),
        s(29, 2, 3, 0),
        s(29, 3, 3, 0),
        s(26, 1, 4, 0),
        s(26, 2, 4, 0),
        s(26, 3, 4, 0),
        s(30, 9, 3, 0),
        s(30, 10, 3, 0),
        s(30, 11, 3, 0),
        s(32, 9, 4, 0),
        s(32, 10, 4, 0),
        s(32, 11, 4, 0),
    ],
    // [1] a.java:2092-2141 — 74 巨灵神击杀：同比 12 件（换到 (4..8) 区）
    &[
        s(29, 4, 4, 0),
        s(29, 4, 5, 0),
        s(29, 5, 4, 0),
        s(26, 7, 4, 0),
        s(26, 8, 4, 0),
        s(26, 8, 5, 0),
        s(30, 4, 7, 0),
        s(30, 4, 8, 0),
        s(30, 5, 8, 0),
        s(32, 8, 8, 0),
        s(32, 7, 8, 0),
        s(32, 8, 7, 0),
    ],
    // [2] a.java:2142 — 68 太上老君击杀：红钥匙27×4（十字：(6,5)(5,6)(7,6)(6,7)）
    &[s(27, 6, 5, 0), s(27, 5, 6, 0), s(27, 7, 6, 0), s(27, 6, 7, 0)],
    // [3] a.java:2143 — 67 啸天犬击杀：金勺子17 @(6,4)
    &[s(17, 6, 4, 0)],
    // [4] a.java:2144 — 69 杨戬击杀：芭蕉扇16 @(5,5)
    &[s(16, 5, 5, 0)],
    // [5] a.java:2145 — 71 玉帝二阶段击杀：捆仙绳24 + 长寿丹32×3 + 红钥匙27（**5 条**）
    &[
        s(24, 6, 2, 0),
        s(32, 5, 1, 0),
        s(32, 6, 1, 0),
        s(32, 7, 1, 0),
        s(27, 6, 6, 0),
    ],
    // [6] a.java:2146-2195 — 73 天蓬元帅击杀且 floor==40：长寿丹32×3 / 黄钥匙26×3 / 威力丹29×3 / 金刚丹30×3
    &[
        s(32, 2, 3, 0),
        s(32, 3, 3, 0),
        s(32, 4, 3, 0),
        s(26, 8, 3, 0),
        s(26, 9, 3, 0),
        s(26, 10, 3, 0),
        s(29, 3, 5, 0),
        s(29, 4, 5, 0),
        s(29, 5, 5, 0),
        s(30, 7, 5, 0),
        s(30, 8, 5, 0),
        s(30, 9, 5, 0),
    ],
    // [7] a.java:2196 — 69 击杀且 floor==35（胜利动画收尾 post-anim）：长寿丹32×3
    &[s(32, 4, 4, 0), s(32, 5, 4, 0), s(32, 6, 4, 0)],
];

/// `bossEventSpawns[event]`；越界 = 空表（Java 会 `ArrayIndexOutOfBounds`，
/// 调用方本不该传越界值——这里给空表便于求解器容错与单测，语义上「无掉落」）。
pub fn event_spawns(event: usize) -> &'static [BossSpawn] {
    match EVENT_SPAWNS.get(event) {
        Some(spawns) => spawns,
        None => &[],
    }
}

/// `bossTypeOrder`（a.java:2198）：击杀钩子查表用的型序列，**下标 = 掉落组号**。
pub fn boss_order() -> &'static [i32] {
    &[75, 74, 68, 67, 69, 71]
}

/// 「这只怪死了，击杀拍触发哪一组掉落」——掉落臂（a.java:10059-10070）。
///
/// 语义（A 级，逐句对应）：
/// 1. `currentFloor > 50` 整段钩子跳过 ⇒ 无掉落（a.java:10060-10062）；
/// 2. `bossTypeOrder` **倒序**查表，命中即 `spawnBossEvent(下标)` 并 `break`
///    （a.java:10063-10068）——`rposition` 与 Java 倒序扫等价（型不重复）；
/// 3. `case 73` 且 `currentFloor == 40` → `spawnBossEvent(6)`（a.java:10071-10077）。
///    73 不在 `bossTypeOrder`，非 40 层没有掉落组；同 case 的
///    `scriptLineFlags[...] = true` + `loadLevelScript(22)` 是**剧本臂**，不由本模块负责；
/// 4. `case 74` / `case 75` 的 `loadLevelScript(21/20)`、`tryRunScene(20,true)` 同为剧本臂，
///    这里**不返回额外组**——它们的掉落已经由第 2 步（74→1、75→0）给出；
/// 5. 其余型号（含 70/72 等收尾脚本特判）无掉落 ⇒ `None`。
pub fn event_of(ety: i32, floor: i32) -> Option<usize> {
    if floor > 50 {
        return None;
    }
    if let Some(i) = boss_order().iter().rposition(|&b| b == ety) {
        return Some(i);
    }
    if ety == 73 && floor == 40 {
        return Some(6);
    }
    None
}

/// 「胜利动画收尾（post-anim）另触发的掉落组」——与 [`event_of`] 的击杀拍钩子**独立**。
///
/// `a.java:10144-10149`：`switch(型) case 69: if (currentFloor == 35) {
/// spawnBossEvent(7); loadLevelScript(36); }`。故 69@35 一次击杀 = 击杀拍组 4（芭蕉扇）
/// + 收尾组 7（长寿丹×3）；`loadLevelScript(36)` 是剧本臂，不由本模块负责。
///
/// 其余型号无 post-anim 掉落（67/68/74 落 default，a.java:10129-10137）。
pub fn post_victory_event_of(ety: i32, floor: i32) -> Option<usize> {
    if ety == 69 && floor == 35 {
        Some(7)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_67_is_exactly_one_gold_spoon_at_6_4() {
        // a.java:2143 `{17, 6, 4, 0}`
        assert_eq!(event_spawns(3), &[s(17, 6, 4, 0)]);
    }

    #[test]
    fn event_69_is_exactly_one_palm_fan_at_5_5() {
        // a.java:2144 `{16, 5, 5, 0}`
        assert_eq!(event_spawns(4), &[s(16, 5, 5, 0)]);
    }

    #[test]
    fn event_71_has_five_entries_with_rope_24_and_red_key_27() {
        // a.java:2145 —— **5 条**（不是 6 条）：24@(6,2)、32@(5,1)(6,1)(7,1)、27@(6,6)。
        // 表内无 26；任务书口述「6 条含两把 27/26」与字节不符，以字节为准
        // （docs/spec/evidence-boss.md §R-boss-3 同表）。
        let ev = event_spawns(5);
        assert_eq!(ev.len(), 5, "a.java:2145 恰 5 条");
        assert!(ev.contains(&s(24, 6, 2, 0)), "捆仙绳 24@(6,2)");
        assert!(ev.contains(&s(27, 6, 6, 0)), "红钥匙 27@(6,6)");
        assert_eq!(
            ev.iter().filter(|e| e.ty == 32).count(),
            3,
            "长寿丹 32×3 @(5,1)(6,1)(7,1)"
        );
        assert_eq!(ev.iter().filter(|e| e.ty == 26).count(), 0, "表内无 26");
    }

    #[test]
    fn event_out_of_range_is_empty() {
        assert!(event_spawns(8).is_empty(), "只有 8 组（0..=7）");
        assert!(event_spawns(99).is_empty());
    }

    #[test]
    fn event_of_matches_boss_order_indices() {
        // a.java:2198 bossTypeOrder ⇒ 下标即组号
        assert_eq!(event_of(75, 1), Some(0));
        assert_eq!(event_of(74, 1), Some(1));
        assert_eq!(event_of(68, 25), Some(2));
        assert_eq!(event_of(67, 1), Some(3));
        assert_eq!(event_of(69, 35), Some(4));
        assert_eq!(event_of(71, 49), Some(5));
        // 50 层（含）仍触发；51 层起整段跳过（a.java:10060）
        assert_eq!(event_of(75, 50), Some(0));
        assert_eq!(event_of(75, 51), None);
        // 不在表内的普通怪无掉落
        assert_eq!(event_of(41, 1), None);
        assert_eq!(event_of(72, 50), None);
    }

    #[test]
    fn event_of_73_only_at_floor_40() {
        // a.java:10071-10077：case 73 的掉落组只在 floor==40
        assert_eq!(event_of(73, 40), Some(6));
        assert_eq!(event_of(73, 39), None);
        assert_eq!(event_of(73, 41), None);
        // >50 层整段跳过优先于特例
        assert_eq!(event_of(73, 51), None);
    }

    #[test]
    fn post_victory_event_of_69_is_group_7_only_at_floor_35() {
        // a.java:10144-10149
        assert_eq!(post_victory_event_of(69, 35), Some(7));
        assert_eq!(post_victory_event_of(69, 36), None);
        assert_eq!(post_victory_event_of(68, 35), None);
    }

    /// 全表逐条钉死（防转写漂移）：组号 → 完整 4 元组序列。
    /// 期望值直接抄自 `a.java:2042-2196` 的字节展开（与 `parse` 脚本产物一致）。
    #[test]
    fn full_table_matches_java_source() {
        let expect: [&[(i32, i32, i32, i32)]; 8] = [
            &[
                (29, 1, 3, 0), (29, 2, 3, 0), (29, 3, 3, 0),
                (26, 1, 4, 0), (26, 2, 4, 0), (26, 3, 4, 0),
                (30, 9, 3, 0), (30, 10, 3, 0), (30, 11, 3, 0),
                (32, 9, 4, 0), (32, 10, 4, 0), (32, 11, 4, 0),
            ],
            &[
                (29, 4, 4, 0), (29, 4, 5, 0), (29, 5, 4, 0),
                (26, 7, 4, 0), (26, 8, 4, 0), (26, 8, 5, 0),
                (30, 4, 7, 0), (30, 4, 8, 0), (30, 5, 8, 0),
                (32, 8, 8, 0), (32, 7, 8, 0), (32, 8, 7, 0),
            ],
            &[(27, 6, 5, 0), (27, 5, 6, 0), (27, 7, 6, 0), (27, 6, 7, 0)],
            &[(17, 6, 4, 0)],
            &[(16, 5, 5, 0)],
            &[(24, 6, 2, 0), (32, 5, 1, 0), (32, 6, 1, 0), (32, 7, 1, 0), (27, 6, 6, 0)],
            &[
                (32, 2, 3, 0), (32, 3, 3, 0), (32, 4, 3, 0),
                (26, 8, 3, 0), (26, 9, 3, 0), (26, 10, 3, 0),
                (29, 3, 5, 0), (29, 4, 5, 0), (29, 5, 5, 0),
                (30, 7, 5, 0), (30, 8, 5, 0), (30, 9, 5, 0),
            ],
            &[(32, 4, 4, 0), (32, 5, 4, 0), (32, 6, 4, 0)],
        ];
        for (ev, rows) in expect.iter().enumerate() {
            let got: Vec<(i32, i32, i32, i32)> = event_spawns(ev)
                .iter()
                .map(|b| (b.ty, b.x, b.y, b.extra))
                .collect();
            assert_eq!(&got[..], *rows, "event {ev} 与 a.java 字节组不一致");
        }
        assert_eq!(
            expect.iter().map(|g| g.len()).sum::<usize>(),
            50,
            "全表 50 条（12+12+4+1+1+5+12+3）"
        );
    }
}
