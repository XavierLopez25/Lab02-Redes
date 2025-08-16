# Laboratorio 2 - Parte 2

Para esta parte necesitarás compilar tanto el receptor como el emisor.

## Compilar Emisor

### En Linux / WSL
```bash
javac -d out src/app/Main.java src/correction/Hamming.java src/detection/CRC32.java src/app/Bench.java
java -cp out app.Main
```

### En Windows (PowerShell)
```powershell
javac -d out src\app\Main.java src\correction\Hamming.java src\detection\CRC32.java src\app\Bench.java
java -cp out app.Main
```
Nota: Navegar hasta la carpeta PT2\emisor

## Compilar Receptor

### En Linux / WSL
```bash
cargo build
```

### En Windows (PowerShell)
```powershell
cargo build
```
Nota: Navegar hasta la carpeta PT2\receptor


## Correr las pruebas
- Para correr las pruebas necesitarás hacer un entorno virtual de python e instalar los `requirements.txt`.
- El script `pruebas.py` deberá ser ejecutado en `PT2/`.
- Esto realizará las pruebas, levantando tanto emisor como receptor, automatizando el proceso de pruebas.

## Generar gráficas

- Para generar las gráficas deberás correr `graficas.py` desde la carpeta `PT2/`.