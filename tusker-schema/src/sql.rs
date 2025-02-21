use std::fmt;

#[derive(Debug, Default)]
pub struct StatementBuilder {
    parts: Vec<String>,
}

impl StatementBuilder {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn part(&mut self, s: impl ToString) {
        self.parts.push(s.to_string());
    }
    pub fn ident(&mut self, s: impl ToString) {
        self.part(quote_ident(&s.to_string()));
    }
}

impl fmt::Display for StatementBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.parts.join(" "))
    }
}

pub fn quote_ident(ident: &str) -> String {
    // FIXME add escapes
    format!("\"{}\"", ident)
}
