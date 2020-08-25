use crate::types::OutputEncoding;

pub(crate) fn default_output_encoding_hex() -> OutputEncoding {
    OutputEncoding::Hex
}

pub(crate) fn default_output_encoding_base64() -> OutputEncoding {
    OutputEncoding::Base64
}
