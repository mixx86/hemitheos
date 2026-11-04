#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum Errno {
    HEMITHEOS_ENOENT = 2,
    HEMITHEOS_ENOMEM = 12,
    HEMITHEOS_EINVAL = 22,
    HEMITHEOS_TABLE_MAP_FAILED = 23,
    HEMITHEOS_EOPNOTSUPP = 95,
}
