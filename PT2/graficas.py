#!/usr/bin/env python3
import argparse
from pathlib import Path
import re

import pandas as pd
import matplotlib.pyplot as plt

def coerce_bool(series):
    return series.map(lambda x: True if str(x).strip().lower() == "true" or str(x) == "1" else False)

def safe_div(num, den):
    return (num / den) if den != 0 else 0.0

def load_all_csvs(in_dir: Path):
    files = sorted(in_dir.glob("resultados_algo=*.csv"))
    if not files:
        raise SystemExit(f"No se encontraron CSV en {in_dir} con patrón resultados_algo=*.csv")

    dfs = []
    for f in files:
        df = pd.read_csv(f)
        # Normalizar columnas esperadas
        # Columnas esperadas (de tu runner): 
        # trial_idx, mensaje_original, mensaje_recibido, valido,
        # errores_corregidos, errores_corregidos_count,
        # algoritmo, hamming_n, ber, msg_bytes, bits_tx, useful_bits_delivered
        for col in ["valido", "errores_corregidos"]:
            if col in df.columns:
                df[col] = coerce_bool(df[col])
        for col in ["errores_corregidos_count", "msg_bytes", "bits_tx", "useful_bits_delivered"]:
            if col in df.columns:
                df[col] = pd.to_numeric(df[col], errors="coerce").fillna(0).astype(int)
        if "ber" in df.columns:
            df["ber"] = pd.to_numeric(df["ber"], errors="coerce")

        # Completar columnas que pueden venir vacías en CRC
        if "hamming_n" in df.columns:
            # Si hay NaN en hamming_n (por CRC), llénalo con cadena vacía
            df["hamming_n"] = df["hamming_n"].fillna("").astype(str)

        # Asegurar algoritmo en string
        if "algoritmo" in df.columns:
            df["algoritmo"] = df["algoritmo"].astype(str)

        dfs.append(df)

    big = pd.concat(dfs, ignore_index=True)
    return big

def aggregate_metrics(df: pd.DataFrame) -> pd.DataFrame:
    # Agregamos por combo: algoritmo, ber, msg_bytes, hamming_n
    grp_cols = ["algoritmo", "ber", "msg_bytes", "hamming_n"]
    # Algunas ejecuciones pueden no tener hamming_n (CRC). Lo tratamos como "" (string vacío).
    if "hamming_n" not in df.columns:
        df["hamming_n"] = ""

    # Métricas:
    # - tasa de entrega = mean(valido)
    # - tasa de corrección (Hamming) = mean(errores_corregidos)
    # - goodput = sum(useful_bits_delivered) / sum(bits_tx)
    # - trials = conteo
    agg = df.groupby(grp_cols, dropna=False).agg(
        delivery_rate=("valido", "mean"),
        correction_rate=("errores_corregidos", "mean"),
        trials=("valido", "size"),
        useful_bits=("useful_bits_delivered", "sum"),
        bits_tx=("bits_tx", "sum"),
    ).reset_index()

    agg["goodput"] = agg.apply(lambda r: safe_div(r["useful_bits"], r["bits_tx"]), axis=1)
    # Para la leyenda, queremos etiquetas limpias:
    agg["label"] = agg.apply(lambda r: "CRC-32" if r["algoritmo"].upper().startswith("CRC")
                             else f"Hamming(n={r['hamming_n']})", axis=1)
    return agg

def ensure_out(out_dir: Path):
    out_dir.mkdir(parents=True, exist_ok=True)


def plot_delivery_vs_ber(agg: pd.DataFrame, msg_bytes: int, out_dir: Path):
    sub = agg[agg["msg_bytes"] == msg_bytes].copy()
    if sub.empty:
        return
    ensure_out(out_dir)
    plt.figure()
    # Queremos una curva por "label"
    for label, df_label in sub.groupby("label"):
        df_label = df_label.sort_values("ber")
        plt.plot(df_label["ber"], df_label["delivery_rate"], marker="o", label=label)
    plt.xlabel("BER")
    plt.ylabel("Tasa de entrega")
    plt.title(f"Tasa de entrega vs BER (msg={msg_bytes} B)")
    plt.grid(True, linestyle="--", alpha=0.4)
    plt.legend()
    out_path = out_dir / f"tasa_entrega_vs_ber_msg{msg_bytes}B.png"
    plt.savefig(out_path, dpi=150, bbox_inches="tight")
    plt.close()

def plot_correction_hamming_vs_ber(agg: pd.DataFrame, msg_bytes: int, out_dir: Path):
    sub = agg[(agg["msg_bytes"] == msg_bytes) & (agg["label"].str.startswith("Hamming"))].copy()
    if sub.empty:
        return
    ensure_out(out_dir)
    plt.figure()
    for label, df_label in sub.groupby("label"):
        df_label = df_label.sort_values("ber")
        plt.plot(df_label["ber"], df_label["correction_rate"], marker="o", label=label)
    plt.xlabel("BER")
    plt.ylabel("Tasa de corrección (fracción de pruebas con corrección)")
    plt.title(f"Tasa de corrección Hamming vs BER (msg={msg_bytes} B)")
    plt.grid(True, linestyle="--", alpha=0.4)
    plt.legend()
    out_path = out_dir / f"tasa_correccion_hamming_vs_ber_msg{msg_bytes}B.png"
    plt.savefig(out_path, dpi=150, bbox_inches="tight")
    plt.close()

def plot_goodput_vs_ber(agg: pd.DataFrame, msg_bytes: int, out_dir: Path):
    sub = agg[agg["msg_bytes"] == msg_bytes].copy()
    if sub.empty:
        return
    ensure_out(out_dir)
    plt.figure()
    for label, df_label in sub.groupby("label"):
        df_label = df_label.sort_values("ber")
        plt.plot(df_label["ber"], df_label["goodput"], marker="o", label=label)
    plt.xlabel("BER")
    plt.ylabel("Goodput (bits útiles / bits transmitidos)")
    plt.title(f"Goodput vs BER (msg={msg_bytes} B)")
    plt.grid(True, linestyle="--", alpha=0.4)
    plt.legend()
    out_path = out_dir / f"goodput_vs_ber_msg{msg_bytes}B.png"
    plt.savefig(out_path, dpi=150, bbox_inches="tight")
    plt.close()

def main():
    ap = argparse.ArgumentParser(description="Graficador de resultados de laboratorio (Hamming / CRC-32).")
    ap.add_argument("--dir", default="resultados", help="Directorio con los CSV (por defecto: resultados)")
    ap.add_argument("--out", default="graficas", help="Directorio de salida para PNG (por defecto: graficas)")
    args = ap.parse_args()

    in_dir = Path(args.dir)
    out_dir = Path(args.out)

    big = load_all_csvs(in_dir)
    agg = aggregate_metrics(big)

    # Reporte de resumen por consola (útil para checar)
    resumen = agg.sort_values(["msg_bytes", "algoritmo", "hamming_n", "ber"])
    print("\n=== RESUMEN AGREGADO ===")
    cols = ["msg_bytes", "algoritmo", "hamming_n", "ber", "trials", "delivery_rate", "correction_rate", "goodput"]
    print(resumen[cols].to_string(index=False, float_format=lambda x: f"{x:.4f}"))

    # Graficar para cada tamaño de mensaje disponible
    for msg_bytes in sorted(agg["msg_bytes"].unique()):
        plot_delivery_vs_ber(agg, msg_bytes, out_dir)
        plot_correction_hamming_vs_ber(agg, msg_bytes, out_dir)
        plot_goodput_vs_ber(agg, msg_bytes, out_dir)

    print(f"\nListo. PNGs en: {out_dir.resolve()}")

if __name__ == "__main__":
    main()
