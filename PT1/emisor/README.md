# EMISOR – Capa de Enlace (Java)

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
