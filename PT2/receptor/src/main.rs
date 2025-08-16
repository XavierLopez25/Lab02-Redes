mod detection;
mod correction;

use std::io::{BufRead, BufReader};
use std::net::{TcpListener};
use std::io::{self, Write};


fn bits_to_string_u8(bits: &str) -> Result<String, String> {
    if bits.len() % 8 != 0 { return Err(format!("Longitud no múltiplo de 8: {}", bits.len())); }
    let mut out = String::new();
    for chunk in bits.as_bytes().chunks(8) {
        let s = std::str::from_utf8(chunk).map_err(|_| "utf8".to_string())?;
        let v = u8::from_str_radix(s, 2).map_err(|_| format!("byte inválido: {}", s))?;
        out.push(v as char);
    }
    Ok(out)
}

fn parse_param_map(s: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    for part in s.split(';') {
        let p = part.trim();
        if p.is_empty() { continue; }
        if let Some((k,v)) = p.split_once('=') { map.insert(k.to_string(), v.to_string()); }
    }
    map
}

fn main() -> std::io::Result<()> {
    let addr = "0.0.0.0:9000";
    println!("=== RECEPTOR (Parte 2) – escuchando en {} ===", addr);
    io::stdout().flush().unwrap();

    let listener = TcpListener::bind(addr)?;

    for stream in listener.incoming() {
        let stream = stream?;
        let peer = stream.peer_addr().ok();
        println!("Conexión de {:?}", peer);
        io::stdout().flush().unwrap();

        let mut reader = BufReader::new(stream);

        let mut line = String::new();
        // ALGO=
        line.clear(); reader.read_line(&mut line)?; let algo = line.trim_start_matches("ALGO=").trim().to_string();
        // PARAM=
        line.clear(); reader.read_line(&mut line)?; let param_str = line.trim_start_matches("PARAM=").trim().to_string();
        // BITS=
        line.clear(); reader.read_line(&mut line)?; let bits = line.trim_start_matches("BITS=").trim().to_string();

        println!("ALGO={} | PARAM={} | bits={}… ({} bits)", algo, param_str, &bits.chars().take(32).collect::<String>(), bits.len());
        io::stdout().flush().unwrap();


        match algo.as_str() {
            "CRC32" => {
                match detection::crc32::verify_crc32_poly(&bits) {
                    Ok(ok) if ok.valid => {
                        let msg_bits = ok.original_message.unwrap();
                        match bits_to_string_u8(&msg_bits) {
                            Ok(s) => {
                                println!("CRC válido. Mensaje: {}", s);
                                io::stdout().flush().unwrap();
                            }
                            Err(e) => {
                                println!("CRC válido, pero no se pudo decodificar ASCII: {}", e);
                                io::stdout().flush().unwrap();
                            },
                        }
                    }
                    Ok(_) => {
                        println!("CRC inválido: mensaje descartado");
                        io::stdout().flush().unwrap();
                    },
                    Err(e) => {
                        println!("Error CRC: {}", e);
                        io::stdout().flush().unwrap();
                    },
                }
            }
            "HAMMING" => {
                let params = parse_param_map(&param_str);
                let n: usize = params.get("n").and_then(|v| v.parse().ok()).unwrap_or(7);
                let pad: usize = params.get("pad").and_then(|v| v.parse().ok()).unwrap_or(0);
                match correction::hamming::decode_stream(&bits, n) {
                    Ok(res) => {
                        let mut data = res.data_bits;
                        if pad > 0 && pad <= data.len() { data.truncate(data.len() - pad); }
                        match bits_to_string_u8(&data) {
                            Ok(s) => {
                                if res.corrected_positions.is_empty() {
                                    println!("Hamming: sin errores. Mensaje: {}", s);
                                    io::stdout().flush().unwrap();
                                } else {
                                    println!("Hamming: errores corregidos en {:?}. Mensaje: {}", res.corrected_positions, s);
                                    io::stdout().flush().unwrap();
                                }
                            }
                            Err(e) => {
                                println!("Hamming ok, pero no se pudo decodificar ASCII: {}", e);
                                io::stdout().flush().unwrap();
                            },
                        }
                    }
                    Err(e) => { 
                        println!("Hamming: errores no corregibles. {}", e);
                        io::stdout().flush().unwrap();
                    }
                }
            }
            other => println!("Algoritmo no soportado: {}", other),
        }
    }

    Ok(())
}
