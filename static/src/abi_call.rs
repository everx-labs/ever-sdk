use crypto::digest::Digest;
use crypto::sha2::Sha256;
use crypto::ed25519::signature;
use std::marker::PhantomData;
use tvm::bitstring::Bitstring;
use tvm::cells_serialization::BagOfCells;
use tvm::stack::{BuilderData, SliceData};
use types::common::prepend_data_to_chain;
use types::{ABIInParameter, ABITypeSignature};

pub const ABI_VERSION: u8 = 0;

/// Empty struct for contract call serialization
pub struct ABICall<TIn: ABIInParameter + ABITypeSignature, TOut: ABIInParameter + ABITypeSignature> {
    input: PhantomData<TIn>,
    output: PhantomData<TOut>,
}


impl<TIn, TOut> ABICall<TIn, TOut> 
where
    TIn: ABIInParameter + ABITypeSignature,
    TOut: ABIInParameter + ABITypeSignature
{
    /// Computes function ID for contract function
    fn get_function_id(fn_name: String) -> [u8; 4] {
        let signature = fn_name + &TIn::type_signature() + &TOut::type_signature();

        //println!("{}", signature);

        // Sha256 hash of signature
        let mut hasher = Sha256::new();

        hasher.input_str(&signature);

        let mut function_hash = [0 as u8; 32];
        hasher.result(&mut function_hash);

        let mut bytes = [0; 4];
        bytes.copy_from_slice(&function_hash[..4]);
        //println!("{:X?}", bytes);
        bytes
    }

    /// serializes tree into Vec<u8>
    fn serialize_message(root: SliceData) -> Vec<u8> {
        let mut data = Vec::new();
        BagOfCells::with_root(root)
            .write_to(&mut data, false)
            .unwrap();

        data
    }

    /// Encodes provided function parameters into `Vec<u8>` containing ABI contract call
    pub fn encode_function_call<T>(fn_name: T, parameters: TIn) -> Vec<u8>
    where
        T: Into<String>,
    {
        Self::serialize_message(
            Self::encode_function_call_into_slice(fn_name, parameters, |builder| builder)
        )
    }

    /// Encodes provided function parameters into `Vec<u8>` containing ABI contract call
    pub fn encode_signed_function_call<T>(fn_name: T, parameters: TIn, secret_key: &[u8]) -> Vec<u8>
    where
        T: Into<String>,
    {
        Self::serialize_message(
            Self::encode_function_call_into_slice(fn_name, parameters, |mut builder| {
                builder.append_reference(BuilderData::new());
                // let bag  = BagOfCells::with_root(builder.clone().into());
                // let data = bag.get_repr_hash_by_index(0);
                // let data = signature(data.unwrap().as_slice(), secret_key);
                // let len  = data.len() * 8;
                // prepend_data_to_chain(builder, Bitstring::create(data.to_vec(), len))
                builder
            })
        )
    }

    /// Encodes provided function parameters into `SliceData` containing ABI contract call
    pub fn encode_function_call_into_slice<T, F>(fn_name: T, parameters: TIn, op: F) -> SliceData
    where
        T: Into<String>,
        F: FnOnce(BuilderData) -> BuilderData
    {
        let fn_name = fn_name.into();
        let builder = op(BuilderData::new());
        let builder = parameters.prepend_to(builder);
        let builder = prepend_data_to_chain(builder, {
            // make prefix with ABI version and function ID
            let mut vec = vec![ABI_VERSION];
            vec.extend_from_slice(&Self::get_function_id(fn_name)[..]);
            let len = vec.len() * 8;
            Bitstring::create(vec, len)
        });

        // serialize tree into Vec<u8>
        builder.into()
    }
}
