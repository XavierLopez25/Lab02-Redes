#!/usr/bin/env python3
import subprocess
import time
import csv
import random
import string
import re
import threading
from pathlib import Path
from collections import deque

# Receptor (Rust, binario ya compilado con `cargo build`)
# Forzamos line-buffering para que Python vea cada línea al instante
RECEIVER_CMD = ["stdbuf", "-oL", "-eL", "./receptor"]
RECEIVER_CWD = "receptor/target/debug"

# Emisor Java headless (usa app.Bench)
EMITTER_CLASSPATH = "emisor/out"
EMITTER_CLASS     = "app.Bench"

# Parrilla de parámetros
BERS           = [0.0, 0.01, 0.02]
MSG_BYTES_LIST = [1, 8]
HAMMING_NS     = [7, 15]
ALGORITHMS     = ["HAMMING", "CRC32"]

N_TRIALS_PER_COMBO = 10_000
HOST = "127.0.0.1"
PORT = "9000"

# Debug
DEBUG_RX = False   # True para ver todas las líneas del receptor
PRINT_EVERY = 100  # progreso

RE_H_OK        = re.compile(r"Hamming:\s.*Mensaje:\s*([^\n]+)", re.I)
RE_H_CORR      = re.compile(r"errores corregidos", re.I)
RE_H_CORR_CNT  = re.compile(r"\(\s*\d+\s*,\s*\d+\s*\)")  # cuenta tuplas (bloque, bit)
RE_CRC_OK      = re.compile(r"CRC válido\.\s*Mensaje:\s*([^\n]+)", re.I)
RE_DROP        = re.compile(r"(CRC inválido|descartado|no corregibles)", re.I)
RE_ALGO_LINE   = re.compile(r"^ALGO=")
RE_TX_BITS     = re.compile(r"bits=(\d+)")

def parse_line_to_event(s: str):
    """
    Convierte una línea del receptor en un evento:
      - {'type': 'ALGO'}
      - {'type': 'RESULT', 'ok': True, 'msg': '...', 'corrected': bool, 'corrected_count': int}
      - {'type': 'RESULT', 'ok': False}
      - None
    """
    s = s.strip()
    if RE_ALGO_LINE.search(s):
        return {"type": "ALGO"}

    # Hamming OK
    m = RE_H_OK.search(s)
    if m:
        msg = m.group(1)
        corrected = bool(RE_H_CORR.search(s))
        corr_cnt = len(RE_H_CORR_CNT.findall(s)) if corrected else 0
        return {"type": "RESULT", "ok": True, "msg": msg, "corrected": corrected, "corrected_count": corr_cnt}

    # CRC OK
    m = RE_CRC_OK.search(s)
    if m:
        msg = m.group(1)
        return {"type": "RESULT", "ok": True, "msg": msg, "corrected": False, "corrected_count": 0}

    # Cualquier descarte
    if RE_DROP.search(s):
        return {"type": "RESULT", "ok": False, "msg": "", "corrected": False, "corrected_count": 0}

    return None

def start_receiver():
    proc = subprocess.Popen(
        RECEIVER_CMD,
        cwd=RECEIVER_CWD,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        bufsize=1
    )
    return proc

def reader_thread_fn(proc, events: deque, lock: threading.Lock):
    """Lee continuamente stdout del receptor, y encola eventos parseados."""
    while True:
        line = proc.stdout.readline()
        if not line:
            break
        s = line.rstrip("\n")
        if DEBUG_RX:
            print(f"[RX] {s}")
        ev = parse_line_to_event(s)
        if ev is not None:
            with lock:
                events.append(ev)

def wait_next_result_blocking(events: deque, lock: threading.Lock):
    """
    Bloquea hasta leer el PRÓXIMO RESULT (ok/drop).
    Devuelve: {'ok': bool, 'msg': str, 'corrected': bool, 'corrected_count': int}
    """
    while True:
        with lock:
            ev = events.popleft() if events else None
        if ev is None:
            time.sleep(0.01)
            continue
        if ev["type"] == "RESULT":
            return {
                "ok": ev.get("ok", False),
                "msg": ev.get("msg", ""),
                "corrected": ev.get("corrected", False),
                "corrected_count": ev.get("corrected_count", 0)
            }
        # si es ALGO, seguimos esperando

def random_message(n_bytes: int) -> str:
    # Mensaje solo con mayúsculas ASCII para fácil visualización
    return ''.join(random.choices(string.ascii_uppercase, k=n_bytes))

def run_one_trial(algo: str, msg: str, ber: float, n_hamming: int | None):
    """
    Ejecuta 1 envío usando app.Bench, devuelve (em_out_text, bits_tx:int)
    """
    if algo == "HAMMING":
        em_cmd = [
            "java", "-cp", EMITTER_CLASSPATH, EMITTER_CLASS,
            msg, "1", str(n_hamming), str(ber), HOST, PORT
        ]
    elif algo == "CRC32":
        em_cmd = [
            "java", "-cp", EMITTER_CLASSPATH, EMITTER_CLASS,
            msg, "2", str(ber), HOST, PORT
        ]
    else:
        raise ValueError("Algoritmo desconocido")

    em = subprocess.Popen(
        em_cmd,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True
    )
    em_out, _ = em.communicate()  # bloquea hasta terminar de enviar
    m = RE_TX_BITS.search(em_out or "")
    bits_tx = int(m.group(1)) if m else 0
    return em_out, bits_tx

def run_batch(receptor_proc, events, lock, algo: str, ber: float, msg_bytes: int, n_hamming: int | None, trials: int, out_csv: Path):
    out_csv.parent.mkdir(parents=True, exist_ok=True)
    with out_csv.open("w", newline="") as f:
        writer = csv.writer(f)
        writer.writerow([
            "trial_idx", "mensaje_original", "mensaje_recibido", "valido",
            "errores_corregidos", "errores_corregidos_count",
            "algoritmo", "hamming_n", "ber", "msg_bytes",
            "bits_tx", "useful_bits_delivered"
        ])

        for t in range(1, trials+1):
            original = random_message(msg_bytes)
            em_out, bits_tx = run_one_trial(algo, original, ber, n_hamming)

            # Espera el RESULT correspondiente
            res = wait_next_result_blocking(events, lock)
            recibido = res["msg"]
            valido   = (res["ok"] and (recibido == original))
            corr     = 1 if res["corrected"] else 0
            corr_cnt = res["corrected_count"]

            useful_bits = (8 * msg_bytes) if valido else 0

            writer.writerow([
                t, original, recibido, valido,
                corr, corr_cnt,
                algo, (n_hamming if n_hamming is not None else ""),
                ber, msg_bytes,
                bits_tx, useful_bits
            ])

            if t % PRINT_EVERY == 0 or t == trials:
                print(f"[{algo} n={n_hamming} ber={ber} msg={msg_bytes}B] "
                      f"Progreso: {t}/{trials} (últ. valido={valido}, corr={corr}, bits_tx={bits_tx})")

def main():
    print("Iniciando receptor (Rust)...")
    receptor_proc = start_receiver()

    events = deque()
    lock = threading.Lock()
    t = threading.Thread(target=reader_thread_fn, args=(receptor_proc, events, lock), daemon=True)
    t.start()

    # Darle tiempo a que imprima "escuchando"
    time.sleep(0.8)

    results_dir = Path("resultados")

    # Barrido
    for algo in ALGORITHMS:
        for msg_bytes in MSG_BYTES_LIST:
            for ber in BERS:
                if algo == "HAMMING":
                    for n in HAMMING_NS:
                        out_csv = results_dir / f"resultados_algo=Hamming_n={n}_msg={msg_bytes}B_ber={ber}.csv"
                        print(f"==> Ejecutando {algo} n={n} msg={msg_bytes}B ber={ber} -> {out_csv.name}")
                        run_batch(receptor_proc, events, lock, algo, ber, msg_bytes, n, N_TRIALS_PER_COMBO, out_csv)
                else:
                    out_csv = results_dir / f"resultados_algo=CRC32_msg={msg_bytes}B_ber={ber}.csv"
                    print(f"==> Ejecutando {algo} msg={msg_bytes}B ber={ber} -> {out_csv.name}")
                    run_batch(receptor_proc, events, lock, algo, ber, msg_bytes, None, N_TRIALS_PER_COMBO, out_csv)

    receptor_proc.terminate()
    print("Listo. CSVs en carpeta 'resultados/'")

if __name__ == "__main__":
    main()
