use sha2::{Digest, Sha256, Sha512};
use ed25519_dalek::*;
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

        hasher.input(&signature.into_bytes()[..]);

        let function_hash = hasher.result();

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
            Self::encode_function_call_into_slice(fn_name, parameters).into()
        )
    }

    /// Encodes provided function parameters into `Vec<u8>` containing ABI contract call
    pub fn encode_signed_function_call<T>(fn_name: T, parameters: TIn, pair: &Keypair) -> Vec<u8>
    where
        T: Into<String>
    {
        Self::serialize_message(
            Self::encode_signed_function_call_into_slice(fn_name, parameters, pair).into()
        )
    }

    /// Encodes provided function parameters into `BuilderData` containing ABI contract call
    pub fn encode_function_call_into_slice<T>(fn_name: T, parameters: TIn) -> BuilderData
    where
        T: Into<String>,
    {
        Self::encode_into_slice(BuilderData::new(), fn_name, parameters)
    }

    /// Encodes provided function parameters into `BuilderData` containing ABI contract call
    pub fn encode_signed_function_call_into_slice<T>(fn_name: T, parameters: TIn, pair: &Keypair) -> BuilderData
    where
        T: Into<String>,
    {
        // prepare standard message
        let mut builder = BuilderData::new();
        builder = parameters.prepend_to(builder);
        
        // if all references are used in root cell then expand cells chain with new root
        // to put signature cell reference there
        if builder.references_capacity() == builder.references_used() {
            let mut new_builder = BuilderData::new();
            new_builder.append_reference(builder);
            builder = new_builder;
        };

        builder = prepend_data_to_chain(builder, {
            // make prefix with ABI version and function ID
            let mut vec = vec![ABI_VERSION];
            vec.extend_from_slice(&Self::get_function_id(fn_name.into())[..]);
            let len = vec.len() * 8;
            Bitstring::create(vec, len)
        });

        
        let bag = BagOfCells::with_root(builder.clone().into());
        let hash = bag.get_repr_hash_by_index(0).unwrap();
        let signature = pair.sign::<Sha512>(hash.as_slice()).to_bytes().to_vec();
        let len = signature.len() * 8;
        builder.prepend_reference(BuilderData::with_raw(signature, len));
        builder
    }

    fn encode_into_slice<T>(builder: BuilderData, fn_name: T, parameters: TIn) -> BuilderData
    where
        T: Into<String>,
    {
        let builder = parameters.prepend_to(builder);
        prepend_data_to_chain(builder, {
            // make prefix with ABI version and function ID
            let mut vec = vec![ABI_VERSION];
            vec.extend_from_slice(&Self::get_function_id(fn_name.into())[..]);
            let len = vec.len() * 8;
            Bitstring::create(vec, len)
        })
    }
}
