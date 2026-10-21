pub const fn KB(n: u64) -> u64 {
    n << 10
}

pub const fn MB(n: u64) -> u64 {
    n << 20
}

pub const fn GB(gb: u64) -> u64 {
    gb << 30
}

pub const fn is_aligned(val: u64, align: u64) -> bool {
    if val % align == 0 {
        return true;
    }
    false
}

pub const fn BIT(nr: u64) -> u64 {
    return 1 << nr;
}
