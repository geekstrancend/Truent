/// Move Manual Overflow Check Detector
///
/// Detects H60 vulnerability: hand-rolled bit-shift-and-mask overflow/bounds
/// checking instead of built-in checked arithmetic.
///
/// The vulnerability occurs when:
/// 1. A left-shift (`<<`) is used to scale a fixed-point value
/// 2. Its safety is verified against a hardcoded hex bitmask rather than a
///    language-level checked-arithmetic primitive
/// 3. The mask threshold is off by even one bit, the check silently passes
///    values that overflow the intended width
/// 4. A corrupted intermediate can then propagate into token-delta/liquidity
///    accounting math
///
/// This is exactly what happened in the Cetus Protocol hack ($223M, May
/// 2025, Sui): a `checked_shlw`-style overflow guard used the wrong bitmask
/// threshold (`0xFFFFFFFFFFFFFFFF << 192` instead of `1 << 192`), letting
/// values that should have aborted pass through and corrupt an add-liquidity
/// token-delta calculation. This detector cannot verify whether a given
/// mask is mathematically correct — only a manual audit or symbolic
/// execution can — so it flags the pattern itself as a high-risk review
/// item rather than a confirmed bug.
use lazy_static::lazy_static;
use regex::Regex;
use truent_core::{Finding, Severity};

lazy_static! {
    static ref LEFT_SHIFT: Regex = Regex::new(r"<<\s*\d").unwrap();
    static ref HEX_MASK_COMPARISON: Regex =
        Regex::new(r"0x[0-9a-fA-F]+\s*(<<|>>)|(<=|>=|<|>)\s*0x[0-9a-fA-F]+").unwrap();
}

pub fn detect_move_manual_overflow_check(source: &str, file_path: &str) -> Vec<Finding> {
    let mut findings = Vec::new();

    let lines: Vec<&str> = source.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with("//") || !LEFT_SHIFT.is_match(line) {
            continue;
        }

        let window_start = i.saturating_sub(15);
        let window_end = lines.len().min(i + 15);
        let window = lines[window_start..window_end].join("\n");

        if HEX_MASK_COMPARISON.is_match(&window) {
            findings.push(
                Finding::new(
                    "move_manual_overflow_check".to_string(),
                    Severity::High,
                    file_path.to_string(),
                    i + 1,
                    0,
                    "Left-shift operation near a hex-bitmask comparison suggests a hand-rolled \
                     overflow/bounds check instead of built-in checked arithmetic; a single \
                     wrong bit in the mask threshold silently admits overflowing values — see \
                     Cetus Protocol ($223M, May 2025), where a checked_shlw-style guard used \
                     0xFFFFFFFFFFFFFFFF << 192 instead of 1 << 192"
                        .to_string(),
                    line.trim().to_string(),
                )
                .with_metadata("chain".to_string(), "move".to_string()),
            );
        }
    }

    findings
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_shift_near_hex_mask_comparison() {
        let source = r#"
module cetus::clmm_math {
    const MASK: u256 = 0xFFFFFFFFFFFFFFFF << 192;

    public fun checked_shlw(n: u256): u256 {
        assert!(n <= MASK, 1);
        n << 64
    }
}
"#;
        let findings = detect_move_manual_overflow_check(source, "clmm_math.move");
        assert!(findings
            .iter()
            .any(|f| f.invariant_id == "move_manual_overflow_check"));
    }

    #[test]
    fn does_not_flag_plain_shift_without_mask_comparison() {
        let source = r#"
module example::math {
    public fun scale(n: u64): u64 {
        n << 8
    }
}
"#;
        let findings = detect_move_manual_overflow_check(source, "math.move");
        assert!(findings.is_empty());
    }

    #[test]
    fn does_not_flag_checked_arithmetic_without_shift() {
        let source = r#"
module example::safe_math {
    public fun add(a: u64, b: u64): u64 {
        assert!(a <= 0xFFFFFFFFFFFFFFFF - b, 1);
        a + b
    }
}
"#;
        let findings = detect_move_manual_overflow_check(source, "safe_math.move");
        assert!(findings.is_empty());
    }
}
