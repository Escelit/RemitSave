import React, { createContext, useContext, useEffect, useState, ReactNode } from 'react';
import {
  Wallet,
  generateWallet,
  loadWallet,
  saveWalletSecurely,
  fundWithFriendbot,
  saveAuthToken,
  loadAuthToken,
  clearAuthSession,
} from '../services/wallet';
import { api, setAuthToken, UserPublic } from '../services/api';

type AuthStatus = 'loading' | 'unauthenticated' | 'authenticated';

interface AuthState {
  status: AuthStatus;
  phoneNumber: string | null;
  wallet: Wallet | null;
  user: UserPublic | null;
  onboard: (identifier: string, password?: string, country?: string) => Promise<void>;
  createWallet: () => Promise<void>;
  logout: () => Promise<void>;
  skip: () => void;
}

const AuthContext = createContext<AuthState | undefined>(undefined);

export function AuthProvider({ children }: { children: ReactNode }) {
  const [status, setStatus] = useState<AuthStatus>('loading');
  const [phoneNumber, setPhoneNumber] = useState<string | null>(null);
  const [wallet, setWallet] = useState<Wallet | null>(null);
  const [user, setUser] = useState<UserPublic | null>(null);

  useEffect(() => {
    (async () => {
      try {
        const savedToken = await loadAuthToken();
        const existingWallet = await loadWallet();

        if (savedToken) {
          setAuthToken(savedToken);
          try {
            const me = await api.getMe();
            setUser(me);
            setPhoneNumber(me.phone);
          } catch {
            // Token might be expired
          }
        }

        if (existingWallet) {
          setWallet(existingWallet);
        }

        if (savedToken || existingWallet) {
          setStatus('authenticated');
        } else {
          setStatus('unauthenticated');
        }
      } catch {
        setStatus('unauthenticated');
      }
    })();
  }, []);

  const onboard = async (identifier: string, password?: string, country = 'NG') => {
    setPhoneNumber(identifier);
    const email = identifier.includes('@')
      ? identifier
      : `${identifier.replace(/[^0-9]/g, '')}@remitsave.africa`;
    const pwd = password || 'RemitSave2026!';

    try {
      // Attempt login first
      const loginRes = await api.login({ email, password: pwd });
      setAuthToken(loginRes.token);
      await saveAuthToken(loginRes.token);
      setUser(loginRes.user);
    } catch {
      // If user doesn't exist, register
      try {
        const regRes = await api.register({
          email,
          phone: identifier,
          country,
          password: pwd,
        });
        setAuthToken(regRes.token);
        await saveAuthToken(regRes.token);
        setUser(regRes.user);
      } catch (err) {
        console.warn('Backend auth registration failed (offline or server starting):', err);
      }
    }
  };

  const createWallet = async () => {
    const w = generateWallet();
    await saveWalletSecurely(w);
    setWallet(w);

    // Fund on testnet friendbot in the background so account exists on ledger
    fundWithFriendbot(w.publicKey).catch((e) =>
      console.warn('Friendbot background funding warning:', e)
    );

    // Link public key to user on backend if authenticated
    try {
      const updatedUser = await api.linkStellarAddress(w.publicKey);
      setUser(updatedUser);
    } catch (e) {
      console.warn('Could not link Stellar address to backend user:', e);
    }

    setStatus('authenticated');
  };

  const logout = async () => {
    await clearAuthSession();
    setAuthToken(null);
    setUser(null);
    setWallet(null);
    setStatus('unauthenticated');
  };

  const skip = () => {
    setStatus('authenticated');
  };

  return (
    <AuthContext.Provider
      value={{
        status,
        phoneNumber,
        wallet,
        user,
        onboard,
        createWallet,
        logout,
        skip,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth(): AuthState {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error('useAuth must be used within AuthProvider');
  return ctx;
}
