// CRC-32 verificación (modo polinomial puro):
// - Polinomio generador estándar CRC-32: 0x04C11DB7 (representación normal).
// Fuente: https://en.wikipedia.org/wiki/Cyclic_redundancy_check

// - Para verificación: se toma la trama completa (mensaje + 32 bits de CRC) y
//   se divide módulo 2 por G(x). Si el residuo es cero, no hay errores.

#[derive(Debug)]
pub struct CrcVerify {
    pub valid: bool,
    pub original_message: Option<String>, // presente cuando valid = true
}

fn parse_bits(s: &str) -> Result<Vec<u8>, String> {
    if !s.chars().all(|c| c == '0' || c == '1') {
        return Err("Solo se aceptan '0' y '1'".to_string());
    }
    Ok(s.chars().map(|c| if c == '1' {1u8} else {0u8}).collect())
}

fn bits_to_string(bits: &[u8]) -> String {
    bits.iter().map(|&b| if b == 1 {'1'} else {'0'}).collect()
}

// Representa el polinomio 0x04C11DB7 como bits MSB->LSB (33 bits, incluye el bit x^32)
fn crc32_poly_bits() -> Vec<u8> {
    let poly: u64 = 0x04C11DB7; // 32 bits sin el término x^32
    let mut v = Vec::with_capacity(33);
    // Agregar bit x^32 en 1
    v.push(1u8);
    // Luego 32 bits de 0x04C11DB7 desde MSB a LSB
    for i in (0..32).rev() {
        let bit = ((poly >> i) & 1) as u8;
        v.push(bit);
    }
    v
}

// División módulo-2 sobre bits MSB->LSB.
// Modifica una copia del dividendo haciendo XOR con el divisor alineado cuando el bit líder es 1.
fn mod2_divide(mut dividend: Vec<u8>, divisor: &[u8]) -> Vec<u8> {
    let n = dividend.len();
    let m = divisor.len();
    if n < m {
        return dividend; // sin suficiente longitud, residuo es el dividendo mismo
    }
    for i in 0..=(n - m) {
        if dividend[i] == 1 {
            for j in 0..m {
                dividend[i + j] ^= divisor[j];
            }
        }
    }
    // residuo: últimos m-1 bits
    dividend[(n - (m - 1))..].to_vec()
}

pub fn verify_crc32_poly(received_bits: &str) -> Result<CrcVerify, String> {
    let bits = parse_bits(received_bits)?;
    if bits.len() < 33 {
        return Err("La trama debe tener al menos 33 bits (>= 1 de datos + 32 de CRC).".into());
    }
    let divisor = crc32_poly_bits();
    let remainder = mod2_divide(bits.clone(), &divisor);

    let valid = remainder.iter().all(|&b| b == 0);
    if valid {
        // Mensaje original: quitar los últimos 32 bits (el CRC)
        let msg = bits_to_string(&bits[..(bits.len() - 32)]);
        Ok(CrcVerify { valid: true, original_message: Some(msg) })
    } else {
        Ok(CrcVerify { valid: false, original_message: None })
    }
}

// --------------------------------- Tests ---------------------------------
// === Helpers para pruebas (emulan al emisor) ===

#[cfg(test)]
mod tests {
    use super::*;

    fn flip_bit_at(s: String, idx: usize) -> String {
        let mut bytes: Vec<u8> = s.chars().map(|c| if c=='1' {1} else {0}).collect();
        if idx < bytes.len() {
            bytes[idx] ^= 1;
        }
        bits_to_string(&bytes)
    }

    #[test]
    fn crc_no_error_varias_longitudes() {
        let casos = vec![
            "1",
            "10101",
            "1110001110001",
        ];
        for msg in casos {
            let codeword = append_crc32_poly(msg).expect("emisor crc");
            let v = verify_crc32_poly(&codeword).expect("verificar");
            assert!(v.valid, "debería ser válido");
            assert_eq!(v.original_message.unwrap(), msg);
        }
    }

    #[test]
    fn crc_error_un_bit() {
        let casos = vec![
            "1",
            "10101",
            "1110001110001",
        ];
        for msg in casos {
            let codeword = append_crc32_poly(msg).expect("emisor crc");
            // voltear un bit en alguna posición (p.ej., bit 3 o el del medio)
            let idx = codeword.len()/2;
            let tampered = flip_bit_at(codeword, idx);
            let v = verify_crc32_poly(&tampered).expect("verificar");
            assert!(!v.valid, "debería ser inválido por 1 error");
        }
    }

    #[test]
    fn crc_error_dos_o_mas_bits() {
        let casos = vec![
            "1",
            "10101",
            "1110001110001",
        ];
        for msg in casos {
            let codeword = append_crc32_poly(msg).expect("emisor crc");
            // voltear dos bits en diferentes posiciones
            let mut tampered = flip_bit_at(codeword.clone(), 0);
            tampered = flip_bit_at(tampered, codeword.len()-1);
            let v = verify_crc32_poly(&tampered).expect("verificar");
            assert!(!v.valid, "debería ser inválido por 2+ errores");
        }
    }
}
