import com.google.common.primitives.Bytes;
import org.xrpl.xrpl4j.codec.addresses.Base58;

import java.util.Arrays;


public class Base58Extended {
    public static String encodeBase58Token(TokenType type, byte[] token) {
        token = Bytes.concat(new byte[]{(byte) type.value}, token);
        return Base58.encodeChecked(token);
    }

    public static byte[] decodeBase58Token(TokenType type, String s) {
        byte[] ret = Base58.decodeChecked(s);
        // Reject zero length tokens
        if (ret.length < 6) {
            System.out.println("Empty token");
            return new byte[]{};
        }

        // The type must match.
        if (ret[0] != type.value) {
            System.out.println("Wrong type");
            return new byte[]{};
        }

        // Skip the leading type byte and the trailing checksum.
        return Arrays.copyOfRange(ret, 1, ret.length - 4);
    }
}
