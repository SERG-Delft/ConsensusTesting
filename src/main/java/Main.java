import okhttp3.HttpUrl;
import org.xrpl.xrpl4j.client.JsonRpcClientErrorException;
import org.xrpl.xrpl4j.client.XrplClient;
import org.xrpl.xrpl4j.client.faucet.FaucetClient;
import org.xrpl.xrpl4j.client.faucet.FundAccountRequest;
import org.xrpl.xrpl4j.model.client.accounts.AccountInfoRequestParams;
import org.xrpl.xrpl4j.model.client.accounts.AccountInfoResult;
import org.xrpl.xrpl4j.wallet.DefaultWalletFactory;
import org.xrpl.xrpl4j.wallet.Wallet;
import org.xrpl.xrpl4j.wallet.WalletFactory;

public class Main {
    public static void main(String[] args) throws JsonRpcClientErrorException {
        HttpUrl rippledUrl = HttpUrl.get("http://localhost:5005/");
        XrplClient xrplClient = new XrplClient(rippledUrl);
        // Create a Wallet using a WalletFactory
        WalletFactory walletFactory = DefaultWalletFactory.getInstance();
        Wallet testWallet = walletFactory.randomWallet(true).wallet();

        // Fund the account using the testnet Faucet
        FaucetClient faucetClient = FaucetClient
                .construct(rippledUrl);
        faucetClient.fundAccount(FundAccountRequest.of(testWallet.classicAddress()));

        // Look up your Account Info
        AccountInfoRequestParams requestParams =
                AccountInfoRequestParams.of(testWallet.classicAddress());
        AccountInfoResult accountInfoResult =
                xrplClient.accountInfo(requestParams);

        // Print the result
        System.out.println(accountInfoResult);
    }
}
