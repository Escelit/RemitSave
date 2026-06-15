import React from 'react';
import { View, Text, StyleSheet, TouchableOpacity, ActivityIndicator } from 'react-native';

interface Props {
  onCreate: () => Promise<void>;
  onSkip: () => void;
  loading: boolean;
}

export default function WalletCreationScreen({ onCreate, onSkip, loading }: Props) {
  return (
    <View style={styles.container}>
      <Text style={styles.emoji}>🔐</Text>
      <Text style={styles.title}>Create Your Wallet</Text>
      <Text style={styles.subtitle}>
        A Stellar wallet will be generated for you. Your secret key is stored
        securely on this device — we never see it.
      </Text>

      <View style={styles.infoBox}>
        <Text style={styles.infoTitle}>What happens next?</Text>
        <Text style={styles.infoItem}>• A Stellar keypair is generated locally</Text>
        <Text style={styles.infoItem}>• Your public key is used for receiving funds</Text>
        <Text style={styles.infoItem}>• Your secret key stays encrypted on your device</Text>
        <Text style={styles.infoItem}>• You can import an existing wallet later</Text>
      </View>

      <TouchableOpacity
        style={[styles.button, loading && styles.buttonDisabled]}
        onPress={onCreate}
        disabled={loading}
      >
        {loading ? (
          <ActivityIndicator color="#FFFFFF" />
        ) : (
          <Text style={styles.buttonText}>Generate Wallet</Text>
        )}
      </TouchableOpacity>

      <TouchableOpacity onPress={onSkip}>
        <Text style={styles.skip}>I'll do this later</Text>
      </TouchableOpacity>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0A1628',
    paddingHorizontal: 24,
    justifyContent: 'center',
  },
  emoji: {
    fontSize: 48,
    textAlign: 'center',
    marginBottom: 16,
  },
  title: {
    fontSize: 26,
    fontWeight: '700',
    color: '#FFFFFF',
    textAlign: 'center',
    marginBottom: 12,
  },
  subtitle: {
    fontSize: 14,
    color: '#8899AA',
    textAlign: 'center',
    lineHeight: 20,
    marginBottom: 32,
  },
  infoBox: {
    backgroundColor: '#1A2A40',
    borderRadius: 12,
    padding: 20,
    marginBottom: 32,
  },
  infoTitle: {
    fontSize: 15,
    fontWeight: '600',
    color: '#FFFFFF',
    marginBottom: 12,
  },
  infoItem: {
    fontSize: 13,
    color: '#8899AA',
    lineHeight: 20,
    marginBottom: 4,
  },
  button: {
    backgroundColor: '#0055FF',
    paddingVertical: 16,
    borderRadius: 12,
    alignItems: 'center',
    marginBottom: 16,
  },
  buttonDisabled: {
    opacity: 0.6,
  },
  buttonText: {
    color: '#FFFFFF',
    fontSize: 17,
    fontWeight: '700',
  },
  skip: {
    color: '#667788',
    fontSize: 14,
    textAlign: 'center',
  },
});
