#[derive(Debug)]
pub struct HammingStreamResult {
    pub data_bits: String,                    // concatenación de todos los bloques (solo datos)
    pub corrected_positions: Vec<(usize, usize)>, // (índice de bloque, posición corregida 1-based dentro del bloque)
}

// Devuelve true si x es potencia de dos (1, 2, 4, 8, ...)
fn is_power_of_two(x: usize) -> bool {
    x != 0 && (x & (x - 1)) == 0
}

// Calcula r tal que 2^r >= n + 1 (n = longitud del código).
// Este r corresponde al número de bits de paridad de un código de Hamming SEC (sin paridad global).
fn parity_bits_count(n: usize) -> usize {
    let mut r = 0usize;
    while (1usize << r) < (n + 1) {
        r += 1;
    }
    r
}

// Decodifica un bloque Hamming de longitud n (n = m + r).
// Retorna (datos, posición corregida) o error si el bloque es inconsistente.
fn decode_block(block: &[u8]) -> Result<(Vec<u8>, Option<usize>), String> {
    let n = block.len();
    if n < 3 {
        return Err("n demasiado pequeño".into());
    }
    let r = parity_bits_count(n);

    // Calcular síndrome
    // NOTA: El síndrome en códigos de Hamming es una secuencia de bits que se utiliza
    // para identificar y corregir errores en la transmisión de datos. La interpretación
    //  del síndrome revela la posición del bit erróneo, permitiendo su corrección. 
    let mut syndrome: usize = 0;
    for i in 0..r {
        // posición de paridad p = 2^i (1-based)
        let p = 1usize << i;
        let mut parity = 0u8;
        for pos in 1..=n {
            if (pos & p) != 0 {
                parity ^= block[pos - 1];
            }
        }
        if parity == 1 {
            syndrome |= p;
        }
    }

    // Si síndrome != 0, corregir ese bit (si está dentro de rango)
    let mut corrected_pos: Option<usize> = None;
    let mut corrected_block = block.to_vec();
    if syndrome != 0 {
        if syndrome >= 1 && syndrome <= n {
            corrected_block[syndrome - 1] ^= 1;
            corrected_pos = Some(syndrome);
        } else {
            return Err(format!("Síndrome {} fuera de rango para n={}", syndrome, n));
        }
    }

    // Extraer solo los bits de datos (omitir posiciones potencia de dos)
    let mut data = Vec::new();
    for pos in 1..=n {
        if !is_power_of_two(pos) {
            data.push(corrected_block[pos - 1]);
        }
    }

    Ok((data, corrected_pos))
}

// Decodifica una secuencia concatenada de bloques Hamming, cada uno de longitud n.
// Retorna todos los datos concatenados y las posiciones corregidas por bloque.
pub fn decode_stream(bits_str: &str, n: usize) -> Result<HammingStreamResult, String> {
    if !bits_str.chars().all(|c| c == '0' || c == '1') {
        return Err("Solo se aceptan '0' y '1'".to_string());
    }
    let bits: Vec<u8> = bits_str.chars().map(|c| if c == '1' {1} else {0}).collect();
    if bits.len() % n != 0 {
        return Err(format!("La longitud de la trama ({}) no es múltiplo de n={}.", bits.len(), n));
    }
    let num_blocks = bits.len() / n;

    let mut all_data = Vec::<u8>::new();
    let mut corrected_positions = Vec::<(usize, usize)>::new();

    for b in 0..num_blocks {
        let start = b * n;
        let end = start + n;
        let block = &bits[start..end];
        match decode_block(block) {
            Ok((data, corrected)) => {
                if let Some(pos) = corrected {
                    corrected_positions.push((b, pos));
                }
                all_data.extend_from_slice(&data);
            }
            Err(e) => {
                return Err(format!("Bloque {} inválido: {}", b + 1, e));
            }
        }
    }

    let data_bits: String = all_data.into_iter().map(|v| if v == 1 {'1'} else {'0'}).collect();
    Ok(HammingStreamResult { data_bits, corrected_positions })
}

// --------------------------------- Tests --------------------------------- 
// === Helpers de emisor para pruebas Hamming ===

// Codifica un bloque de datos (m bits) en un bloque Hamming de longitud n (m + r).
fn encode_block(data: &[u8], n: usize) -> Result<Vec<u8>, String> {
    let r = parity_bits_count(n);
    let m = n - r;
    if data.len() != m {
        return Err(format!("El bloque de datos debe tener m={} bits, recibido {}", m, data.len()));
    }
    // Colocar bits: paridad en potencias de dos, datos en el resto
    let mut block = vec![0u8; n];
    let mut di = 0usize;
    for pos in 1..=n {
        if !is_power_of_two(pos) {
            block[pos - 1] = data[di];
            di += 1;
        }
    }
    // Calcular bits de paridad
    for i in 0..r {
        let p = 1usize << i;
        let mut parity = 0u8;
        for pos in 1..=n {
            if (pos & p) != 0 {
                parity ^= block[pos - 1];
            }
        }
        block[p - 1] = parity;
    }
    Ok(block)
}

// Codifica una secuencia de datos en bloques Hamming de longitud n.
fn encode_stream(data_bits: &str, n: usize) -> Result<String, String> {
    if !data_bits.chars().all(|c| c=='0' || c=='1') {
        return Err("Solo se aceptan '0' y '1'".into());
    }
    let r = parity_bits_count(n);
    let m = n - r;
    let bits: Vec<u8> = data_bits.chars().map(|c| if c=='1' {1} else {0}).collect();
    if bits.len() % m != 0 {
        return Err(format!("La longitud de datos ({}) debe ser múltiplo de m={} para n={}", bits.len(), m, n));
    }
    let mut out = Vec::<u8>::new();
    for chunk in bits.chunks(m) {
        let block = encode_block(chunk, n)?;
        out.extend_from_slice(&block);
    }
    let s: String = out.into_iter().map(|b| if b==1 {'1'} else {'0'}).collect();
    Ok(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flip_bit(s: String, idx: usize) -> String {
        let mut v: Vec<u8> = s.chars().map(|c| if c=='1' {1} else {0}).collect();
        if idx < v.len() { v[idx] ^= 1; }
        v.into_iter().map(|b| if b==1 {'1'} else {'0'}).collect()
    }

    #[test]
    fn hamming_sin_errores_varias_longitudes() {
        let n = 7; // Hamming(7,4)
        let casos = vec![
            "1011",        // 1 bloque (m=4)
            "10110010",    // 2 bloques
            "111000111000" // 3 bloques
        ];
        for data in casos {
            let codeword = encode_stream(data, n).expect("emisor hamming");
            let res = decode_stream(&codeword, n).expect("decodificar");
            assert!(res.corrected_positions.is_empty(), "no debería corregir");
            assert_eq!(res.data_bits, data);
        }
    }

    #[test]
    fn hamming_un_error_corregible() {
        let n = 7;
        let casos = vec![
            "1011",        // 1 bloque
            "10110010",    // 2 bloques
            "111000111000" // 3 bloques
        ];
        for data in casos {
            let codeword = encode_stream(data, n).expect("emisor hamming");
            // Voltear un bit en el primer bloque (p.ej., posición 3 del stream)
            let tampered = flip_bit(codeword, 3);
            let res = decode_stream(&tampered, n).expect("decodificar");
            // Debe haber al menos una corrección
            assert!(!res.corrected_positions.is_empty(), "debería corregir 1 error");
            assert_eq!(res.data_bits, data, "datos corregidos deben coincidir");
        }
    }

    #[test]
    fn hamming_dos_errores_en_bloques_distintos() {
        let n = 7;
        // usaremos al menos 2 bloques para poder corregir 1 por bloque
        let casos = vec![
            "10110010",    // 2 bloques
            "111000111000" // 3 bloques
        ];
        for data in casos {
            let codeword = encode_stream(data, n).expect("emisor hamming");
            // Voltear un bit en el primer bloque y otro en el segundo
            let mut tampered = flip_bit(codeword.clone(), 2); // bloque 1
            tampered = flip_bit(tampered, 8);                 // bloque 2 (índices 0-based)
            let res = decode_stream(&tampered, n).expect("decodificar");
            // Debe reportar 2 correcciones (una por bloque tocado)
            assert!(res.corrected_positions.len() >= 2, "debería corregir 2 errores en bloques distintos");
            assert_eq!(res.data_bits, data, "datos corregidos deben coincidir");
        }
    }
}