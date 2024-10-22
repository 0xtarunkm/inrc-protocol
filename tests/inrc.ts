import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { Inrc } from '../target/types/inrc';
import { PythSolanaReceiver } from '@pythnetwork/pyth-solana-receiver';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';

describe('inrc', () => {
  const provider = anchor.AnchorProvider.env();
  const wallet = provider.wallet as anchor.Wallet;

  anchor.setProvider(provider);
  const connection = provider.connection;
  const program = anchor.workspace.Inrc as Program<Inrc>;

  const pythSolanaReceiver = new PythSolanaReceiver({
    connection,
    wallet,
  });
  const SOL_USD_FEED_ID =
    '0xef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d';

  const solUsdPriceFeedAccount = pythSolanaReceiver.getPriceFeedAccountAddress(
    0,
    SOL_USD_FEED_ID
  );

  console.log(solUsdPriceFeedAccount);

  const [treasury] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from('collateral_treasury'), wallet.publicKey.toBuffer()],
    program.programId
  );

  it('Is initialized!', async () => {
    const tx = await program.methods
      .initializeConfig()
      .accounts({
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc({ commitment: 'confirmed' });

    console.log(`the log for initialize transaction is ${tx}`);
  });

  it('Deposit collateral and mint USDC', async () => {
    const amountCollateral = 1_000_000_000;
    const amountToMint = 1_000_000_000;

    const tx = await program.methods
      .depositCollateral(
        new anchor.BN(amountCollateral),
        new anchor.BN(amountToMint)
      )
      .accounts({
        depositor: wallet.publicKey,
        priceUpdate: solUsdPriceFeedAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc({ commitment: 'confirmed' });

    console.log(`the log for deposit transaction is ${tx}`);
  });

  it('Redeem collateral and burn USDC', async () => {
    const amountCollateral = 500_000_000;
    const amountToBurn = 500_000_000;

    const tx = await program.methods
      .withdrawCollateral(
        new anchor.BN(amountCollateral),
        new anchor.BN(amountToBurn)
      )
      .accounts({
        priceUpdate: solUsdPriceFeedAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc({ commitment: 'confirmed' });

    console.log(`the log for redeem transaction is ${tx}`);
  });

  it('Update config to make account unhealthy', async () => {
    const tx = await program.methods
      .updateConfig(new anchor.BN(100))
      .accounts({})
      .rpc({ commitment: 'confirmed' });

    console.log(
      `the log for update transaction to make account unhealthy is ${tx}`
    );
  });

  it('liquidate', async () => {
    const amountToBurn = 500_000_000;
    const tx = await program.methods
      .liquidate(new anchor.BN(amountToBurn))
      .accounts({
        priceUpdate: solUsdPriceFeedAccount,
        tokenProgram: TOKEN_PROGRAM_ID,
        treasury,
      })
      .rpc({ commitment: 'confirmed' });

    console.log(`The transaction signature for liquidate is ${tx}`);
  });

  it('Update config to make account healthy', async () => {
    const tx = await program.methods
      .updateConfig(new anchor.BN(1))
      .accounts({})
      .rpc({ commitment: 'confirmed' });

    console.log(
      `the log for update transaction to make account unhealthy is ${tx}`
    );
  });
});
