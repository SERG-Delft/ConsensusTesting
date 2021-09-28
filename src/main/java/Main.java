import org.xrpl.xrpl4j.client.JsonRpcClientErrorException;
import tls.TLSRunable;

public class Main extends Thread {
    public static void main(String[] args) throws JsonRpcClientErrorException {
        int port = 51234;
        String privateKey = "pnKcCFtzqdnxEujPmk2PPdMByLcRkQqJjvhQAsnWDJx4eDaFtvU";
        String publicKey = "nHUD186B2TMCMxD29u4JycomELdzTuQAE44TFkEBjKmN4jX3XShr";
        System.out.println( "Start server on port: " + port );

//        SimpleSocketServer server = new SimpleSocketServer(port);
//        server.startServer();
        TLSRunable runnable = new TLSRunable(publicKey);
        Thread server = new Thread(runnable);
        server.start();

        // Automatically shutdown in 1 minute
        try
        {
            Thread.sleep( 60000 );
        }
        catch( Exception e )
        {
            e.printStackTrace();
        }

        server.stop();
    }
}
