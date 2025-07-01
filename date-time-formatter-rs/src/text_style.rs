#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum TextStyle {
    Full,
    FullStandalone,
    Short,
    ShortStandalone,
    Narrow,
    NarrowStandalone,
}
