use lasso::{Spur, ThreadedRodeo};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::OnceLock;

static INTERNER: OnceLock<ThreadedRodeo> = OnceLock::new();

#[inline]
pub fn global_interner() -> &'static ThreadedRodeo {
    INTERNER.get_or_init(ThreadedRodeo::default)
}

/// 4-byte interned symbol identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolId(pub Spur);

impl SymbolId {
    #[inline(always)]
    pub fn new(raw: &str) -> Self {
        Self(global_interner().get_or_intern(raw))
    }

    #[inline(always)]
    pub fn as_str(&self) -> &str {
        global_interner().resolve(&self.0)
    }
}

impl std::fmt::Display for SymbolId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<&str> for SymbolId {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<String> for SymbolId {
    fn from(s: String) -> Self {
        Self::new(&s)
    }
}

impl Serialize for SymbolId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for SymbolId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(SymbolId::new(&s))
    }
}
