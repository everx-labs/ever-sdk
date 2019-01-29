use tonlabs_sdk_emulator::stack::{
    BuilderData,
};
use tonlabs_sdk_emulator::bitstring::Bitstring;

pub fn prepend_data(builder: &mut BuilderData, data: &Bitstring) {
        builder.update_cell(|cell_data, _, data| {
            let mut buffer = data.clone();
            buffer.append(
                &Bitstring::from_bitstring_with_completion_tag(cell_data.clone())
            );
            cell_data.clear();
            buffer.into_bitstring_with_completion_tag(cell_data);
        }, data);
    }
