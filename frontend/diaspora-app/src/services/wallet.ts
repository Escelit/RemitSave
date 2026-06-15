import { Keypair } from '@stellar/stellar-sdk';

export interface Wallet {
  publicKey: string;
  secretKey: string;
}

export function generateWallet(): Wallet {
  const keypair = Keypair.random();
  return {
    publicKey: keypair.publicKey(),
    secretKey: keypair.secret(),
  };
}

export async function saveWalletSecurely(wallet: Wallet): Promise<void> {
  try {
    const SecureStore = require('expo-secure-store');
    await SecureStore.setItemAsync('remitsave_wallet_pk', wallet.publicKey);
    await SecureStore.setItemAsync('remitsave_wallet_sk', wallet.secretKey);
  } catch {
    // Fallback for environments without SecureStore (web)
    localStorage.setItem('remitsave_wallet_pk', wallet.publicKey);
    localStorage.setItem('remitsave_wallet_sk', wallet.secretKey);
  }
}

export async function loadWallet(): Promise<Wallet | null> {
  try {
    const SecureStore = require('expo-secure-store');
    const publicKey = await SecureStore.getItemAsync('remitsave_wallet_pk');
    const secretKey = await SecureStore.getItemAsync('remitsave_wallet_sk');
    if (publicKey && secretKey) {
      return { publicKey, secretKey };
    }
  } catch {
    const publicKey = localStorage.getItem('remitsave_wallet_pk');
    const secretKey = localStorage.getItem('remitsave_wallet_sk');
    if (publicKey && secretKey) {
      return { publicKey, secretKey };
    }
  }
  return null;
}
