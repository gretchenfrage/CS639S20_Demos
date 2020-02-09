
use std::fmt::{self, Display, Formatter};

pub struct Indent<'a, I: Display>(pub &'a str, pub I);

impl<'a, I: Display> Display for Indent<'a, I> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let string = format!("{}", self.1);
        let mut first = true;
        for line in string.lines() {
            if first {
                first = false;
            } else {
                f.write_str("\n")?;
            }
            f.write_str(self.0)?;
            f.write_str(line)?;
        }
        Ok(())
    }
}

pub const INFO_INDENT: &'static str = "       ";