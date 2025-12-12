/// E1106: Long function (too many lines)
/// Severity: LOW
/// LLM confusion: 2 (LOW)
///
/// Description: This function is too long (exceeds 250 lines), making it hard to understand,
/// test, and maintain. Long functions often do too many things and are difficult to reason
/// about. Fix by breaking the function into smaller, focused functions that each do one thing
/// well. Each function should have a clear, single purpose.
///
/// Mitigation: Use `#![warn(clippy::too_many_lines)]` to detect overly long functions. Follow
/// the Single Responsibility Principle - each function should do one thing. Extract logical
/// sections into helper functions with descriptive names.

pub fn e1106_bad_long_function(x: i32) -> i32 {
    // PROBLEM E1106: Function exceeds 250 lines
    let mut result = 0;
    result += x;
    result += x + 2;
    result += x + 3;
    result += x + 4;
    result += x + 5;
    result += x + 6;
    result += x + 7;
    result += x + 8;
    result += x + 9;
    result += x + 10;
    result += x + 11;
    result += x + 12;
    result += x + 13;
    result += x + 14;
    result += x + 15;
    result += x + 16;
    result += x + 17;
    result += x + 18;
    result += x + 19;
    result += x + 20;
    result += x + 21;
    result += x + 22;
    result += x + 23;
    result += x + 24;
    result += x + 25;
    result += x + 26;
    result += x + 27;
    result += x + 28;
    result += x + 29;
    result += x + 30;
    result += x + 31;
    result += x + 32;
    result += x + 33;
    result += x + 34;
    result += x + 35;
    result += x + 36;
    result += x + 37;
    result += x + 38;
    result += x + 39;
    result += x + 40;
    result += x + 41;
    result += x + 42;
    result += x + 43;
    result += x + 44;
    result += x + 45;
    result += x + 46;
    result += x + 47;
    result += x + 48;
    result += x + 49;
    result += x + 50;
    result += x + 51;
    result += x + 52;
    result += x + 53;
    result += x + 54;
    result += x + 55;
    result += x + 56;
    result += x + 57;
    result += x + 58;
    result += x + 59;
    result += x + 60;
    result += x + 61;
    result += x + 62;
    result += x + 63;
    result += x + 64;
    result += x + 65;
    result += x + 66;
    result += x + 67;
    result += x + 68;
    result += x + 69;
    result += x + 70;
    result += x + 71;
    result += x + 72;
    result += x + 73;
    result += x + 74;
    result += x + 75;
    result += x + 76;
    result += x + 77;
    result += x + 78;
    result += x + 79;
    result += x + 80;
    result += x + 81;
    result += x + 82;
    result += x + 83;
    result += x + 84;
    result += x + 85;
    result += x + 86;
    result += x + 87;
    result += x + 88;
    result += x + 89;
    result += x + 90;
    result += x + 91;
    result += x + 92;
    result += x + 93;
    result += x + 94;
    result += x + 95;
    result += x + 96;
    result += x + 97;
    result += x + 98;
    result += x + 99;
    result += x + 100;
    result += x + 101;
    result += x + 102;
    result += x + 103;
    result += x + 104;
    result += x + 105;
    result += x + 106;
    result += x + 107;
    result += x + 108;
    result += x + 109;
    result += x + 110;
    result += x + 111;
    result += x + 112;
    result += x + 113;
    result += x + 114;
    result += x + 115;
    result += x + 116;
    result += x + 117;
    result += x + 118;
    result += x + 119;
    result += x + 120;
    result += x + 121;
    result += x + 122;
    result += x + 123;
    result += x + 124;
    result += x + 125;
    result += x + 126;
    result += x + 127;
    result += x + 128;
    result += x + 129;
    result += x + 130;
    result += x + 131;
    result += x + 132;
    result += x + 133;
    result += x + 134;
    result += x + 135;
    result += x + 136;
    result += x + 137;
    result += x + 138;
    result += x + 139;
    result += x + 140;
    result += x + 141;
    result += x + 142;
    result += x + 143;
    result += x + 144;
    result += x + 145;
    result += x + 146;
    result += x + 147;
    result += x + 148;
    result += x + 149;
    result += x + 150;
    result += x + 151;
    result += x + 152;
    result += x + 153;
    result += x + 154;
    result += x + 155;
    result += x + 156;
    result += x + 157;
    result += x + 158;
    result += x + 159;
    result += x + 160;
    result += x + 161;
    result += x + 162;
    result += x + 163;
    result += x + 164;
    result += x + 165;
    result += x + 166;
    result += x + 167;
    result += x + 168;
    result += x + 169;
    result += x + 170;
    result += x + 171;
    result += x + 172;
    result += x + 173;
    result += x + 174;
    result += x + 175;
    result += x + 176;
    result += x + 177;
    result += x + 178;
    result += x + 179;
    result += x + 180;
    result += x + 181;
    result += x + 182;
    result += x + 183;
    result += x + 184;
    result += x + 185;
    result += x + 186;
    result += x + 187;
    result += x + 188;
    result += x + 189;
    result += x + 190;
    result += x + 191;
    result += x + 192;
    result += x + 193;
    result += x + 194;
    result += x + 195;
    result += x + 196;
    result += x + 197;
    result += x + 198;
    result += x + 199;
    result += x + 200;
    result += x + 201;
    result += x + 202;
    result += x + 203;
    result += x + 204;
    result += x + 205;
    result += x + 206;
    result += x + 207;
    result += x + 208;
    result += x + 209;
    result += x + 210;
    result += x + 211;
    result += x + 212;
    result += x + 213;
    result += x + 214;
    result += x + 215;
    result += x + 216;
    result += x + 217;
    result += x + 218;
    result += x + 219;
    result += x + 220;
    result += x + 221;
    result += x + 222;
    result += x + 223;
    result += x + 224;
    result += x + 225;
    result += x + 226;
    result += x + 227;
    result += x + 228;
    result += x + 229;
    result += x + 230;
    result += x + 231;
    result += x + 232;
    result += x + 233;
    result += x + 234;
    result += x + 235;
    result += x + 236;
    result += x + 237;
    result += x + 238;
    result += x + 239;
    result += x + 240;
    result += x + 241;
    result += x + 242;
    result += x + 243;
    result += x + 244;
    result += x + 245;
    result += x + 246;
    result += x + 247;
    result += x + 248;
    result += x + 249;
    result += x + 250;
    result
}

pub fn e1106_entry() -> Result<(), Box<dyn std::error::Error>> {
    let _ = e1106_bad_long_function(1);
    Ok(())
}

// ============================================================================
// GOOD EXAMPLES - Proper alternatives
// ============================================================================

/// GOOD: Use a loop or iterator instead of repetitive code
pub fn e1106_good_use_loop(x: i32) -> i32 {
    let mut result = 0;
    for i in 0..=250 {
        result += x + i;
    }
    result
}

/// GOOD: Use iterator methods
pub fn e1106_good_iterator(x: i32) -> i32 {
    (0..=250).map(|i| x + i).sum()
}

/// GOOD: Break long functions into focused helpers
fn e1106_good_validate_input(x: i32) -> Result<i32, &'static str> {
    if x < 0 {
        return Err("input must be non-negative");
    }
    Ok(x)
}

fn e1106_good_compute_base(x: i32) -> i32 {
    (0..100).map(|i| x + i).sum()
}

fn e1106_good_compute_extended(x: i32) -> i32 {
    (100..=250).map(|i| x + i).sum()
}

pub fn e1106_good_split_into_helpers(x: i32) -> Result<i32, &'static str> {
    let x = e1106_good_validate_input(x)?;
    let base = e1106_good_compute_base(x);
    let extended = e1106_good_compute_extended(x);
    Ok(base + extended)
}

/// GOOD: Use formula instead of iteration when possible
pub fn e1106_good_formula(x: i32) -> i32 {
    // Sum of (x + 0) + (x + 1) + ... + (x + 250)
    // = 251 * x + (0 + 1 + 2 + ... + 250)
    // = 251 * x + (250 * 251) / 2
    let n = 250i64;
    let count = n + 1;
    let sum_of_offsets = (n * (n + 1)) / 2;
    (count * x as i64 + sum_of_offsets) as i32
}

// ============================================================================
// GOOD EXAMPLES unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn e1106_good_loop_matches_iterator() {
        assert_eq!(e1106_good_use_loop(2), e1106_good_iterator(2));
    }

    #[test]
    fn e1106_good_formula_matches_loop() {
        assert_eq!(e1106_good_use_loop(3), e1106_good_formula(3));
    }

    #[test]
    fn e1106_good_split_helpers_validates_input() {
        assert!(e1106_good_split_into_helpers(-1).is_err());
    }
}
