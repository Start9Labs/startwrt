use eyre::{bail, eyre, Error};
use inpt::split::{Quoted, SingleQuoted, Word};
use inpt::{inpt, inpt_step, Inpt, InptStep};
use std::fmt;
use std::io::{BufRead, BufWriter, Write};
use std::{borrow::Cow, fs::File, path::Path};

#[derive(Inpt, Clone, Copy)]
pub enum Token<'a> {
    Q(Quoted<&'a str>),
    Sq(SingleQuoted<&'a str>),
    W(Word<&'a str>),
}

impl Token<'_> {
    fn as_str(&self) -> Cow<str> {
        // TODO: inpt doesn't currently do unescaping
        match self {
            Token::Q(x) => x.unescape(),
            Token::Sq(x) => x.unescape(),
            Token::W(x) => Cow::Borrowed(x.inner),
        }
    }
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Q(quoted) => write!(f, "\"{}\"'", quoted.inner),
            Token::Sq(single_quoted) => write!(f, "'{}'", single_quoted.inner),
            Token::W(word) => write!(f, "{}", word.inner),
        }
    }
}

pub enum Line<'a> {
    Empty,
    Comment(&'a str),
    Section {
        ty: Token<'a>,
        name: Option<Token<'a>>,
    },
    Option {
        option: Token<'a>,
        value: Token<'a>,
    },
    List {
        list: Token<'a>,
        item: Token<'a>,
    },
}

impl fmt::Display for Line<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Line::Empty => writeln!(f),
            Line::Comment(text) => writeln!(f, "#{}", text),
            Line::Section { ty, name: None } => writeln!(f, "config {}", ty),
            Line::Section {
                ty,
                name: Some(name),
            } => writeln!(f, "config {} {}", ty, name),
            Line::Option { option, value } => writeln!(f, "\toption {} {}", option, value),
            Line::List { list, item } => writeln!(f, "\tlist {} {}", list, item),
        }
    }
}

impl<'a> Line<'a> {
    pub fn parse(line: &'a str) -> Result<Self, Error> {
        let rest = line.trim();
        if rest.is_empty() {
            return Ok(Line::Empty);
        }
        if let Some(rest) = rest.strip_prefix("#") {
            return Ok(Line::Comment(rest));
        }
        let InptStep {
            data: Ok(keyword),
            rest,
        } = inpt_step::<Token>(rest)
        else {
            unreachable!()
        };
        Ok(match &*keyword.as_str() {
            "config" => {
                let (ty, name): (Token, Token) =
                    inpt(rest).map_err(|err| eyre!("could not parse section: {err}"))?;
                let name: Option<_> = match name {
                    Token::W(Word { inner: "" }) => None,
                    _ => Some(name),
                };
                Line::Section { ty, name }
            }
            "option" => {
                let (option, value): (Token, Token) =
                    inpt(rest).map_err(|err| eyre!("could not parse option: {err}"))?;
                Line::Option { option, value }
            }
            "list" => {
                let (list, item): (Token, Token) =
                    inpt(rest).map_err(|err| eyre!("could not parse list: {err}"))?;
                Line::List { list, item }
            }
            kw => bail!("unknown UCI keyword {kw:?}"),
        })
    }
}

pub type Bump = bumpalo::Bump;
pub type Lines<'a> = Vec<Line<'a>>;

pub trait Section<'a>: Sized {
    fn read(lines: &mut Lines<'a>, index: usize) -> Result<Self, Error>;
    fn write(&self, lines: &mut Lines<'a>, bump: &'a Bump, index: usize) -> Result<(), Error>;
    fn append(&self, lines: &mut Lines<'a>, bump: &'a Bump) -> Result<(), Error>;
}

/// TODO: async version?
pub fn rewrite_config<V>(
    path: impl AsRef<Path>,
    with: impl for<'a> FnOnce(&mut Lines<'a>, &'a Bump) -> Result<V, Error>,
) -> Result<V, Error> {
    use fd_lock_rs::{FdLock, LockType};
    use std::io::BufReader;
    let file = File::options()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(path)?;
    let mut locked = FdLock::lock(file, LockType::Exclusive, true)?;
    let mut lines = Vec::new();
    let bump = Bump::new();
    for line in BufReader::new(&mut *locked).lines() {
        let line = bump.alloc_str(&line?); // TODO: no extra alloc?
        lines.push(Line::parse(line)?);
    }
    let v = with(&mut lines, &bump)?;
    locked.set_len(0)?;
    let mut writer = BufWriter::new(&mut *locked);
    for line in lines {
        write!(writer, "{}", line)?;
    }
    Ok(v)
}
