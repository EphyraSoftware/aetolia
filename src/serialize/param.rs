use std::io::Write;
use crate::serialize::WriteModel;

impl WriteModel for crate::model::Param {
    fn write_model<W: Write>(&self, writer: &mut W) -> anyhow::Result<()> {
        match self {
            crate::model::Param::AltRep { uri } => {
                write!(writer, "ALTREP=\"{}\"", uri)?;
            }
            _ => {
                unimplemented!()
            }
        }

        Ok(())
    }
}
