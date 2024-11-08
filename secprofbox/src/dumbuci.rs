use color_eyre::eyre::{bail, Error};
use inpt::{
    inpt,
    split::{Quoted, SingleQuoted, Word},
    Inpt,
};
use std::{fmt::Write, io::BufRead, path::Path, str::SplitWhitespace};

pub trait VisitUci {
    fn section(&mut self, ty: &str, name: Option<&str>) -> Result<(), Error>;
    fn option(&mut self, key: &str, value: &str) -> Result<(), Error>;
    fn list(&mut self, key: &str, item: &str) -> Result<(), Error>;
}

impl<V: VisitUci> VisitUci for &mut V {
    fn section(&mut self, ty: &str, name: Option<&str>) -> Result<(), Error> {
        V::section(self, ty, name)
    }

    fn option(&mut self, key: &str, value: &str) -> Result<(), Error> {
        V::option(self, key, value)
    }

    fn list(&mut self, key: &str, item: &str) -> Result<(), Error> {
        V::list(self, key, item)
    }
}

pub struct WriteUci<W: Write> {
    writer: W,
}

impl<W: Write> VisitUci for WriteUci<W> {
    fn section(&mut self, ty: &str, name: Option<&str>) -> Result<(), Error> {
        match name {
            Some(name) => writeln!(self.writer, "config {ty:?} {name:?}")?,
            None => writeln!(self.writer, "config {ty:?}")?,
        }
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
}

pub fn parse_uci(mut read: impl BufRead, visit: &mut impl VisitUci) -> Result<(), Error> {
    #[derive(Inpt)]
    enum Token<'s> {
        Q(Quoted<&'s str>),
        Sq(SingleQuoted<&'s str>),
        W(Word<&'s str>),
    }

    impl<'s> Token<'s> {
        fn to_str(self) -> &'s str {
            // TODO: inpt doesn't currently do unescaping
            match self {
                Token::Q(x) => x.inner,
                Token::Sq(x) => x.inner,
                Token::W(x) => x.inner,
            }
        }
    }

    let mut buf = String::new();
    loop {
        buf.clear();
        if read.read_line(&mut buf)? == 0 {
            break Ok(());
        }

        let line = buf.as_str().trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut toks = buf.split_whitespace();
        let Ok(t) = inpt::<Token>(line) else {
            unreachable!()
        };
        match keyword {
            "config" => {}
            "option" => {}
            "list" => {}
            _ => bail!("unknown uci keyword {inst:?}"),
        }
    }
}

pub fn rewrite_config(path: impl AsRef<Path>) -> Result<(), Error> {
    use fd_lock_rs::{FdLock, LockType};
    use std::fs::File;
    let file = File::options().write(true).truncate(false).open(path)?;
    let locked = FdLock::lock(file, fd_lock_rs::LockType::Exclusive, true)?;
    locked.t
}
