//! ④ script DSL 词法拆分辅助（供后续脚本解释器用；**只做词法切分，不解释语义**）。
//!
//! 脚本行内指令以**空格**分隔；每条指令 = **定长 3 字母指令词** + 以 `_` 分隔的整数参数，
//! 最后一个参数以空格（而非 `_`）结尾。本模块把一条指令流拆成「指令 token + 下划线参数」。
//!
//! # 证据（A 级）
//!
//! - 定长前缀匹配：`reference/seed/a.java:6121` `string = object.substring(n, n + 3)` 配
//!   `a.java:6122-6433` 的 16 路 `if/else if`（`docs/findings/script-dsl-semantics.md:36`）。
//! - 参数分隔符 tokenizer：`reference/seed/a.java:6440-6446` `private int a(String, int, String)`——
//!   从游标找下一个分隔符（`"_"` 或 `" "`），子串 `Integer.parseInt`，游标写入 `this.bs`
//!   （`docs/findings/script-dsl-semantics.md:37`）。
//! - 指令间以空格分隔、行内线性执行（`docs/findings/script-dsl-semantics.md:38`）；
//!   `this.n[]` 字面量每条以空格结尾（`reference/seed/a.java:550`）。
//! - GUTS 行形态 `<层号> <指令流…>`：`docs/findings/script-dsl-semantics.md:41`
//!   （编号行 = 层号 = `this.n[]` 下标），资源文件 `script` 的 `GUTS:` 段（CRLF 行尾）实测同形。
//!
//! 已知边界（不发明行为）：本模块只切词，不校验参数个数/语义（解释器职责），
//! 不处理 `MOT_IF`/`MOT_L{n}`/`POST` 等非指令常量（`docs/findings/script-dsl-semantics.md:145-147`）。

use crate::error::Error;

/// 16 个指令词（定长 3 字母，大小写敏感）。
/// Evidence: `docs/findings/script-dsl-semantics.md:36`；`reference/seed/a.java:6122-6433` 的 `.equals` 链。
pub const OPCODES: [&str; 16] = [
    "CES", "MOV", "TAK", "DES", "ROS", "SEE", "GUT", "GIN", "ADD", "GLV", "RES", "SWD", "MVS", "LAY",
    "END", "SMS",
];

/// 是否已知指令词（仅供参考；词法切分**不**因未知指令词报错）。
pub fn is_known_opcode(opcode: &str) -> bool {
    OPCODES.contains(&opcode)
}

/// 一条指令：3 字母指令词 + 整数参数序列。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Instr<'a> {
    /// 指令 token 原文（如 `TAK_8_9`），便于回指脚本串。
    pub raw: &'a str,
    /// 3 字母指令词（如 `TAK`）。Evidence: `a.java:6121`。
    pub opcode: &'a str,
    /// 下划线切分出的整数参数（如 `[8, 9]`）。Evidence: `a.java:6440-6446`。
    pub args: Vec<i32>,
}

/// 切分单个指令 token：3 字母指令词 + `_` 分隔的整数参数。
///
/// `CES_84_6_11` → (`CES`, `[84, 6, 11]`)；`SWD` → (`SWD`, `[]`)；`DES_-59_0` → (`DES`, `[-59, 0]`，负参数
/// 见 `docs/findings/script-dsl-semantics.md:58`)。
///
/// 游标模型与解释器等价：指令词占 3 字符（`a.java:6121`），其后每个参数前缀 `_`
/// （末参数在行内以空格结尾，切词后即 token 尾，`docs/findings/script-dsl-semantics.md:38`）。
pub fn split_instr_token(token: &str) -> Result<Instr<'_>, Error> {
    let token = token.trim();
    if token.len() < 3 {
        return Err(Error::BadToken {
            token: token.to_string(),
            reason: "指令 token 至少 3 字符（定长指令词，a.java:6121）",
        });
    }
    let (opcode, mut rest) = token.split_at(3);
    if !opcode.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(Error::BadToken {
            token: token.to_string(),
            reason: "指令词必须是 3 个 ASCII 字母（a.java:6121 substring(n, n+3)）",
        });
    }
    let mut args = Vec::new();
    while !rest.is_empty() {
        // 每个参数以 '_' 前缀引导（a.java:6440-6446 的分隔符 tokenizer）
        let body = rest.strip_prefix('_').ok_or_else(|| Error::BadToken {
            token: token.to_string(),
            reason: "参数必须以 '_' 分隔（a.java:6440-6446）",
        })?;
        let (arg, next) = match body.find('_') {
            Some(at) => (&body[..at], &body[at..]),
            None => (body, ""),
        };
        let value = arg.parse::<i32>().map_err(|_| Error::BadToken {
            token: token.to_string(),
            reason: "参数不是整数（Integer.parseInt，a.java:6445）",
        })?;
        args.push(value);
        rest = next;
    }
    Ok(Instr { raw: token, opcode, args })
}

/// 把一条指令流按空格切成指令 token 并逐个拆参数。
/// Evidence: 指令间以空格分隔、线性执行（`docs/findings/script-dsl-semantics.md:38`）。
///
/// 空流（全空白）返回空 vec。
pub fn lex_instr_stream(stream: &str) -> Result<Vec<Instr<'_>>, Error> {
    stream.split_whitespace().map(split_instr_token).collect()
}

/// 一条 GUTS 行：`<层号> <指令流…>`。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GutsLine<'a> {
    /// 层号（= `this.n[]` 下标；缺号层由 `GUT_n` 跳转触达，`docs/findings/script-dsl-semantics.md:160`）。
    pub layer: i32,
    /// 指令流（可为空）。
    pub instrs: Vec<Instr<'a>>,
}

/// 解析 `script` 资源 `GUTS:` 段的一行：`<层号> <指令流…>`（CRLF 行尾自动容忍）。
/// Evidence: `docs/findings/script-dsl-semantics.md:41`；`assets/raw/script` 的 `GUTS:` 段。
///
/// 注意：`GUTS:` 段以单独一行 `END` 结束（与 `GAME:` 段同），那不是 GUTS 行，
/// 本函数对它返回 [`Error::BadGutsLine`]。
pub fn parse_guts_line(line: &str) -> Result<GutsLine<'_>, Error> {
    let line = line.trim_matches(|c| c == '\r' || c == '\n').trim();
    let bad = || Error::BadGutsLine { line: line.to_string() };
    let (head, rest) = match line.split_once(char::is_whitespace) {
        Some((head, rest)) => (head, rest),
        None => (line, ""),
    };
    let layer = head.parse::<i32>().map_err(|_| bad())?;
    let instrs = lex_instr_stream(rest).map_err(|_| bad())?;
    Ok(GutsLine { layer, instrs })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 指令 token 与下划线参数切分（样例取自 `assets/raw/script` GUTS 第 0 行，
    /// 与 `docs/findings/script-dsl-semantics.md:70` 引文一致）。
    #[test]
    fn split_token_and_underscore_args() {
        let instr = split_instr_token("CES_84_6_11").unwrap();
        assert_eq!((instr.opcode, instr.args.as_slice()), ("CES", &[84, 6, 11][..]));
        let instr = split_instr_token("MOV_72_3_7_1_8").unwrap();
        assert_eq!((instr.opcode, instr.args.as_slice()), ("MOV", &[72, 3, 7, 1, 8][..]));
        // 无参数指令（`docs/findings/script-dsl-semantics.md:57` SWD 无参数）
        let instr = split_instr_token("SWD").unwrap();
        assert_eq!(instr.opcode, "SWD");
        assert!(instr.args.is_empty(), "SWD 不带参数");
        // 负参数（`docs/findings/script-dsl-semantics.md:58` `DES_-59_0`）
        let instr = split_instr_token("DES_-59_0").unwrap();
        assert_eq!((instr.opcode, instr.args.as_slice()), ("DES", &[-59, 0][..]));
        assert!(is_known_opcode("GLV"));
        assert!(!is_known_opcode("MOT")); // MOT_IF 不是指令（findings:145）
    }

    #[test]
    fn lex_stream_splits_on_spaces() {
        let instrs =
            lex_instr_stream("TAK_22_22 ROS_4_3 TAK_23_32 MOV_72_3_7_1_8").unwrap();
        assert_eq!(instrs.len(), 4);
        assert_eq!(instrs[0].raw, "TAK_22_22");
        assert_eq!(instrs[3].args, vec![72, 3, 7, 1, 8]);
        assert_eq!(lex_instr_stream("   ").unwrap().len(), 0);
    }

    #[test]
    fn bad_tokens_rejected() {
        // 指令词不足 3 字符
        assert!(matches!(
            split_instr_token("AB"),
            Err(Error::BadToken { .. })
        ));
        // 参数不是整数
        assert!(matches!(
            split_instr_token("CES_a_b"),
            Err(Error::BadToken { .. })
        ));
        // 参数未以 '_' 引导
        assert!(matches!(
            split_instr_token("CES0_5"),
            Err(Error::BadToken { .. })
        ));
    }

    /// GUTS 行 = `<层号> <指令流>`（`docs/findings/script-dsl-semantics.md:41`）。
    #[test]
    fn guts_line_layer_and_instrs() {
        let parsed = parse_guts_line("0 CES_84_6_11 MOV_0_5_11 TAK_8_9\r\n").unwrap();
        assert_eq!(parsed.layer, 0);
        assert_eq!(parsed.instrs.len(), 3);
        assert_eq!(parsed.instrs[0].opcode, "CES");
        assert_eq!(parsed.instrs[1].args, vec![0, 5, 11]);
        // 段落终止行 END 不是 GUTS 行
        assert!(matches!(parse_guts_line("END"), Err(Error::BadGutsLine { .. })));
    }
}
