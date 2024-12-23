use super::ast::Expr;

#[derive(Clone, Debug)]
pub struct PassArg {
    pub name: String,
    pub value: Expr
}

impl clap::builder::ValueParserFactory for PassArg {
    type Parser = PassArgParser;

    fn value_parser() -> Self::Parser {
        PassArgParser
    }
}

#[derive(Clone, Debug)]
pub struct PassArgParser;

impl clap::builder::TypedValueParser for PassArgParser {
    type Value = PassArg;

    fn parse_ref(
        &self,
        _cmd: &clap::Command,
        _arg: Option<&clap::Arg>,
        value: &std::ffi::OsStr,
    ) -> Result<Self::Value, clap::Error> {
        let v = value.to_str().ok_or(clap::Error::new(clap::error::ErrorKind::InvalidUtf8))?;
        let mut parts = v.trim().split("=");

        let name = parts.next().ok_or(clap::Error::new(clap::error::ErrorKind::NoEquals))?;
        let value_str = parts.next().ok_or(clap::Error::new(clap::error::ErrorKind::InvalidValue))?;

        let mut errors = Vec::new();
        let value = super::grammar::ExprParser::new().parse(&mut errors, value_str).unwrap();

        if !errors.is_empty() {
            return Err(clap::Error::new(clap::error::ErrorKind::ValueValidation));
        }

        Ok(PassArg {
            name: name.to_string(),
            value: *value,
        })
    }
}
