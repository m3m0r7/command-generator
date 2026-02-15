#[derive(Debug, Clone)]
pub(crate) struct CommandHead {
    pub(crate) name: String,
    pub(crate) prefixed_builtin: bool,
    pub(crate) prefixed_command: bool,
    pub(crate) prefixed_backslash: bool,
    pub(crate) token_index: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct SegmentToken {
    pub(crate) raw: String,
    pub(crate) cooked: String,
    pub(crate) start: usize,
}

#[derive(Debug)]
pub(crate) struct RuntimeCheck {
    pub(crate) ok: bool,
    pub(crate) note: Option<String>,
}
