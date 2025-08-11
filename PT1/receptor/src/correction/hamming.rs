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
