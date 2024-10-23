# INRC Protocol

INRC is a Solana-based lending protocol that allows users to deposit SOL as collateral and mint USDC stablecoins. The protocol includes features like health factor monitoring, liquidations, and price feed integration with Pyth Oracle.

## Key Features

- Deposit SOL as collateral
- Mint USDC against collateral
- Dynamic health factor calculation using Pyth Oracle price feeds
- Liquidation mechanism for unhealthy positions
- Configurable parameters for risk management
- Fully on-chain lending and borrowing

## Technical Architecture

### Core Components

1. **Config Account**: Stores protocol parameters

   - Liquidation threshold
   - Liquidation bonus
   - Minimum health factor
   - Authority
   - Mint information

2. **Treasury Account**: Manages user positions

   - Collateral balance
   - Minted amount
   - User information
   - Health metrics

3. **Price Oracle**: Integrates with Pyth for real-time SOL/USD price feeds
