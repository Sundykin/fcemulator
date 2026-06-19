//! Cheat engine: Game Genie codes (ROM read substitution) + raw RAM pokes.
//!
//! - **Game Genie** codes (6 or 8 letters) target ROM ($8000-$FFFF) and patch
//!   the *read* path; 8-letter codes only substitute when the original byte
//!   matches the compare value.
//! - **Raw** cheats target any address; RAM addresses (< $8000) are written
//!   every frame (optionally gated by a compare value).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cheat {
    /// Original code / text the user entered.
    pub code: String,
    pub addr: u16,
    pub value: u8,
    pub compare: Option<u8>,
    pub enabled: bool,
    pub desc: String,
}

impl Cheat {
    /// Is this a ROM patch (Game Genie style, $8000-$FFFF read substitution)?
    pub fn is_rom_patch(&self) -> bool {
        self.addr >= 0x8000
    }
}

const GG: [char; 16] = [
    'A', 'P', 'Z', 'L', 'G', 'I', 'T', 'Y', 'E', 'O', 'X', 'U', 'K', 'S', 'V', 'N',
];

fn gg_digit(c: char) -> Option<u8> {
    let c = c.to_ascii_uppercase();
    GG.iter().position(|&g| g == c).map(|i| i as u8)
}

/// Decode a 6 or 8 letter NES Game Genie code into (addr, value, compare).
pub fn decode_game_genie(code: &str) -> Option<(u16, u8, Option<u8>)> {
    let n: Vec<u8> = code
        .chars()
        .filter(|c| !c.is_whitespace())
        .map(gg_digit)
        .collect::<Option<_>>()?;
    if n.len() != 6 && n.len() != 8 {
        return None;
    }
    let g = |i: usize| n[i] as u16;
    let addr = 0x8000u16
        | ((g(3) & 7) << 12)
        | ((g(5) & 7) << 8)
        | ((g(4) & 8) << 8)
        | ((g(2) & 7) << 4)
        | ((g(1) & 8) << 4)
        | (g(1) & 7)
        | (g(0) & 8);

    if n.len() == 6 {
        let value = ((n[0] & 7) << 4) | ((n[5] & 8) << 4) | (n[4] & 7) | (n[3] & 8);
        Some((addr, value, None))
    } else {
        let value = ((n[0] & 7) << 4) | ((n[7] & 8) << 4) | (n[6] & 7) | (n[5] & 8);
        let compare = ((n[7] & 7) << 4) | ((n[6] & 8) << 4) | (n[5] & 7) | (n[4] & 8);
        Some((addr, value, Some(compare)))
    }
}

/// Parse a raw cheat: `AAAA:VV` (poke) or `AAAA?CC:VV` (compare-gated), hex.
pub fn decode_raw(code: &str) -> Option<(u16, u8, Option<u8>)> {
    let s = code.trim();
    let (addr_part, rest) = s.split_once(':')?;
    let value = u8::from_str_radix(rest.trim(), 16).ok()?;
    if let Some((a, c)) = addr_part.split_once('?') {
        let addr = u16::from_str_radix(a.trim(), 16).ok()?;
        let compare = u8::from_str_radix(c.trim(), 16).ok()?;
        Some((addr, value, Some(compare)))
    } else {
        let addr = u16::from_str_radix(addr_part.trim(), 16).ok()?;
        Some((addr, value, None))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_poke() {
        assert_eq!(decode_raw("07F8:09"), Some((0x07F8, 0x09, None)));
    }
    #[test]
    fn raw_compare() {
        assert_eq!(decode_raw("0010?00:99"), Some((0x0010, 0x99, Some(0x00))));
    }
    #[test]
    fn gg_all_a() {
        // All 'A' (=0) must map to $8000 / $00, no compare.
        assert_eq!(decode_game_genie("AAAAAA"), Some((0x8000, 0x00, None)));
    }
    #[test]
    fn gg_lengths() {
        assert!(decode_game_genie("SXIOPO").is_some()); // 6-letter
        assert!(decode_game_genie("YEUZUGAA").map_or(false, |(_, _, c)| c.is_some())); // 8 → compare
        assert!(decode_game_genie("BADCODE").is_none()); // 7 letters invalid
    }
    #[test]
    fn gg_addr_in_rom() {
        let (addr, _, _) = decode_game_genie("SXIOPO").unwrap();
        assert!(addr >= 0x8000);
    }
}

/// Decode any supported cheat format into a [`Cheat`].
pub fn parse(code: &str, desc: &str) -> Option<Cheat> {
    let (addr, value, compare) = decode_game_genie(code).or_else(|| decode_raw(code))?;
    Some(Cheat {
        code: code.trim().to_string(),
        addr,
        value,
        compare,
        enabled: true,
        desc: desc.to_string(),
    })
}
