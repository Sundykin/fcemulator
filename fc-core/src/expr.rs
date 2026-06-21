//! Tiny expression evaluator for conditional breakpoints (debugger).
//!
//! Modelled on Mesen2's `ExpressionEvaluator.Nes` — a breakpoint may carry a
//! condition like `a == 0xff && scanline >= 30`; the debugger only halts when it
//! evaluates non-zero. Operands resolve against the live CPU/PPU state ([`Ctx`]).
//!
//! Grammar (precedence low→high): `||` · `&&` · `== != < <= > >=` · `| ^` · `&`
//! · `+ -` · `* / %` · unary `! - ~` · primary (`number`, `identifier`, `( … )`).
//! Numbers are decimal or `0x`-hex. Comparisons yield 1/0. Unknown identifier or
//! a parse error makes the whole expression evaluate to "true" (fail-open), so a
//! mistyped condition still stops rather than silently never firing.

/// Live state a condition can reference. `value`/`addr` are set for read/write
/// watchpoint conditions; -1 means "not applicable".
#[derive(Debug, Clone, Copy)]
pub struct Ctx {
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub p: u8,
    pub sp: u8,
    pub pc: u16,
    pub cycles: u64,
    pub scanline: u16,
    pub dot: u16,
    pub value: i64,
    pub addr: i64,
}

/// Evaluate `expr` against `ctx`. Returns true when the expression is non-zero
/// (or on parse error — fail-open, see module docs).
pub fn eval_cond(expr: &str, ctx: &Ctx) -> bool {
    let mut p = Parser {
        s: expr.as_bytes(),
        i: 0,
        ctx,
    };
    match p.parse() {
        Some(v) => {
            p.skip_ws();
            // Trailing garbage → treat as malformed → fail-open.
            if p.i == p.s.len() {
                v != 0
            } else {
                true
            }
        }
        None => true,
    }
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
    ctx: &'a Ctx,
}

impl Parser<'_> {
    fn parse(&mut self) -> Option<i64> {
        self.logic_or()
    }

    fn skip_ws(&mut self) {
        while self.i < self.s.len() && self.s[self.i].is_ascii_whitespace() {
            self.i += 1;
        }
    }

    fn eat(&mut self, tok: &str) -> bool {
        self.skip_ws();
        let t = tok.as_bytes();
        if self.s[self.i..].starts_with(t) {
            // Don't let `=` match the front of `==`, etc.
            self.i += t.len();
            true
        } else {
            false
        }
    }

    fn peek2(&mut self, a: &str, b: &str) -> Option<bool> {
        // Disambiguate single vs double-char ops without consuming.
        self.skip_ws();
        if self.s[self.i..].starts_with(a.as_bytes()) {
            Some(true)
        } else if self.s[self.i..].starts_with(b.as_bytes()) {
            Some(false)
        } else {
            None
        }
    }

    fn logic_or(&mut self) -> Option<i64> {
        let mut v = self.logic_and()?;
        while self.eat("||") {
            let r = self.logic_and()?;
            v = ((v != 0) || (r != 0)) as i64;
        }
        Some(v)
    }
    fn logic_and(&mut self) -> Option<i64> {
        let mut v = self.compare()?;
        while self.eat("&&") {
            let r = self.compare()?;
            v = ((v != 0) && (r != 0)) as i64;
        }
        Some(v)
    }
    fn compare(&mut self) -> Option<i64> {
        let v = self.bit_or()?;
        self.skip_ws();
        // Two-char comparators first.
        if self.eat("==") {
            return Some((v == self.bit_or()?) as i64);
        }
        if self.eat("!=") {
            return Some((v != self.bit_or()?) as i64);
        }
        if self.eat("<=") {
            return Some((v <= self.bit_or()?) as i64);
        }
        if self.eat(">=") {
            return Some((v >= self.bit_or()?) as i64);
        }
        if matches!(self.peek2("<", "<"), Some(_)) && self.eat("<") {
            return Some((v < self.bit_or()?) as i64);
        }
        if self.eat(">") {
            return Some((v > self.bit_or()?) as i64);
        }
        Some(v)
    }
    fn bit_or(&mut self) -> Option<i64> {
        let mut v = self.bit_and()?;
        loop {
            self.skip_ws();
            // `|` but not `||`
            if self.peek2("||", "|") == Some(false) && self.eat("|") {
                v |= self.bit_and()?;
            } else if self.eat("^") {
                v ^= self.bit_and()?;
            } else {
                break;
            }
        }
        Some(v)
    }
    fn bit_and(&mut self) -> Option<i64> {
        let mut v = self.additive()?;
        loop {
            self.skip_ws();
            // `&` but not `&&`
            if self.peek2("&&", "&") == Some(false) && self.eat("&") {
                v &= self.additive()?;
            } else {
                break;
            }
        }
        Some(v)
    }
    fn additive(&mut self) -> Option<i64> {
        let mut v = self.mul()?;
        loop {
            if self.eat("+") {
                v += self.mul()?;
            } else if self.eat("-") {
                v -= self.mul()?;
            } else {
                break;
            }
        }
        Some(v)
    }
    fn mul(&mut self) -> Option<i64> {
        let mut v = self.unary()?;
        loop {
            if self.eat("*") {
                v *= self.unary()?;
            } else if self.eat("/") {
                let d = self.unary()?;
                v = if d == 0 { 0 } else { v / d };
            } else if self.eat("%") {
                let d = self.unary()?;
                v = if d == 0 { 0 } else { v % d };
            } else {
                break;
            }
        }
        Some(v)
    }
    fn unary(&mut self) -> Option<i64> {
        self.skip_ws();
        if self.eat("!") {
            return Some((self.unary()? == 0) as i64);
        }
        if self.eat("~") {
            return Some(!self.unary()?);
        }
        if self.eat("-") {
            return Some(-self.unary()?);
        }
        self.primary()
    }
    fn primary(&mut self) -> Option<i64> {
        self.skip_ws();
        if self.eat("(") {
            let v = self.parse()?;
            if !self.eat(")") {
                return None;
            }
            return Some(v);
        }
        let start = self.i;
        // Number: 0x-hex or decimal.
        if self.i < self.s.len() && self.s[self.i].is_ascii_digit() {
            if self.s[self.i..].starts_with(b"0x") || self.s[self.i..].starts_with(b"0X") {
                self.i += 2;
                let h = self.i;
                while self.i < self.s.len() && self.s[self.i].is_ascii_hexdigit() {
                    self.i += 1;
                }
                return i64::from_str_radix(std::str::from_utf8(&self.s[h..self.i]).ok()?, 16).ok();
            }
            while self.i < self.s.len() && self.s[self.i].is_ascii_digit() {
                self.i += 1;
            }
            return std::str::from_utf8(&self.s[start..self.i])
                .ok()?
                .parse()
                .ok();
        }
        // Identifier (operand).
        while self.i < self.s.len()
            && (self.s[self.i].is_ascii_alphanumeric() || self.s[self.i] == b'_')
        {
            self.i += 1;
        }
        if self.i == start {
            return None;
        }
        let id = std::str::from_utf8(&self.s[start..self.i])
            .ok()?
            .to_ascii_lowercase();
        let c = self.ctx;
        Some(match id.as_str() {
            "a" => c.a as i64,
            "x" => c.x as i64,
            "y" => c.y as i64,
            "p" => c.p as i64,
            "sp" | "s" => c.sp as i64,
            "pc" => c.pc as i64,
            "cycles" | "cyc" | "cycle" => c.cycles as i64,
            "scanline" | "sl" => c.scanline as i64,
            "dot" | "cyc_dot" | "pixel" => c.dot as i64,
            "value" | "val" => c.value,
            "addr" | "address" => c.addr,
            // status-flag bits as conveniences
            "n" => ((c.p >> 7) & 1) as i64,
            "v" => ((c.p >> 6) & 1) as i64,
            "d" => ((c.p >> 3) & 1) as i64,
            "i" => ((c.p >> 2) & 1) as i64,
            "z" => ((c.p >> 1) & 1) as i64,
            "c" => (c.p & 1) as i64,
            _ => return None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ctx() -> Ctx {
        Ctx {
            a: 0xFF,
            x: 0x10,
            y: 0,
            p: 0xA5,
            sp: 0xFD,
            pc: 0x8000,
            cycles: 100,
            scanline: 30,
            dot: 5,
            value: -1,
            addr: -1,
        }
    }
    #[test]
    fn basics() {
        let c = ctx();
        assert!(eval_cond("a == 0xff", &c));
        assert!(!eval_cond("a == 0", &c));
        assert!(eval_cond("a == 0xff && scanline >= 30", &c));
        assert!(!eval_cond("a == 0xff && scanline > 30", &c));
        assert!(eval_cond("x + 1 == 17", &c)); // 0x10 + 1 = 17
        assert!(eval_cond("(a & 0x0f) == 0x0f", &c));
        assert!(eval_cond("pc == 0x8000 || a == 0", &c));
        assert!(eval_cond("c == 1", &c)); // P=0xA5 → carry set
        assert!(eval_cond("!z", &c)); // P=0xA5 → zero clear
    }
    #[test]
    fn fail_open() {
        // Malformed / unknown → true (stops rather than silently never firing).
        assert!(eval_cond("a == ", &ctx()));
        assert!(eval_cond("bogus > 3", &ctx()));
    }
}
