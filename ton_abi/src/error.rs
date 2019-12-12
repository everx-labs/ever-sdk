/*
* Copyright 2018-2019 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.  You may obtain a copy of the
* License at: https://ton.dev/licenses
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use ton_types::SliceData;

error_chain! {

    types {
        AbiError, AbiErrorKind, AbiResultExt, AbiResult;
    }

    foreign_links {
        Io(std::io::Error);
        TvmError(ton_vm::error::TvmError);
        SerdeError(serde_json::Error);
        TvmException(ton_vm::types::Exception);
        TvmExceptionCode(ton_vm::types::ExceptionCode);
        TryFromIntError(std::num::TryFromIntError);
    }

    errors {
        FailureError(error: failure::Error) {
            description("Failure error"),
            display("Failure error: {}", error.to_string())
        }
        BlockError(error: ton_block::BlockError) {
            description("Block error"),
            display("Block error: {}", error.to_string())
        }
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
        DeserializationError(description: &'static str, cursor: SliceData) {
            description("Deserialization error"),
            display("Deserialization error {}: {}", description, cursor)
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
        IncompleteDeserializationError(cursor: SliceData) {
            description("Incomplete deserialization error"),
            display("Incomplete deserialization error: {}", cursor)
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
            display("Wrong function ID: {:x}", id)
        }
    }
}

impl From<failure::Error> for AbiError {
    fn from(error: failure::Error) -> Self {
        AbiErrorKind::FailureError(error).into()
    }
}
