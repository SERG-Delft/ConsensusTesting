package tls;
/**
 * This class provides a runnable that can be used to initialize a {@link TLSServer} thread.
 * <p/>
 * Run starts the server, which will start listening to the configured IP address and port for
 * new SSL/TLS connections and serve the ones already connected to it.
 * <p/>
 * Also a stop method is provided in order to gracefully close the server and stop the thread.
 *
 * @author <a href="mailto:alex.a.karnezis@gmail.com">Alex Karnezis</a>
 */
public class TLSRunable implements Runnable {

    TLSServer server;
    String publicKey;

    public TLSRunable(String publicKey) {
        this.publicKey = publicKey;
    }

    @Override
    public void run() {
        try {
            server = new TLSServer("TLSv1.3", "localhost", 51234, publicKey);
            server.start();
        } catch (Exception e) {
            e.printStackTrace();
        }
    }

    /**
     * Should be called in order to gracefully stop the server.
     */
    public void stop() {
        server.stop();
    }

}