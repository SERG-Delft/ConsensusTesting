import com.google.common.io.ByteStreams;
import org.xrpl.xrpl4j.codec.addresses.AddressCodec;
import org.xrpl.xrpl4j.codec.binary.XrplBinaryCodec;

import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.net.Socket;
import java.util.stream.Collectors;
import java.util.stream.IntStream;

public class RequestHandler extends Thread {
    private final Socket socket;
    RequestHandler(Socket socket)
    {
        this.socket = socket;
    }

    @Override
    public void run()
    {
        try
        {
            System.out.println( "Received a connection" );
            AddressCodec addressCodec = AddressCodec.getInstance();
            XrplBinaryCodec binaryCodec = new XrplBinaryCodec();

            // Get input and output streams
            BufferedReader in = new BufferedReader( new InputStreamReader( socket.getInputStream() ) );
            String line = in.readLine();

            // Echo lines back to the client until the client closes the connection or we receive an empty line
            while( line != null && line.length() > 0 )
            {
                System.out.println(line);
                line = in.readLine();
            }

            // Close our connection
            socket.close();

            System.out.println( "Connection closed" );
        }
        catch( Exception e )
        {
            e.printStackTrace();
        }
    }
}
