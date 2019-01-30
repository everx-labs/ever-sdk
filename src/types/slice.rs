use tonlabs_sdk_emulator::stack::BuilderData;
use super::ABIParameter;
use super::common::{prepend_array};

impl<T> ABIParameter for &[T] where T: ABIParameter {
    fn prepend_to(&self, destination: BuilderData) -> BuilderData {
        prepend_array(destination, self, true)
    }

    fn type_signature() -> String {
        format!(
            "{}[]",
            T::type_signature())
    }
}