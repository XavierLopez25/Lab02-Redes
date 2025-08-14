package app;

import java.io.BufferedReader;
import java.io.IOException;
import java.io.InputStreamReader; 

import correction.Hamming;
import detection.CRC32;

public class Main {
    private static String readLine(String prompt) throws IOException {
        System.out.print(prompt);
        BufferedReader br = new BufferedReader(new InputStreamReader(System.in));
        return br.readLine().trim();
    }
    private static boolean isBinary(String s) {
        for (int i = 0; i < s.length(); i++) {
            char c = s.charAt(i);
            if (c != '0' && c != '1') return false;
        }
        return true;
    }
    public static void main(String[] args) throws Exception {
        System.out.println("=== EMISOR de Capa de Enlace ===");
        System.out.println("Algoritmos disponibles:");
        System.out.println("  1) Hamming (codificacion de errores)");
        System.out.println("  2) CRC-32 (deteccion de errores)");
        String choice = readLine("Selecciona algoritmo [1/2]: ");

        String msg = readLine("Ingresa el mensaje en binario (solo 0/1): ");
        if (msg.isEmpty() || !isBinary(msg)) {
            System.err.println("Error: el mensaje debe contener solo '0' y '1' y no estar vacío.");
            System.exit(1);
        }

        switch (choice) {
            case "1": {
                String nStr = readLine("Hamming: especifica el tamano de bloque (n) del codigo (p.ej., 7, 12, 15): ");
                int n;
                try {
                    n = Integer.parseInt(nStr);
                } catch (NumberFormatException e) {
                    System.err.println("Valor de n inválido.");
                    return;
                }
                if (n < 3) {
                    System.err.println("n debe ser >= 3");
                    return;
                }
                Hamming.EncodeResult res = Hamming.encodeStream(msg, n);
                if (res.paddingZeros > 0) {
                    System.out.println("Nota: se aplicó padding de " + res.paddingZeros + " cero(s) al último bloque de datos.");
                }
                System.out.println("Trama codificada Hamming: " + res.encodedBits);
                break;
            }
            case "2": {
                String codeword = CRC32.generateCRC32Poly(msg);
                System.out.println("Trama con CRC-32 (puro): " + codeword);
                break;
            }
            default:
                System.err.println("Opción inválida.");
        }
    }
}
