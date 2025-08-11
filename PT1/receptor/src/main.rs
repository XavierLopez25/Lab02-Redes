mod detection;
mod correction;

use std::io::{self, Write};

fn read_line(prompt: &str) -> String {
    print!("{}", prompt);
    let _ = io::stdout().flush();
    let mut s = String::new();
    io::stdin().read_line(&mut s).expect("failed to read line");
    s.trim().to_string()
}

fn main() {
    println!("=== RECEPTOR de Capa de Enlace ===");
    println!("Algoritmos disponibles:");
    println!("  1) Hamming (corrección de errores)");
    println!("  2) CRC-32 (detección de errores, polinomial puro)");
    let choice = read_line("Selecciona algoritmo [1/2]: ");

    let bits = read_line("Ingresa la trama en binario (solo 0/1): ");
    if !bits.chars().all(|c| c == '0' || c == '1') {
        eprintln!("Error: la trama debe contener solo caracteres '0' y '1'.");
        std::process::exit(1);
    }

    match choice.as_str() {
        "1" => {
            // Hamming
            let n_str = read_line("Hamming: especifica el tamaño de bloque (n) del código (p.ej., 7, 12, 15): ");
            let n: usize = match n_str.parse() {
                Ok(v) if v >= 3 => v,
                _ => {
                    eprintln!("Valor de n inválido.");
                    std::process::exit(1);
                }
            };

            match correction::hamming::decode_stream(&bits, n) {
                Ok(res) => {
                    if res.corrected_positions.is_empty() {
                        println!("Resultado: No se detectaron errores.");
                        println!("Mensaje original (datos sin bits de paridad): {}", res.data_bits);
                    } else {
                        println!("Resultado: Se detectaron y corrigieron errores.");
                        println!("Posiciones corregidas (posición dentro de cada bloque de longitud n, base 1):");
                        for (block_idx, pos) in res.corrected_positions {
                            println!("  Bloque {}, bit {}", block_idx + 1, pos);
                        }
                        println!("Mensaje corregido (datos sin bits de paridad): {}", res.data_bits);
                    }
                }
                Err(e) => {
                    println!("Resultado: Se detectaron errores no corregibles. Se descarta el mensaje.");
                    println!("Detalle: {}", e);
                }
            }
        }
        "2" => {
            // CRC-32
            match detection::crc32::verify_crc32_poly(&bits) {
                Ok(ok) => {
                    if ok.valid {
                        println!("Resultado: No se detectaron errores (CRC correcto).");
                        println!("Mensaje original (sin los 32 bits de CRC): {}", ok.original_message.unwrap_or_default());
                    } else {
                        println!("Resultado: Se detectaron errores: trama descartada (CRC inválido).");
                    }
                }
                Err(e) => {
                    eprintln!("Error al verificar CRC-32: {}", e);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("Opción inválida.");
            std::process::exit(1);
        }
    }
}
