use tinyvg_rs::TinyVg;

use crate::resource_data::ResourceData;

#[derive(Debug)]
pub struct TinyVgResource {
    pub common_data: ResourceData,
    pub tinyvg: Option<TinyVg>,
}

impl TinyVgResource {
    pub(crate) fn new(mut data: ResourceData) -> Self {
        if let Some(tinyvg_data) = data.data.as_ref() {
            let tinyvg = TinyVg::from_bytes(tinyvg_data);
            data.data = None;

            TinyVgResource {
                common_data: data,
                tinyvg: tinyvg.ok(),
            }
        } else {
            data.data = None;

            TinyVgResource {
                common_data: data,
                tinyvg: None,
            }
        }
    }
}
