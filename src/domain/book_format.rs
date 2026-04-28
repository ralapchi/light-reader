use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BookFormat {
    Epub,
    Txt,
    ReservedPdf,
    ReservedMobi,
}
