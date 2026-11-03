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

export async function fundWithFriendbot(publicKey: string): Promise<boolean> {
  const friendbotUrl = process.env.EXPO_PUBLIC_FRIENDBOT_URL || 'http://localhost:8000/friendbot';
  try {
    const res = await fetch(`${friendbotUrl}?addr=${encodeURIComponent(publicKey)}`);
    return res.ok;
  } catch (err) {
    console.warn('Friendbot funding request failed (offline or standalone):', err);
    return false;
  }
}

export async function saveAuthToken(token: string): Promise<void> {
  try {
    const SecureStore = require('expo-secure-store');
    await SecureStore.setItemAsync('remitsave_jwt_token', token);
  } catch {
    localStorage.setItem('remitsave_jwt_token', token);
  }
}

export async function loadAuthToken(): Promise<string | null> {
  try {
    const SecureStore = require('expo-secure-store');
    const token = await SecureStore.getItemAsync('remitsave_jwt_token');
    if (token) return token;
  } catch {
    const token = localStorage.getItem('remitsave_jwt_token');
    if (token) return token;
  }
  return null;
}

export async function clearAuthSession(): Promise<void> {
  try {
    const SecureStore = require('expo-secure-store');
    await SecureStore.deleteItemAsync('remitsave_jwt_token');
  } catch {
    localStorage.removeItem('remitsave_jwt_token');
  }
}
