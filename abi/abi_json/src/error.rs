use ton_abi_core::types::DeserializationError;

error_chain! {

    types {
        AbiError, AbiErrorKind, AbiResultExt, AbiResult;
    }

    foreign_links {
        Io(std::io::Error);
        BlockError(tvm::block::BlockError);
        TvmError(tvm::error::TvmError);
        SerdeError(serde_json::Error);
        TvmException(tvm::types::Exception);
    }

    errors {
        InvalidData(msg: String) {
            description("Invalid data"),
            display("Invalid data: {}", msg)
        }
        InvalidName(name: String) {
            description("Invalid name"),
            display("Invalid name: {}", name)
        }
        InvalidFunctionId(id: u32) {
            description("Invalid function id"),
            display("Invalid function id: {}", id)
        }
        DeserializationError(err: DeserializationError) {
            description("Deserialization error"),
            display("Deserialization error: {:?}", err)
        }
        NotImplemented {
            description("Not implemented"),
            display("Not implemented")
        }
        WrongParametersCount(expected: usize, provided: usize) {
            description("Wrong parameters count"),
            display("Wrong parameters count. Expected: {}, provided: {}", expected, provided)
        }
        WrongParameterType {
            description("Not implemented"),
            display("Not implemented")
        }
        WrongDataFormat(val: serde_json::Value) {
            description("Wrong data format"),
            display("Wrong data format:\n{}", val)
        }
        InvalidParameterLength(val: serde_json::Value) {
            description("Invalid parameter length"),
            display("Invalid parameter length:\n{}", val)
        }
        InvalidParameterValue(val: serde_json::Value) {
            description("Invalid parameter value"),
            display("Invalid parameter value:\n{}", val)
        }
        IncompleteDeserializationError {
            description("Incomplete deserialization error"),
            display("Incomplete deserialization error")
        }
        InvalidInputData(msg: String) {
            description("Invalid input data"),
            display("Invalid input data: {}", msg)
        }
        WrongVersion(version: u8) {
            description("Wrong version"),
            display("Wrong version: {}", version)
        }
        WrongId(id: u32) {
            description("Wrong function ID"),
            display("Wrong function ID: {}", id)
        }
    }
}
