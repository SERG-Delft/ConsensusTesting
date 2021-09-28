import com.google.common.io.BaseEncoding;
import org.xrpl.xrpl4j.crypto.PrivateKey;
import org.xrpl.xrpl4j.crypto.PublicKey;
import org.xrpl.xrpl4j.keypairs.DefaultKeyPairService;
import org.xrpl.xrpl4j.keypairs.KeyPair;
import org.xrpl.xrpl4j.keypairs.KeyPairService;

import javax.net.ServerSocketFactory;
import javax.net.ssl.KeyManagerFactory;
import javax.net.ssl.SSLContext;
import javax.net.ssl.SSLServerSocket;
import javax.net.ssl.SSLServerSocketFactory;
import java.io.FileInputStream;
import java.io.IOException;
import java.net.ServerSocket;
import java.net.Socket;
import java.security.KeyStore;
import java.security.Security;
import java.security.spec.ECGenParameterSpec;
import java.util.Arrays;

public class SimpleSocketServer extends Thread {
    private SSLServerSocket sslServerSocket;
    private final SSLServerSocketFactory sslServerSocketFactory;
    private KeyPair keypair;
    private final int port;
    private boolean running = false;
    private String privateKey = "pnKcCFtzqdnxEujPmk2PPdMByLcRkQqJjvhQAsnWDJx4eDaFtvU";
    private String publicKey = "nHUD186B2TMCMxD29u4JycomELdzTuQAE44TFkEBjKmN4jX3XShr";

    private static final String TLS_PROTOCOL = "TLSv1.3";

    private static final String[] CIPHER_SUITES = new String[]
    {
            "TLS_AES_128_GCM_SHA256",
            "TLS_AES_256_GCM_SHA384"
    };

    public SimpleSocketServer(int port)
    {
        this.port = port;
        this.sslServerSocketFactory = (SSLServerSocketFactory) SimpleSocketServer.getServerSocketFactory();
        createKeys();
    }

    public void startServer()
    {
        try
        {
            sslServerSocket = (SSLServerSocket) sslServerSocketFactory.createServerSocket(port);
            this.start();
        }
        catch (IOException e)
        {
            e.printStackTrace();
        }
    }

    public void stopServer()
    {
        running = false;
        this.interrupt();
    }

    @Override
    public void run()
    {
        running = true;
        while( running )
        {
            try
            {
                System.out.println( "Listening for a connection" );

                // Call accept() to receive the next connection
                Socket socket = sslServerSocket.accept();

                // Pass the socket to the RequestHandler thread for processing
                RequestHandler requestHandler = new RequestHandler( socket );
                requestHandler.start();
            }
            catch (IOException e)
            {
                e.printStackTrace();
            }
        }
    }

    private void createKeys() {
        KeyPairService keyPairService = DefaultKeyPairService.getInstance();
        String seed = keyPairService.generateSeed();
        System.out.println("Generated seed: " + seed);

        keypair = keyPairService.deriveKeyPair(seed);
        System.out.println("Derived KeyPair: " + keypair);

        String message = BaseEncoding.base16().encode("test message".getBytes());
        String signature = keyPairService.sign(message, keypair.privateKey());
        System.out.println("Message signature: " + signature);

        boolean verifies = keyPairService.verify(message, signature, keypair.publicKey());
        System.out.println("Signature verified? : " + verifies);
    }

    private static ServerSocketFactory getServerSocketFactory() {
        SSLServerSocketFactory ssf = null;
        try {
            // set up key manager to do server authentication
            SSLContext ctx;
            KeyManagerFactory kmf;
            KeyStore ks;
            char[] passphrase = "passphrase".toCharArray();

            System.out.println(Arrays.toString(Security.getProviders()));
            ctx = SSLContext.getInstance(TLS_PROTOCOL);
            kmf = KeyManagerFactory.getInstance("SunX509");
            ks = KeyStore.getInstance("JKS");

//            ks.load(new FileInputStream("testkeys"), passphrase);
//            kmf.init(ks, passphrase);
            ctx.init(null, null, null);

            ssf = ctx.getServerSocketFactory();
            return ssf;
        } catch (Exception e) {
            e.printStackTrace();
        }
        return null;
    }
}
