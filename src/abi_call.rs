use crypto::digest::Digest;
use crypto::sha2::Sha256;
use std::marker::PhantomData;
use std::sync::Arc;
use tonlabs_sdk_emulator::bitstring::Bitstring;
use tonlabs_sdk_emulator::cells_serialization::BagOfCells;
use tonlabs_sdk_emulator::stack::{BuilderData, CellData, SliceData};
use types::common::prepend_data_to_chain;
use types::ABIParameter;

pub const ABI_VERSION: u8 = 0;

pub struct ABICall<TIn: ABIParameter, TOut: ABIParameter> {
    input: PhantomData<TIn>,
    output: PhantomData<TOut>,
}

impl<TIn: ABIParameter, TOut: ABIParameter> ABICall<TIn, TOut> {
    fn get_function_id(fn_name: String) -> [u8; 4] {
        let signature = fn_name + &TIn::type_signature() + &TOut::type_signature();

        println!("{}", signature);

        // Sha256 hash of signature
        let mut hasher = Sha256::new();

        hasher.input_str(&signature);

        let mut function_hash = [0 as u8; 32];
        hasher.result(&mut function_hash);

        let mut bytes = [0; 4];
        bytes.copy_from_slice(&function_hash[..4]);
        bytes
    }

    pub fn encode_function_call<T>(fn_name: T, parameters: TIn) -> Vec<u8>
    where
        T: Into<String>,
    {
        let fn_name = fn_name.into();
        let builder = BuilderData::new();
        let builder = parameters.prepend_to(builder);
        let builder = prepend_data_to_chain(builder, {
            // make prefix with ABI version and function ID
            let mut bitstring = Bitstring::new();

            bitstring.append_u8(ABI_VERSION);
            for chunk in Self::get_function_id(fn_name).iter() {
                bitstring.append_u8(*chunk);
            }
            bitstring
        });

        // serialize tree into Vec<u8>
        let root_cell = Arc::<CellData>::from(&builder);
        let root = SliceData::from(root_cell);

        let mut data = Vec::new();
        BagOfCells::with_root(root)
            .write_to(&mut data, false, 2, 2)
            .unwrap();

        data
    }
}
