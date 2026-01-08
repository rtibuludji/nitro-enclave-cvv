use des::Des;
use des::TdesEde2;
use des::TdesEde3;
use des::cipher::{BlockEncrypt, BlockDecrypt, KeyInit};

#[derive(Clone)]
enum DesKey {
    Single([u8; 8]),
    Double([u8; 16]),
    Triple([u8; 24]),
}

impl DesKey {
    fn from_hex(key_hex: &str) -> Result<Self, String> {
        let key_bytes = hex::decode(key_hex)
            .map_err(|e| format!("Failed to decode key: {}", e))?;
        
        match key_bytes.len() {
            8 => {
                let k: [u8; 8] = key_bytes.try_into()
                    .map_err(|_| "Failed to convert key to 8-byte array".to_string())?;
                Ok(DesKey::Single(k))
            }
            16 => {
                let k: [u8; 16] = key_bytes.try_into()
                    .map_err(|_| "Failed to convert key to 16-byte array".to_string())?;
                Ok(DesKey::Double(k))
            }
            24 => {
                let k: [u8; 24] = key_bytes.try_into()
                    .map_err(|_| "Failed to convert key to 24-byte array".to_string())?;
                Ok(DesKey::Triple(k))
            }
            _ => {
                Err(format!(
                    "Invalid key length: {} bytes. Expected 8, 16, or 24 bytes (16, 32, or 48 hex characters)",
                    key_bytes.len()
                ))
            }
        }
    }

    fn encrypt(&self, block: &[u8; 8]) -> [u8; 8] {
        match self {
            DesKey::Single(key) => {
                let cipher = Des::new(key.into());
                let mut block_array = des::cipher::generic_array::GenericArray::clone_from_slice(block);
                cipher.encrypt_block(&mut block_array);
                let mut output = [0u8; 8];
                output.copy_from_slice(&block_array);
                output
            }
            DesKey::Double(key) => {
                let cipher = TdesEde2::new(key.into());
                let mut block_array = des::cipher::generic_array::GenericArray::clone_from_slice(block);
                cipher.encrypt_block(&mut block_array);
                let mut output = [0u8; 8];
                output.copy_from_slice(&block_array);
                output
            }
            DesKey::Triple(key) => {
                let cipher = TdesEde3::new(key.into());
                let mut block_array = des::cipher::generic_array::GenericArray::clone_from_slice(block);
                cipher.encrypt_block(&mut block_array);
                let mut output = [0u8; 8];
                output.copy_from_slice(&block_array);
                output
            }
        }
    }

    fn decrypt(&self, block: &[u8; 8]) -> [u8; 8] {
        match self {
            DesKey::Single(key) => {
                let cipher = Des::new(key.into());
                let mut block_array = des::cipher::generic_array::GenericArray::clone_from_slice(block);
                cipher.decrypt_block(&mut block_array);
                let mut output = [0u8; 8];
                output.copy_from_slice(&block_array);
                output
            }
            DesKey::Double(key) => {
                let cipher = TdesEde2::new(key.into());
                let mut block_array = des::cipher::generic_array::GenericArray::clone_from_slice(block);
                cipher.decrypt_block(&mut block_array);
                let mut output = [0u8; 8];
                output.copy_from_slice(&block_array);
                output
            }
            DesKey::Triple(key) => {
                let cipher = TdesEde3::new(key.into());
                let mut block_array = des::cipher::generic_array::GenericArray::clone_from_slice(block);
                cipher.decrypt_block(&mut block_array);
                let mut output = [0u8; 8];
                output.copy_from_slice(&block_array);
                output
            }
        }
    }
}

pub struct Cvv {
    cvk_a: DesKey,
    cvk_b: DesKey
}

impl Cvv {
    
    pub fn new(cvk_a: &str, cvk_b: &str) -> Result<Self, String> {
        let cvk_a = DesKey::from_hex(cvk_a).map_err(|e| format!("cvk a: {}", e))?;
        let cvk_b = DesKey::from_hex(cvk_b).map_err(|e| format!("cvk b: {}", e))?;

        Ok(Cvv {cvk_a, cvk_b})
    }

    pub fn calculate(&self, pan: &str, expired_date: &str, service_code: &str) -> Result<String, String> {
        let str   = format!("{}{}{}", pan, expired_date, service_code);
        let block = if str.len() >= 32 {
            str
        }
        else {
            let len    = 32 - str.len();
            let zeroes = "0".repeat(len);

            format!("{}{}", str, zeroes)
        };

        let block1 = block.chars().take(16).collect::<String>();
        let block2 = block.chars().skip(16).take(16).collect::<String>();

        let block1_bytes: [u8; 8] = hex::decode(block1)
            .map_err(|e| format!("Failed to decode block1: {}", e))?
            .try_into()
            .map_err(|_| "Block1 must be exactly 8 bytes".to_string())?;
        
        let block2_bytes: [u8; 8] = hex::decode(block2)
            .map_err(|e| format!("Failed to decode block2: {}", e))?
            .try_into()
            .map_err(|_| "Block2 must be exactly 8 bytes".to_string())?;

        let e1 = self.cvk_a.encrypt(&block1_bytes);
        let m1: [u8; 8] = {
            let mut result = [0u8; 8];
            for i in 0..8 {
                result[i] = e1[i] ^ block2_bytes[i];
            }
            result
        };

        let e2 = self.cvk_a.encrypt(&m1);
        let e3 = self.cvk_b.decrypt(&e2);
        let ef = self.cvk_a.encrypt(&e3);

        let cvv = {
            let mut digits = String::with_capacity(16);
            let mut hex_digits = String::with_capacity(16);
            
            for &byte in &ef {
                let high = (byte & 0xF0) >> 4;
                let low = byte & 0x0F;

                if high <= 9 {
                    digits.push((high + b'0') as char);
                } 
                else {
                    hex_digits.push((high - 10 + b'0') as char);
                }

                if low <= 9 {
                    digits.push((low + b'0') as char);
                } 
                else {
                    hex_digits.push((low - 10 + b'0') as char);
                }
            }
            
            digits + &hex_digits
        };

        Ok(cvv[..3].to_string())        
    }

    pub fn verify(&self, pan: &str, expired_date: &str, service_code: &str, cvv: &str) -> Result<bool, String> {

        let cvv_calc = self.calculate(pan, expired_date, service_code)
            .map_err(|e| format!("Failed to calculate cvv: {}", e))?;

        let result   = cvv_calc == cvv;
        Ok(result)
    }

}
