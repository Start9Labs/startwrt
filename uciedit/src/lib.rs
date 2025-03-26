use eyre::Context;
pub use eyre::{bail, eyre as error, Error};
pub use inpt::inpt;
use inpt::split::{Quoted, SingleQuoted, Spaced};
use inpt::{inpt_step, Inpt, InptStep};
use std::fmt::Display;
use std::io::{BufRead, BufWriter, Seek};
use std::{borrow::Cow, fs::File, path::Path};
use std::{fmt, fs};
pub use uciedit_macros::UciSection;

pub mod openwrt;

pub fn parse_config<V>(
    path: impl AsRef<Path>,
    with: impl FnOnce(Sections) -> Result<V, Error>,
) -> Result<V, Error> {
    let text = fs::read_to_string(path)?;
    parse_config_string(&text, with)
}

pub fn parse_config_string<V>(
    config: &str,
    with: impl FnOnce(Sections) -> Result<V, Error>,
) -> Result<V, Error> {
    let lines = config.lines().map(Line::parse).collect::<Result<_, _>>()?;
    with(Sections {
        lines: &lines,
        index: 0,
        started: false,
    })
}

/// TODO: async version?
pub fn rewrite_config<V>(
    path: impl AsRef<Path>,
    with: impl for<'a> FnOnce(SectionsMut) -> Result<V, Error>,
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
        let parse =
            Line::parse(line).with_context(|| format!("syntax error on line {}", lines.len()))?;
        lines.push(parse);
    }
    let v = with(SectionsMut {
        lines: &mut lines,
        index: 0,
        arena: &arena,
        section_start: None,
        retain: true,
    })?;
    locked.set_len(0)?;
    locked.seek(std::io::SeekFrom::Start(0))?;
    let mut writer = BufWriter::new(&mut *locked);
    for line in lines {
        write!(writer, "{}", line)?;
    }
    Ok(v)
}

pub fn rewrite_config_string(
    config: String,
    with: impl for<'a> FnOnce(SectionsMut) -> Result<(), Error>,
) -> Result<String, Error> {
    use std::fmt::Write;

    let mut lines = Vec::new();
    let arena = Arena::new();
    for line in arena.alloc(config).lines() {
        lines.push(Line::parse(line)?);
    }
    with(SectionsMut {
        lines: &mut lines,
        index: 0,
        arena: &arena,
        section_start: None,
        retain: true,
    })?;
    let mut writer = String::new();
    for line in lines {
        write!(writer, "{}", line)?;
    }
    Ok(writer)
}

pub type Lines<'a> = Vec<Line<'a>>;
pub type Arena = typed_arena::Arena<String>;

pub struct Sections<'a> {
    lines: &'a Lines<'a>,
    index: usize,
    started: bool,
}

impl<'a> Sections<'a> {
    pub fn ty(&self) -> Cow<str> {
        if !self.started {
            panic!("call step at least once");
        }
        if let Line::Section { ty, .. } = &self.lines[self.index] {
            return ty.as_str();
        }
        panic!("section ctx not at a section")
    }

    pub fn name(&self) -> Option<Cow<str>> {
        if !self.started {
            panic!("call step at least once");
        }
        if let Line::Section { name, .. } = &self.lines[self.index] {
            return name.as_ref().map(|n| n.as_str());
        }
        panic!("section ctx not at a section")
    }

    pub fn get<S: UciSection<'a>>(&self) -> Result<S, Error> {
        if !self.started {
            panic!("call step at least once");
        }
        S::read(self.lines, self.index)
    }

    pub fn step(&mut self) -> bool {
        if self.started {
            self.index += 1;
        }

        self.started = true;
        while let Some(line) = self.lines.get(self.index) {
            match line {
                Line::Section { .. } => {
                    return true;
                }
                _ => {
                    self.index += 1;
                    continue;
                }
            };
        }

        // Got to the end
        self.started = false;
        false
    }
}

pub struct SectionsMut<'l, 'a> {
    lines: &'l mut Lines<'a>,
    index: usize,
    arena: &'a Arena,
    section_start: Option<usize>,
    retain: bool,
}

impl<'a> SectionsMut<'_, 'a> {
    pub fn ty(&self) -> Cow<str> {
        if self.section_start.is_none() {
            panic!("call step at least once");
        }
        if let Line::Section { ty, .. } = &self.lines[self.index] {
            return ty.as_str();
        }
        panic!("section ctx not at a section")
    }

    pub fn name(&self) -> Option<Cow<str>> {
        if self.section_start.is_none() {
            panic!("call step at least once");
        }
        if let Line::Section { name, .. } = &self.lines[self.index] {
            return name.as_ref().map(|n| n.as_str());
        }
        panic!("section ctx not at a section")
    }

    pub fn get<S: UciSection<'a>>(&self) -> Result<S, Error> {
        if self.section_start.is_none() {
            panic!("call step at least once");
        }
        if self.section_start.is_none() {
            panic!("call step at least once");
        }
        S::read(self.lines, self.index)
    }

    pub fn set<S: UciSection<'a>>(&mut self, section: S) -> Result<(), Error> {
        if self.section_start.is_none() {
            panic!("call step at least once");
        }
        section.write(self.lines, self.arena, self.index)
    }

    pub fn push<S: UciSection<'a>>(
        &mut self,
        section: S,
        name: Option<impl Display>,
    ) -> Result<(), Error> {
        section.append(
            self.lines,
            self.arena,
            name.map(|n| self.arena.alloc(n.to_string()).as_str()),
        )
    }

    pub fn remove(&mut self) {
        self.set_retain(false);
    }

    pub fn set_retain(&mut self, retain: bool) {
        if self.section_start.is_none() {
            panic!("call step at least once");
        }
        self.retain = retain;
    }

    pub fn step(&mut self) -> bool {
        if let Some(first_index) = self.section_start {
            if self.retain {
                // Retain the section and move on
                self.index += 1;
            } else {
                // Remove the section
                let mut last_index = self.index;
                for i in self.index + 1..self.lines.len() {
                    if matches!(self.lines[i], Line::Section { .. }) {
                        break;
                    }
                    if self.lines[i].is_in_section() {
                        last_index = i;
                    }
                }
                self.lines.splice(first_index..=last_index, []);
                self.index = first_index;
            }
        }

        let mut first_index = self.index;
        while let Some(line) = self.lines.get(self.index) {
            match line {
                Line::Section { .. } => {
                    self.section_start = Some(first_index);
                    self.retain = true;
                    return true;
                }
                Line::Empty => {
                    self.index += 1;
                    first_index = self.index;
                    continue;
                }
                _ if line.is_in_section() => {
                    self.index += 1;
                    first_index = self.index;
                    continue;
                }
                _ => {
                    self.index += 1;
                    continue;
                }
            };
        }

        // Got to the end. If called a second time, check the same index.
        self.section_start = None;
        false
    }
}

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

pub enum Line<'a> {
    Empty,
    Comment {
        indent: bool,
        text: &'a str,
    },
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
            Line::Comment {
                indent: false,
                text,
            } => writeln!(f, "#{}", text),
            Line::Comment { indent: true, text } => writeln!(f, "\t#{}", text),
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
        if let Some(rest) = rest.strip_prefix('#') {
            return Ok(Line::Comment {
                indent: line.starts_with(char::is_whitespace),
                text: rest,
            });
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

    pub fn is_in_section(&self) -> bool {
        matches!(
            self,
            Line::Comment { indent: true, .. } | Line::Option { .. } | Line::List { .. }
        )
    }
}

#[derive(Inpt, Clone, Copy)]
pub enum Token<'a> {
    Q(Quoted<&'a str>),
    Sq(SingleQuoted<&'a str>),
    W(Spaced<&'a str>),
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
            Token::W(Spaced { inner: s })
        }
    }

    pub fn from_str(s: &'a str, arena: &'a Arena) -> Self {
        if s.contains(|c: char| c.is_whitespace()) {
            let q = arena.alloc(format!("{:?}", s));
            Token::Q(Quoted {
                inner: &q[1..q.len() - 1],
            })
        } else {
            Token::W(Spaced { inner: s })
        }
    }
}

impl PartialEq<str> for Token<'_> {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Q(quoted) => write!(f, "\"{}\"", quoted.inner),
            Token::Sq(single_quoted) => write!(f, "'{}'", single_quoted.inner),
            Token::W(word) => write!(f, "{}", word.inner),
        }
    }
}

extern crate self as uciedit; // for proc-macros in the tests below

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

    let parsed: Bar = parse_config_string(original, |mut ctx| {
        assert!(ctx.step());
        ctx.get()
    })
    .unwrap();

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

    let edited = rewrite_config_string(original.to_string(), |mut ctx| {
        ctx.push(
            Bar {
                always: 0,
                yes: Some(1),
                no: None,
                many: vec![2, 3, 4],
            },
            Some("appended"),
        )
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
config other
    option something here
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

    # ignored comment
    option ignored 6
    option yes 1
    list many 3
    list many 4

# bottom comment
config other
    option something here
";

    #[derive(UciSection)]
    struct Bar {
        always: i32,
        yes: Option<i32>,
        no: Option<i32>,
        many: Vec<i32>,
        few: Vec<i32>,
    }

    let edited = rewrite_config_string(original.to_string(), |mut ctx| {
        while ctx.step() {
            let _ = ctx.set(Bar {
                always: 0,
                yes: Some(1),
                no: None,
                many: vec![2, 3, 4],
                few: vec![5],
            });
        }
        Ok(())
    })
    .unwrap();

    println!("===Original==={original}===Edited==={edited}===Expected==={expected}=====");
    assert_eq!(edited.replace("\t", "    "), expected);
}

#[test]
fn test_remove_sections() {
    let original = r"
# section 1
config retain
    option foo bar
    # comment 1
    
# section 2
config remove
    option foo bar
    # comment 2

# section 3
config remove
    option foo bar
    # comment 3

# section 4
config retain
    option foo bar
    # comment 4

# section 5
config remove
config retain
";

    let expected = r"
# section 1
config retain
    option foo bar
    # comment 1



# section 4
config retain
    option foo bar
    # comment 4

config retain
";

    let edited = rewrite_config_string(original.to_string(), |mut ctx| {
        while ctx.step() {
            ctx.set_retain(ctx.ty() == "retain");
        }
        Ok(())
    })
    .unwrap();

    println!("===Original==={original}===Edited==={edited}===Expected==={expected}=====");
    assert_eq!(edited.replace("\t", "    "), expected);
}
