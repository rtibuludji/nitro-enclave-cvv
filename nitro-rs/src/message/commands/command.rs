
/// VERIFY CVV
pub const CMD_VERIFYCVV_REQUEST:  [u8; 2] = *b"CY";
pub const CMD_VERIFYCVV_RESPONSE: [u8; 2] = *b"CZ";

/// GET KEY
pub const CMD_GETKEY_REQUEST:  [u8; 2] = *b"Z0";
pub const CMD_GETKEY_RESPONSE: [u8; 2] = *b"Z1";

/// RESPONSE CODE
pub const RESPONSE_SUCCESS: [u8; 2] = *b"00";
pub const RESPONSE_ERROR_NOT_FOUND: [u8; 2] = *b"01";