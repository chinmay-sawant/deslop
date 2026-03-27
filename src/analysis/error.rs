use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("failed to configure {language} parser: {message}")]
    ParserConfiguration { language: &'static str, message: String },
    #[error("tree-sitter returned no parse tree for {language}")]
    MissingParseTree { language: &'static str },
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub(crate) fn parser_configuration(language: &'static str, message: impl Into<String>) -> Self {
        Self::ParserConfiguration {
            language,
            message: message.into(),
        }
    }

    pub(crate) fn missing_parse_tree(language: &'static str) -> Self {
        Self::MissingParseTree { language }
    }
}