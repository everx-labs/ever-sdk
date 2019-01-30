use super::common::prepend_data;
use super::ABIParameter;
use tonlabs_sdk_emulator::bitstring::{Bit, Bitstring};
use tonlabs_sdk_emulator::stack::BuilderData;

impl ABIParameter for bool {
    fn prepend_to(&self, destination: BuilderData) -> BuilderData {
        let mut destination = {
            if 1 + destination.bits_used() > destination.bits_capacity() {
                let mut next = BuilderData::new();
                next.append_reference(destination);
                next
            } else {
                destination
            }
        };
        prepend_data(
            &mut destination,
            Bitstring::new().append_bit(&{
                if *self {
                    Bit::One
                } else {
                    Bit::Zero
                }
            }),
        );
        destination
    }

    fn type_signature() -> String {
        "bool".to_string()
    }
}
