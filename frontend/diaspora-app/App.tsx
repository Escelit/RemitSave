import React, { useState } from 'react';
import { StatusBar } from 'expo-status-bar';
import { SafeAreaView, StyleSheet, View, Text, ActivityIndicator } from 'react-native';
import { AuthProvider, useAuth } from './src/context/AuthContext';
import WelcomeScreen from './src/screens/WelcomeScreen';
import PhoneAuthScreen from './src/screens/PhoneAuthScreen';
import WalletCreationScreen from './src/screens/WalletCreationScreen';

type Screen = 'loading' | 'welcome' | 'phoneAuth' | 'walletCreation' | 'home';

function OnboardingFlow() {
  const { status, onboard, createWallet, wallet } = useAuth();
  const [screen, setScreen] = useState<Screen>('loading');
  const [walletLoading, setWalletLoading] = useState(false);

  React.useEffect(() => {
    if (status === 'authenticated') {
      setScreen('home');
    } else if (status === 'unauthenticated') {
      setScreen('welcome');
    }
  }, [status]);

  if (screen === 'loading' || (status === 'authenticated' && screen === 'home')) {
    return (
      <View style={styles.centered}>
        {status === 'authenticated' && wallet ? (
          <View style={styles.homeContainer}>
            <Text style={styles.homeTitle}>RemitSave</Text>
            <Text style={styles.homeSubtitle}>Welcome back!</Text>
            <View style={styles.walletBox}>
              <Text style={styles.walletLabel}>Wallet</Text>
              <Text style={styles.walletAddress} numberOfLines={1}>
                {wallet.publicKey.slice(0, 8)}...{wallet.publicKey.slice(-4)}
              </Text>
            </View>
          </View>
        ) : (
          <ActivityIndicator color="#0055FF" size="large" />
        )}
        <StatusBar style="light" />
      </View>
    );
  }

  return (
    <>
      {screen === 'welcome' && (
        <WelcomeScreen onGetStarted={() => setScreen('phoneAuth')} />
      )}
      {screen === 'phoneAuth' && (
        <PhoneAuthScreen
          onVerified={(phone) => {
            onboard(phone);
            setScreen('walletCreation');
          }}
          onBack={() => setScreen('welcome')}
        />
      )}
      {screen === 'walletCreation' && (
        <WalletCreationScreen
          onCreate={async () => {
            setWalletLoading(true);
            await createWallet();
            setWalletLoading(false);
            setScreen('home');
          }}
          onSkip={() => setScreen('home')}
          loading={walletLoading}
        />
      )}
      <StatusBar style="light" />
    </>
  );
}

export default function App() {
  return (
    <AuthProvider>
      <SafeAreaView style={styles.container}>
        <OnboardingFlow />
      </SafeAreaView>
    </AuthProvider>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0A1628',
  },
  centered: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: '#0A1628',
  },
  homeContainer: {
    alignItems: 'center',
    paddingHorizontal: 24,
  },
  homeTitle: {
    fontSize: 32,
    fontWeight: '800',
    color: '#FFFFFF',
    letterSpacing: 1,
  },
  homeSubtitle: {
    fontSize: 16,
    color: '#8899AA',
    marginTop: 4,
    marginBottom: 32,
  },
  walletBox: {
    backgroundColor: '#1A2A40',
    borderRadius: 12,
    padding: 20,
    width: '100%',
    alignItems: 'center',
  },
  walletLabel: {
    fontSize: 12,
    fontWeight: '600',
    color: '#667788',
    textTransform: 'uppercase',
    letterSpacing: 1,
    marginBottom: 8,
  },
  walletAddress: {
    fontSize: 16,
    fontWeight: '500',
    color: '#FFFFFF',
  },
});
