use super::common::prepend_array;
use super::ABIParameter;
use tonlabs_sdk_emulator::stack::BuilderData;

impl<T> ABIParameter for &[T]
where
    T: ABIParameter,
{
    fn prepend_to(&self, destination: BuilderData) -> BuilderData {
        prepend_array(destination, self, true)
    }

    fn type_signature() -> String {
        format!("{}[]", T::type_signature())
    }
}
