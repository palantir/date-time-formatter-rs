#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum SignStyle {
    Normal,
    Always,
    _Never,
    NotNegative,
    ExceedsPad,
}

impl SignStyle {
    pub fn parse(&self, positive: bool, strict: bool, fixed_width: bool) -> bool {
        match self {
            SignStyle::Normal => !positive || !strict,
            SignStyle::Always | SignStyle::ExceedsPad => true,
            SignStyle::_Never | SignStyle::NotNegative => !strict && !fixed_width,
        }
    }
}
