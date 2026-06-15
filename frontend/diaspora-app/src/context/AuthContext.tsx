import React, { createContext, useContext, useEffect, useState, ReactNode } from 'react';
import { Wallet, generateWallet, loadWallet, saveWalletSecurely } from '../services/wallet';

type AuthStatus = 'loading' | 'unauthenticated' | 'authenticated';

interface AuthState {
  status: AuthStatus;
  phoneNumber: string | null;
  wallet: Wallet | null;
  onboard: (phone: string) => Promise<void>;
  createWallet: () => Promise<void>;
  skip: () => void;
}

const AuthContext = createContext<AuthState | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [status, setStatus] = useState<AuthStatus>('loading');
  const [phoneNumber, setPhoneNumber] = useState<string | null>(null);
  const [wallet, setWallet] = useState<Wallet | null>(null);

  useEffect(() => {
    (async () => {
      const existing = await loadWallet();
      if (existing) {
        setWallet(existing);
        setStatus('authenticated');
      } else {
        setStatus('unauthenticated');
      }
    })();
  }, []);

  const onboard = async (phone: string) => {
    setPhoneNumber(phone);
  };

  const createWallet = async () => {
    const w = generateWallet();
    await saveWalletSecurely(w);
    setWallet(w);
    setStatus('authenticated');
  };

  const skip = () => {
    setStatus('authenticated');
  };

  return (
    <AuthContext.Provider value={{ status, phoneNumber, wallet, onboard, createWallet, skip }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth(): AuthState {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error('useAuth must be used within AuthProvider');
  return ctx;
}
