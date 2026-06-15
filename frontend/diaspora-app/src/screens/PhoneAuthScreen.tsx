import React, { useState } from 'react';
import {
  View,
  Text,
  TextInput,
  StyleSheet,
  TouchableOpacity,
  Alert,
} from 'react-native';

interface Props {
  onVerified: (phone: string) => void;
  onBack: () => void;
}

export default function PhoneAuthScreen({ onVerified, onBack }: Props) {
  const [phone, setPhone] = useState('');
  const [code, setCode] = useState('');
  const [step, setStep] = useState<'phone' | 'otp'>('phone');
  const [loading, setLoading] = useState(false);

  const sendCode = async () => {
    const cleaned = phone.replace(/[^0-9+]/g, '');
    if (cleaned.length < 8) {
      Alert.alert('Invalid Phone', 'Please enter a valid phone number.');
      return;
    }
    setLoading(true);
    // Mock: simulate sending OTP
    await new Promise((r) => setTimeout(r, 1000));
    setLoading(false);
    setStep('otp');
  };

  const verifyCode = async () => {
    if (code.length < 4) {
      Alert.alert('Invalid Code', 'Please enter the verification code.');
      return;
    }
    setLoading(true);
    // Mock: simulate OTP verification — accept any 6-digit code
    await new Promise((r) => setTimeout(r, 800));
    setLoading(false);
    onVerified(phone);
  };

  return (
    <View style={styles.container}>
      <TouchableOpacity onPress={onBack}>
        <Text style={styles.back}>← Back</Text>
      </TouchableOpacity>

      <Text style={styles.title}>
        {step === 'phone' ? 'Your Phone Number' : 'Verify Code'}
      </Text>
      <Text style={styles.subtitle}>
        {step === 'phone'
          ? 'Enter your phone number to get started.'
          : `Enter the 6-digit code sent to ${phone}`}
      </Text>

      {step === 'phone' ? (
        <TextInput
          style={styles.input}
          placeholder="+234 800 000 0000"
          placeholderTextColor="#445566"
          keyboardType="phone-pad"
          value={phone}
          onChangeText={setPhone}
          autoFocus
        />
      ) : (
        <TextInput
          style={styles.input}
          placeholder="000 000"
          placeholderTextColor="#445566"
          keyboardType="number-pad"
          maxLength={6}
          value={code}
          onChangeText={setCode}
          autoFocus
        />
      )}

      <TouchableOpacity
        style={[styles.button, loading && styles.buttonDisabled]}
        onPress={step === 'phone' ? sendCode : verifyCode}
        disabled={loading}
      >
        <Text style={styles.buttonText}>
          {loading ? 'Please wait...' : step === 'phone' ? 'Send Code' : 'Verify'}
        </Text>
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
  back: {
    color: '#0055FF',
    fontSize: 16,
    marginBottom: 32,
  },
  title: {
    fontSize: 26,
    fontWeight: '700',
    color: '#FFFFFF',
    marginBottom: 8,
  },
  subtitle: {
    fontSize: 14,
    color: '#8899AA',
    marginBottom: 32,
    lineHeight: 20,
  },
  input: {
    backgroundColor: '#1A2A40',
    color: '#FFFFFF',
    fontSize: 20,
    paddingVertical: 16,
    paddingHorizontal: 20,
    borderRadius: 12,
    marginBottom: 24,
  },
  button: {
    backgroundColor: '#0055FF',
    paddingVertical: 16,
    borderRadius: 12,
    alignItems: 'center',
  },
  buttonDisabled: {
    opacity: 0.6,
  },
  buttonText: {
    color: '#FFFFFF',
    fontSize: 17,
    fontWeight: '700',
  },
});
