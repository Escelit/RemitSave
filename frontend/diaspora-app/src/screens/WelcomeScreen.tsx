import React from 'react';
import { View, Text, StyleSheet, TouchableOpacity } from 'react-native';

interface Props {
  onGetStarted: () => void;
}

export default function WelcomeScreen({ onGetStarted }: Props) {
  return (
    <View style={styles.container}>
      <View style={styles.hero}>
        <Text style={styles.brand}>RemitSave</Text>
        <Text style={styles.tagline}>
          Send money home.{'\n'}Save automatically.
        </Text>
      </View>

      <View style={styles.features}>
        <Feature icon="📤" title="Send" desc="Cross-border remittances to Africa" />
        <Feature icon="💰" title="Save" desc="Auto-save a portion of every transfer" />
        <Feature icon="📈" title="Grow" desc="Earn yield on your savings" />
      </View>

      <TouchableOpacity style={styles.button} onPress={onGetStarted}>
        <Text style={styles.buttonText}>Get Started</Text>
      </TouchableOpacity>

      <Text style={styles.disclaimer}>
        By continuing, you agree to our Terms of Service and Privacy Policy.
      </Text>
    </View>
  );
}

function Feature({ icon, title, desc }: { icon: string; title: string; desc: string }) {
  return (
    <View style={styles.feature}>
      <Text style={styles.featureIcon}>{icon}</Text>
      <View style={styles.featureText}>
        <Text style={styles.featureTitle}>{title}</Text>
        <Text style={styles.featureDesc}>{desc}</Text>
      </View>
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
  hero: {
    alignItems: 'center',
    marginBottom: 48,
  },
  brand: {
    fontSize: 36,
    fontWeight: '800',
    color: '#FFFFFF',
    letterSpacing: 1,
  },
  tagline: {
    fontSize: 18,
    color: '#8899AA',
    textAlign: 'center',
    marginTop: 8,
    lineHeight: 26,
  },
  features: {
    marginBottom: 48,
  },
  feature: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingVertical: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#1A2A40',
  },
  featureIcon: {
    fontSize: 28,
    marginRight: 16,
  },
  featureText: {
    flex: 1,
  },
  featureTitle: {
    fontSize: 16,
    fontWeight: '600',
    color: '#FFFFFF',
  },
  featureDesc: {
    fontSize: 13,
    color: '#667788',
    marginTop: 2,
  },
  button: {
    backgroundColor: '#0055FF',
    paddingVertical: 16,
    borderRadius: 12,
    alignItems: 'center',
  },
  buttonText: {
    color: '#FFFFFF',
    fontSize: 17,
    fontWeight: '700',
  },
  disclaimer: {
    color: '#445566',
    fontSize: 11,
    textAlign: 'center',
    marginTop: 16,
  },
});
