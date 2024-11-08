use color_eyre::eyre::{bail, eyre, Error};
use inpt::split::{Quoted, SingleQuoted, Word};
use inpt::{inpt, inpt_step, Inpt, InptStep};
use std::io::{BufRead, BufReader, Seek, Write};
use std::{borrow::Cow, fs::File, path::Path};

pub trait VisitUci {
    fn enter_section(&mut self, ty: &str, name: Option<&str>) -> Result<(), Error>;
    fn exit_section(&mut self) -> Result<(), Error>;
    fn option(&mut self, option: &str, value: &str) -> Result<(), Error>;
    fn list(&mut self, list: &str, item: &str) -> Result<(), Error>;
    fn finish(&mut self) -> Result<(), Error>;
}

impl<V: VisitUci> VisitUci for &mut V {
    fn enter_section(&mut self, ty: &str, name: Option<&str>) -> Result<(), Error> {
        V::enter_section(self, ty, name)
    }

    fn exit_section(&mut self) -> Result<(), Error> {
        V::exit_section(self)
    }

    fn option(&mut self, option: &str, value: &str) -> Result<(), Error> {
        V::option(self, option, value)
    }

    fn list(&mut self, list: &str, item: &str) -> Result<(), Error> {
        V::list(self, list, item)
    }

    fn finish(&mut self) -> Result<(), Error> {
        V::finish(self)
    }
}

pub struct WriteUci<W: Write = Vec<u8>> {
    writer: W,
}

impl<W: Write> VisitUci for WriteUci<W> {
    fn enter_section(&mut self, ty: &str, name: Option<&str>) -> Result<(), Error> {
        match name {
            Some(name) => writeln!(self.writer, "config {ty:?} {name:?}")?,
            None => writeln!(self.writer, "config {ty:?}")?,
        }
        Ok(())
    }

    fn exit_section(&mut self) -> Result<(), Error> {
        writeln!(self.writer)?;
        Ok(())
    }

    fn option(&mut self, option: &str, value: &str) -> Result<(), Error> {
        writeln!(self.writer, "\toption {option:?} {value:?}")?;
        Ok(())
    }

    fn list(&mut self, list: &str, item: &str) -> Result<(), Error> {
        writeln!(self.writer, "\tlist {list:?} {item:?}")?;
        Ok(())
    }

    fn finish(&mut self) -> Result<(), Error> {
        Ok(())
    }
}

pub fn parse_uci(mut read: impl BufRead, visit: &mut impl VisitUci) -> Result<(), Error> {
    #[derive(Inpt)]
    enum Token<'s> {
        Q(Quoted<&'s str>),
        Sq(SingleQuoted<&'s str>),
        W(Word<&'s str>),
    }

    impl<'s> Token<'s> {
        fn as_str(&self) -> Cow<str> {
            // TODO: inpt doesn't currently do unescaping
            match self {
                Token::Q(x) => x.unescape(),
                Token::Sq(x) => x.unescape(),
                Token::W(x) => Cow::Borrowed(x.inner),
            }
        }
    }

    let mut buf = String::new();
    let mut in_section = false;
    loop {
        buf.clear();
        if read.read_line(&mut buf)? == 0 {
            break;
        }

        let line = buf.as_str().trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let rest = line;

        let InptStep {
            data: Ok(keyword),
            rest,
        } = inpt_step::<Token>(rest)
        else {
            unreachable!()
        };
        match &*keyword.as_str() {
            "config" => {
                let (ty, name): (Token, Token) =
                    inpt(rest).map_err(|err| eyre!("could not parse section: {err}"))?;
                let name: Option<Cow<'_, str>> = match name {
                    Token::W(Word { inner: "" }) => None,
                    _ => Some(ty.as_str()),
                };
                if in_section {
                    visit.exit_section()?;
                } else {
                    in_section = true;
                }
                visit.enter_section(&ty.as_str(), name.as_deref())?;
            }
            "option" => {
                let (option, value): (Token, Token) =
                    inpt(rest).map_err(|err| eyre!("could not parse option: {err}"))?;
                visit.option(&option.as_str(), &value.as_str())?;
            }
            "list" => {
                let (list, item): (Token, Token) =
                    inpt(rest).map_err(|err| eyre!("could not parse list: {err}"))?;
                visit.list(&list.as_str(), &item.as_str())?;
            }
            kw => bail!("unknown UCI keyword {kw:?}"),
        }
    }

    if in_section {
        visit.exit_section()?;
    }
    visit.finish()?;

    Ok(())
}

pub fn read_config<V: VisitUci>(path: impl AsRef<Path>, mut with: V) -> Result<V, Error> {
    let file = File::open(path)?;
    parse_uci(BufReader::new(file), &mut with)?;
    Ok(with)
}

/// TODO: async version?
/// TODO: is the visitor->output pattern worse than parsing the file into hashmaps and mutating them?
pub fn rewrite_config<'a, V: VisitUci + AsRef<WriteUci>>(
    path: impl AsRef<Path>,
    with: impl FnOnce(WriteUci) -> V,
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
    let mut v = with(WriteUci {
        writer: Vec::<u8>::new(),
    });
    parse_uci(BufReader::new(&mut *locked), &mut v)?;
    locked.set_len(0)?;
    locked.write_all(&v.as_ref().writer)?;
    Ok(v)
}
