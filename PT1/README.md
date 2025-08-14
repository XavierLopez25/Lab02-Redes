# Laboratorio 2 - Parte 1

# Emisor:

Emisor en **Java** que toma un mensaje en binario y genera la información de integridad según el algoritmo seleccionado:

- **Corrección**: **Hamming**. Devuelve bloques codificados de longitud `n`.
- **Detección**: **CRC-32**. Devuelve `[MENSAJE][CRC32]`.

---

## Estructura

```
src/
  app/
    Main.java
  correction/
    Hamming.java
  detection/
    CRC32.java
```

---

## Compilar y ejecutar

### En Linux / WSL
```bash
javac -d out src/app/Main.java src/correction/Hamming.java src/detection/CRC32.java
java -cp out app.Main
```

### En Windows (PowerShell)
```powershell
javac -d out src\app\Main.java src\correction\Hamming.java src\detection\CRC32.java
java -cp out app.Main
```
Nota: Navegar hasta la carpeta PT1\emisor

---

## Uso

Al ejecutar se verá:

```
=== EMISOR de Capa de Enlace ===
Algoritmos disponibles:
  1) Hamming (codificación de errores)
  2) CRC-32 (detección de errores)
Selecciona algoritmo [1/2]:
```

### Opción 1 – Hamming
- Ingresa el **mensaje en binario** (solo 0/1).
- Ingresa `n` (longitud del **bloque codificado**, por ejemplo 7 para Hamming(7,4)).
- El emisor:
  - Divide el mensaje en bloques de **m = n - r** bits de datos (calcula `r` para que `2^r ≥ n + 1`).
  - **Aplica padding con ceros** si el último bloque no llena `m` bits (te indicará cuántos ceros se añadieron).
  - Coloca los datos en las posiciones **que no** son potencia de 2 y calcula las paridades en 1,2,4,… (paridad **par**).
  - Devuelve la concatenación de los **bloques codificados** (cada uno de longitud `n`).

### Opción 2 – CRC-32
- Ingresa el **mensaje en binario** (solo 0/1).
- El emisor calcula el **resto** de dividir `[MENSAJE][32 ceros]` por el polinomio estándar **100000100110000010001110110110111** y devuelve:
  ```
  [MENSAJE][CRC de 32 bits]
  ```

---

## Ejemplos rápidos

### Hamming (7,4)
```
Entrada mensaje: 1011
n: 7
Salida (1 bloque de 7 bits): <trama hamming>
```

### CRC-32 (puro)
```
Entrada mensaje: 10110010
Salida: 10110010 10110100101111001011011000010000   (sin espacios)
```

---

# Receptor:

### Para correr la parte del receptor se utilizan los siguientes comandos:

> Nota: Tener Rust instalado.

- Para correr el programa utilizar `cargo run`

- Verás el menú:

```
=== RECEPTOR de Capa de Enlace ===
Algoritmos disponibles:
  1) Hamming (corrección de errores)
  2) CRC-32 (detección de errores, polinomial puro)
Selecciona algoritmo [1/2]:
```

---

### Uso: Hamming (opción 1)

**¿Qué es `n`?** Es la longitud **del bloque codificado** (datos + paridades).
Ejemplos típicos:

- Hamming(7,4) → `n = 7`, `r = 3`, `m = 4`
- Hamming(12,8) → `n = 12`, `r = 4`, `m = 8`
- Hamming(15,11) → `n = 15`, `r = 4`, `m = 11`

**Regla clave**: la **longitud de la trama** debe ser **múltiplo de `n`** (porque se procesa por bloques).

#### Pasos

1. Elige `1` en el menú.
2. Pega la **trama codificada con Hamming** (bits `0/1`).
3. Ingresa el `n` correcto (acordado con el emisor).
4. El receptor:

   - Detecta/corrige **1 error por bloque** (SEC).
   - Devuelve **solo los bits de datos** (sin paridades).
   - Reporta posiciones corregidas (1-based dentro de cada bloque).

#### Ejemplos rápidos (Hamming(7,4), `n = 7`)

- **Sin errores (1 bloque)**
  Entrada:

  ```
  Algoritmo: 1
  Trama: 0110011
  n: 7
  ```

  Salida esperada:

  ```
  Resultado: No se detectaron errores.
  Mensaje original (datos): 1011
  ```

- **Un error (1 bloque, flip en un bit)**
  Entrada:

  ```
  Algoritmo: 1
  Trama: 0110001
  n: 7
  ```

  Salida esperada:

  ```
  Resultado: Se detectaron y corrigieron errores.
  Posiciones corregidas: Bloque 1, bit 6
  Mensaje corregido (datos): 1011
  ```

- **Dos bloques concatenados (14 bits)**
  Entrada:

  ```
  Algoritmo: 1
  Trama: 01100110000000   (sin espacios)
  n: 7
  ```

  Salida: datos concatenados de ambos bloques.

> Nota: Paridades en posiciones **potencia de 2** (1,2,4,…) y datos en las demás.

---

### Uso: CRC-32 (opción 2)

Implementa la **división polinomial “pura”** con polinomio estándar: **0x04C11DB7** (forma normal, con x^32 implícito).

- **Entrada esperada**: `[MENSAJE][CRC de 32 bits]`
- Si el **residuo = 0** → “No se detectaron errores” y te muestra el **mensaje sin CRC**.
- Si **≠ 0** → “Se detectaron errores: trama descartada”.

#### Ejemplos

- **Trama válida trivial** (mensaje=8 ceros, CRC=32 ceros):

  ```
  Algoritmo: 2
  Trama: 0000000000000000000000000000000000000000   # 8 + 32 = 40 ceros
  → Resultado: No se detectaron errores.
    Mensaje original: 00000000
  ```

- **Trama válida con mensaje “10110010” (8 bits) en modo “puro”**
  El CRC de 32 bits (puro) para `10110010` es:

  ```
  10110100101111001011011000010000
  ```

  Por lo tanto, la trama a ingresar es:

  ```
  1011001010110100101111001011011000010000
  ```

  Resultado esperado: **válido** y muestra `10110010` como mensaje.

- **Forzar error**: cambia cualquier bit de la trama válida → el receptor debe **descartar**.

---

## Correr los tests

```bash
cargo test
```

Los tests cubren los tres escenarios solicitados para cada algoritmo:

- **Sin errores**

  - Hamming: tramas válidas → devuelve datos originales.
  - CRC-32: `[mensaje][CRC correcto]` → válido y devuelve el mensaje.

- **Un error**

  - Hamming: 1 bit volteado en un bloque → **corrige**, reporta posición y entrega los datos.
  - CRC-32: 1 bit volteado en la trama → **inválido**, se descarta.

- **Dos o más errores**

  - Hamming (SEC): si hay >1 error **en el mismo bloque**, puede fallar en detectar/corregir; si hay 1 error por **bloque distinto**, corrige cada bloque.
  - CRC-32: cualquier ≥2 errores → **inválido**, se descarta.
