pub use eyre::{bail, eyre as error, Error};
use inpt::split::{Quoted, SingleQuoted, Word};
use inpt::{inpt, inpt_step, Inpt, InptStep};
use std::io::{BufRead, BufWriter};
use std::{borrow::Cow, fs::File, path::Path};
use std::{fmt, fs};
pub use uciedit_macros::UciSection;

extern crate self as uciedit;

#[derive(Inpt, Clone, Copy)]
pub enum Token<'a> {
    Q(Quoted<&'a str>),
    Sq(SingleQuoted<&'a str>),
    W(Word<&'a str>),
}

impl<'a> Token<'a> {
    pub fn as_str(&self) -> Cow<str> {
        // TODO: inpt doesn't currently do unescaping
        match self {
            Token::Q(x) => x.unescape(),
            Token::Sq(x) => x.unescape(),
            Token::W(x) => Cow::Borrowed(x.inner),
        }
    }

    pub fn from_display(s: &impl fmt::Display, arena: &'a Arena) -> Self {
        Self::from_string(s.to_string(), arena)
    }

    pub fn from_string(s: String, arena: &'a Arena) -> Self {
        if s.contains(|c: char| c.is_whitespace()) {
            let q = arena.alloc(format!("{:?}", s));
            Token::Q(Quoted {
                inner: &q[1..q.len() - 1],
            })
        } else {
            let s = arena.alloc(s);
            Token::W(Word { inner: s })
        }
    }

    pub fn from_str(s: &'a str, arena: &'a Arena) -> Self {
        if s.contains(|c: char| c.is_whitespace()) {
            let q = arena.alloc(format!("{:?}", s));
            Token::Q(Quoted {
                inner: &q[1..q.len() - 1],
            })
        } else {
            Token::W(Word { inner: s })
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
    Skip,
}

impl fmt::Display for Line<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Line::Empty => writeln!(f),
            Line::Comment(text) => writeln!(f, "{}", text),
            Line::Section { ty, name: None } => writeln!(f, "config {}", ty),
            Line::Section {
                ty,
                name: Some(name),
            } => writeln!(f, "config {} {}", ty, name),
            Line::Option { option, value } => writeln!(f, "\toption {} {}", option, value),
            Line::List { list, item } => writeln!(f, "\tlist {} {}", list, item),
            Line::Skip => Ok(()),
        }
    }
}

impl<'a> Line<'a> {
    pub fn parse(line: &'a str) -> Result<Self, Error> {
        let rest = line.trim();
        if rest.is_empty() {
            return Ok(Line::Empty);
        }
        if rest.starts_with('#') {
            return Ok(Line::Comment(line));
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
                let (ty, rest) = match inpt_step::<Token>(rest) {
                    InptStep { data: Ok(ty), rest } => (ty, rest),
                    _ => bail!("could not parse section type"),
                };
                let name: Option<_> = if rest.is_empty() {
                    None
                } else {
                    match inpt::<Token>(rest) {
                        Ok(name) => Some(name),
                        _ => bail!("could not parse section name"),
                    }
                };
                Line::Section { ty, name }
            }
            "option" => {
                let (option, value): (Token, Token) =
                    inpt(rest).map_err(|err| error!("could not parse option: {err}"))?;
                Line::Option { option, value }
            }
            "list" => {
                let (list, item): (Token, Token) =
                    inpt(rest).map_err(|err| error!("could not parse list: {err}"))?;
                Line::List { list, item }
            }
            kw => bail!("unknown UCI keyword {kw:?}"),
        })
    }
}

pub type Lines<'a> = Vec<Line<'a>>;
pub type Arena = typed_arena::Arena<String>;

pub trait UciSection<'a>: Sized {
    fn read(lines: &Lines<'a>, index: usize) -> Result<Self, Error>;
    fn write(&self, lines: &mut Lines<'a>, arena: &'a Arena, index: usize) -> Result<(), Error>;
    fn append(
        &self,
        lines: &mut Lines<'a>,
        arena: &'a Arena,
        name: Option<&'a str>,
    ) -> Result<(), Error>;
}

pub fn parse_config<V>(
    path: impl AsRef<Path>,
    with: impl FnOnce(Lines<'_>) -> Result<V, Error>,
) -> Result<V, Error> {
    let text = fs::read_to_string(path)?;
    with(parse_config_string(&text)?)
}

pub fn parse_config_string(config: &str) -> Result<Lines<'_>, Error> {
    config.lines().map(Line::parse).collect()
}

/// TODO: async version?
pub fn rewrite_config<V>(
    path: impl AsRef<Path>,
    with: impl for<'a> FnOnce(&mut Lines<'a>, &'a Arena) -> Result<V, Error>,
) -> Result<V, Error> {
    use std::io::Write;

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
    let arena = Arena::new();
    for line in BufReader::new(&mut *locked).lines() {
        let line = arena.alloc(line?);
        lines.push(Line::parse(line)?);
    }
    let v = with(&mut lines, &arena)?;
    locked.set_len(0)?;
    let mut writer = BufWriter::new(&mut *locked);
    for line in lines {
        write!(writer, "{}", line)?;
    }
    Ok(v)
}

pub fn rewrite_config_string(
    config: String,
    with: impl for<'a> FnOnce(&mut Lines<'a>, &'a Arena) -> Result<(), Error>,
) -> Result<String, Error> {
    use std::fmt::Write;

    let mut lines = Vec::new();
    let arena = Arena::new();
    for line in arena.alloc(config).lines() {
        lines.push(Line::parse(line)?);
    }
    with(&mut lines, &arena)?;
    let mut writer = String::new();
    for line in lines {
        write!(writer, "{}", line)?;
    }
    Ok(writer)
}

#[test]
fn test_read_section() {
    let original = r"
config bar
    option always 0
    option yes 1
    list many 2
    list many 3
    list many 4
";

    let expected = Bar {
        always: 0,
        yes: Some(1),
        no: None,
        many: vec![2, 3, 4],
    };

    #[derive(UciSection, PartialEq, Eq, Debug)]
    struct Bar {
        always: i32,
        yes: Option<i32>,
        no: Option<i32>,
        many: Vec<i32>,
    }

    let lines = parse_config_string(original).unwrap();
    let parsed = Bar::read(&lines, 1).unwrap();

    println!(
        "===Original==={original}===Parsed===\n{parsed:#?}\n===Expected===\n{expected:#?}\n====="
    );
    assert_eq!(parsed, expected);
}

#[test]
fn test_append_section() {
    let original = r"
config foo
    option hello world
    # a comment here
";

    let expected = r"
config foo
    option hello world
    # a comment here

config bar appended
    option always 0
    option yes 1
    list many 2
    list many 3
    list many 4
";

    #[derive(UciSection)]
    struct Bar {
        always: i32,
        yes: Option<i32>,
        no: Option<i32>,
        many: Vec<i32>,
    }

    let edited = rewrite_config_string(original.to_string(), |lines, arena| {
        Bar {
            always: 0,
            yes: Some(1),
            no: None,
            many: vec![2, 3, 4],
        }
        .append(lines, arena, Some("appended"))
    })
    .unwrap();

    println!("===Original==={original}===Edited==={edited}===Expected==={expected}=====");
    assert_eq!(edited.replace("\t", "    "), expected);
}

#[test]
fn test_edit_section() {
    let original = r"
# top comment
config bar named
    # always comment
    option always 0

    # no comment
    option no 1

    # many comment
    list many 2

    # few comment
    list few 3
    list few 4
    list few 5

    # ignored comment
    option ignored 6

# bottom comment
";

    let expected = r"
# top comment
config bar named
    # always comment
    option always 0

    # no comment

    # many comment
    list many 2

    # few comment
    list few 5
    option yes 1
    list many 3
    list many 4

    # ignored comment
    option ignored 6

# bottom comment
";

    #[derive(UciSection)]
    struct Bar {
        always: i32,
        yes: Option<i32>,
        no: Option<i32>,
        many: Vec<i32>,
        few: Vec<i32>,
    }

    let edited = rewrite_config_string(original.to_string(), |lines, arena| {
        Bar {
            always: 0,
            yes: Some(1),
            no: None,
            many: vec![2, 3, 4],
            few: vec![5],
        }
        .write(lines, arena, 2)
    })
    .unwrap();

    println!("===Original==={original}===Edited==={edited}===Expected==={expected}=====");
    assert_eq!(edited.replace("\t", "    "), expected);
}
