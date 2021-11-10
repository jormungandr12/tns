use hex;

pub fn decode_node_string_to_bytes(node: String) -> Result<Vec<u8>, hex::FromHexError> {
    hex::decode(node)
}

pub fn encode_node_bytes_to_string(node: Vec<u8>) -> String {
    hex::encode(node)
}
