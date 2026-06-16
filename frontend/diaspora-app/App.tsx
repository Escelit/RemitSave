import React, { useState } from 'react';
import { StatusBar } from 'expo-status-bar';
import { SafeAreaView, StyleSheet, View, Text, ActivityIndicator, TouchableOpacity } from 'react-native';
import { AuthProvider, useAuth } from './src/context/AuthContext';
import WelcomeScreen from './src/screens/WelcomeScreen';
import PhoneAuthScreen from './src/screens/PhoneAuthScreen';
import WalletCreationScreen from './src/screens/WalletCreationScreen';
import SendMoneyScreen from './src/screens/SendMoneyScreen';
import SavingsDashboardScreen from './src/screens/SavingsDashboardScreen';

type Screen = 'loading' | 'welcome' | 'phoneAuth' | 'walletCreation' | 'home' | 'sendMoney' | 'savingsDashboard';

function HomeScreen({ onSendMoney, onSavingsDashboard }: { onSendMoney: () => void; onSavingsDashboard: () => void }) {
  const { wallet } = useAuth();
  return (
    <View style={styles.homeContainer}>
      <Text style={styles.homeTitle}>RemitSave</Text>
      <Text style={styles.homeSubtitle}>Welcome back!</Text>

      {wallet && (
        <View style={styles.walletBox}>
          <Text style={styles.walletLabel}>Wallet</Text>
          <Text style={styles.walletAddress} numberOfLines={1}>
            {wallet.publicKey.slice(0, 8)}...{wallet.publicKey.slice(-4)}
          </Text>
        </View>
      )}

      <View style={styles.homeActions}>
        <TouchableOpacity style={styles.actionButton} onPress={onSendMoney}>
          <Text style={styles.actionIcon}>📤</Text>
          <Text style={styles.actionLabel}>Send Money</Text>
          <Text style={styles.actionDesc}>Send with auto-save</Text>
        </TouchableOpacity>

        <TouchableOpacity style={styles.actionButton} onPress={onSavingsDashboard}>
          <Text style={styles.actionIcon}>📊</Text>
          <Text style={styles.actionLabel}>Savings</Text>
          <Text style={styles.actionDesc}>View your goals</Text>
        </TouchableOpacity>
      </View>
    </View>
  );
}

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

  if (screen === 'loading' || (status === 'loading' && screen === 'home')) {
    return (
      <View style={styles.centered}>
        <ActivityIndicator color="#0055FF" size="large" />
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
      {screen === 'home' && (
        <HomeScreen
          onSendMoney={() => setScreen('sendMoney')}
          onSavingsDashboard={() => setScreen('savingsDashboard')}
        />
      )}
      {screen === 'sendMoney' && (
        <SendMoneyScreen
          onBack={() => setScreen('home')}
        />
      )}
      {screen === 'savingsDashboard' && (
        <SavingsDashboardScreen
          onBack={() => setScreen('home')}
          onSendMoney={() => setScreen('sendMoney')}
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
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    backgroundColor: '#0A1628',
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
    marginBottom: 32,
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
  homeActions: {
    flexDirection: 'row',
    gap: 12,
    width: '100%',
  },
  actionButton: {
    flex: 1,
    backgroundColor: '#1A2A40',
    borderRadius: 12,
    padding: 20,
    alignItems: 'center',
  },
  actionIcon: {
    fontSize: 28,
    marginBottom: 8,
  },
  actionLabel: {
    fontSize: 15,
    fontWeight: '700',
    color: '#FFFFFF',
    marginBottom: 4,
  },
  actionDesc: {
    fontSize: 11,
    color: '#667788',
  },
});
