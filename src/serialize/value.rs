use crate::serialize::WriteModel;
use std::io::Write;

impl WriteModel for (time::Date, time::Time, bool) {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        self.0.write_model(writer)?;
        writer.write_all(b"T")?;
        self.1.write_model(writer)?;
        if self.2 {
            writer.write_all(b"Z")?;
        }

        Ok(())
    }
}

impl WriteModel for time::Date {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        let year = self.year();
        if (0..10).contains(&year) {
            writer.write_all(&[0, 0, 0, year as u8])?;
        } else if (10..100).contains(&year) {
            writer.write_all(&[0, 0, year as u8 / 10, year as u8 % 10])?;
        } else if (100..1000).contains(&year) {
            writer.write_all(&[0, year as u8 / 100, year as u8 / 10 % 10, year as u8 % 10])?;
        } else if (1000..10000).contains(&year) {
            write!(writer, "{}", year)?;
        } else {
            return Err(anyhow::anyhow!("Year [{year}] out of range"));
        }

        match self.month() {
            m @ time::Month::October | m @ time::Month::November | m @ time::Month::December => {
                write!(writer, "{}", m as u8)?;
            }
            m => {
                write!(writer, "0{}", m as u8)?;
            }
        }

        match self.day() {
            d @ 10..=31 => {
                write!(writer, "{}", d)?;
            }
            d => {
                write!(writer, "0{}", d)?;
            }
        }

        Ok(())
    }
}

impl WriteModel for time::Time {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        match self.hour() {
            h @ 10..=23 => {
                write!(writer, "{}", h)?;
            }
            h => {
                write!(writer, "0{}", h)?;
            }
        }

        match self.minute() {
            m @ 10..=59 => {
                write!(writer, "{}", m)?;
            }
            m => {
                write!(writer, "0{}", m)?;
            }
        }

        match self.second() {
            s @ 10..=60 => {
                write!(writer, "{}", s)?;
            }
            s => {
                write!(writer, "0{}", s)?;
            }
        }

        Ok(())
    }
}
