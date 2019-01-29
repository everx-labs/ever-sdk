use tonlabs_sdk_emulator::stack::{
    BuilderData, 
};
use tonlabs_sdk_emulator::bitstring::{Bit, Bitstring};
use super::ABIParameter;
use super::common::prepend_data;


impl<T1, T2> ABIParameter for (T1, T2,) 
where 
    T1: ABIParameter,
    T2: ABIParameter
{

    fn prepend_to(&self, destination: BuilderData) -> BuilderData {
        let destination = self.1.prepend_to(destination);
        let destination = self.0.prepend_to(destination);
        destination
    }

    fn type_signature() -> String {
        format!(
            "({},{})",
            T1::type_signature(),
            T2::type_signature()
        )
    }
}

