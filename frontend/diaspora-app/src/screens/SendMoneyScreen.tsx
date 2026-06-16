import React, { useState, useEffect } from 'react';
import {
  View,
  Text,
  TextInput,
  StyleSheet,
  TouchableOpacity,
  ScrollView,
  ActivityIndicator,
  Alert,
} from 'react-native';
import { api, RemittanceRule } from '../services/api';

const DEFAULT_SPLIT_BPS = 3000;

interface Props {
  onBack: () => void;
  onSendComplete?: (result: { payout_amount: number; savings_amount: number; fee_amount: number }) => void;
}

export default function SendMoneyScreen({ onBack, onSendComplete }: Props) {
  const [beneficiary, setBeneficiary] = useState('');
  const [amount, setAmount] = useState('');
  const [splitBps, setSplitBps] = useState(DEFAULT_SPLIT_BPS);
  const [rules, setRules] = useState<RemittanceRule[]>([]);
  const [selectedRuleId, setSelectedRuleId] = useState<string | null>(null);
  const [sending, setSending] = useState(false);
  const [loadingRules, setLoadingRules] = useState(true);

  useEffect(() => {
    loadRules();
  }, []);

  const loadRules = async () => {
    setLoadingRules(true);
    try {
      const data = await api.listRules();
      setRules(data);
      if (data.length > 0) {
        setSelectedRuleId(data[0].id);
      }
    } catch {
      // No rules yet or API unavailable
    } finally {
      setLoadingRules(false);
    }
  };

  const numericAmount = parseFloat(amount) || 0;
  const totalCents = Math.round(numericAmount * 100);
  const feeBps = 50;
  const feeAmount = Math.round(totalCents * feeBps / 10000);
  const remaining = totalCents - feeAmount;
  const savingsAmount = Math.round(remaining * splitBps / 10000);
  const payoutAmount = remaining - savingsAmount;

  const displayAmount = (cents: number) => {
    return (cents / 100).toLocaleString('en-US', {
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    });
  };

  const createAndSend = async () => {
    if (!beneficiary.trim()) {
      Alert.alert('Missing Info', 'Please enter a beneficiary.');
      return;
    }
    if (numericAmount <= 0) {
      Alert.alert('Invalid Amount', 'Please enter a valid amount.');
      return;
    }

    setSending(true);
    try {
      let ruleId = selectedRuleId;

      if (!ruleId) {
        const rule = await api.createRule({
          beneficiary: beneficiary.trim(),
          incoming_asset: 'USDC',
          local_asset: 'NGN',
          split_type: 'Percentage',
          split_value: splitBps,
        });
        ruleId = rule.id;
      }

      const result = await api.executeRemittance({
        rule_id: ruleId,
        total_amount: totalCents,
      });

      Alert.alert(
        'Sent Successfully!',
        `Sent ${displayAmount(result.payout_amount)} to beneficiary\n` +
        `Saved ${displayAmount(result.savings_amount)}\n` +
        `Fee: ${displayAmount(result.fee_amount)}`,
      );

      onSendComplete?.(result);
      onBack();
    } catch (err: any) {
      Alert.alert('Send Failed', err.message || 'Could not complete remittance.');
    } finally {
      setSending(false);
    }
  };

  return (
    <ScrollView style={styles.container} contentContainerStyle={styles.content}>
      <TouchableOpacity onPress={onBack}>
        <Text style={styles.back}>← Back</Text>
      </TouchableOpacity>

      <Text style={styles.title}>Send Money</Text>
      <Text style={styles.subtitle}>
        Send money home with automatic savings
      </Text>

      <View style={styles.section}>
        <Text style={styles.label}>Beneficiary</Text>
        <TextInput
          style={styles.input}
          placeholder="Phone or wallet address"
          placeholderTextColor="#445566"
          value={beneficiary}
          onChangeText={setBeneficiary}
          autoFocus
        />
      </View>

      <View style={styles.section}>
        <Text style={styles.label}>Amount (USD)</Text>
        <View style={styles.amountRow}>
          <Text style={styles.currencySign}>$</Text>
          <TextInput
            style={styles.amountInput}
            placeholder="0.00"
            placeholderTextColor="#445566"
            keyboardType="decimal-pad"
            value={amount}
            onChangeText={setAmount}
          />
        </View>
      </View>

      <View style={styles.section}>
        <View style={styles.splitHeader}>
          <Text style={styles.label}>Auto-Save Split</Text>
          <Text style={styles.splitPercent}>
            Save {(splitBps / 100).toFixed(0)}%
          </Text>
        </View>
        <View style={styles.sliderContainer}>
          {[1000, 2000, 3000, 4000, 5000].map((bps) => (
            <TouchableOpacity
              key={bps}
              style={[
                styles.sliderOption,
                splitBps === bps && styles.sliderOptionActive,
              ]}
              onPress={() => setSplitBps(bps)}
            >
              <Text
                style={[
                  styles.sliderOptionText,
                  splitBps === bps && styles.sliderOptionTextActive,
                ]}
              >
                {(bps / 100).toFixed(0)}%
              </Text>
            </TouchableOpacity>
          ))}
        </View>
      </View>

      {numericAmount > 0 && (
        <View style={styles.preview}>
          <Text style={styles.previewTitle}>Split Preview</Text>
          <View style={styles.previewBar}>
            <View
              style={[
                styles.previewSegment,
                styles.previewPayout,
                { flex: payoutAmount },
              ]}
            />
            <View
              style={[
                styles.previewSegment,
                styles.previewSavings,
                { flex: savingsAmount },
              ]}
            />
            <View
              style={[
                styles.previewSegment,
                styles.previewFee,
                { flex: feeAmount },
              ]}
            />
          </View>
          <View style={styles.previewRow}>
            <View style={styles.previewItem}>
              <View style={styles.dotPayout} />
              <Text style={styles.previewLabel}>Sent</Text>
              <Text style={styles.previewValue}>
                ${displayAmount(payoutAmount)}
              </Text>
            </View>
            <View style={styles.previewItem}>
              <View style={styles.dotSavings} />
              <Text style={styles.previewLabel}>Saved</Text>
              <Text style={styles.previewValue}>
                ${displayAmount(savingsAmount)}
              </Text>
            </View>
            <View style={styles.previewItem}>
              <View style={styles.dotFee} />
              <Text style={styles.previewLabel}>Fee</Text>
              <Text style={styles.previewValue}>
                ${displayAmount(feeAmount)}
              </Text>
            </View>
          </View>
        </View>
      )}

      <TouchableOpacity
        style={[styles.sendButton, sending && styles.sendButtonDisabled]}
        onPress={createAndSend}
        disabled={sending}
      >
        {sending ? (
          <ActivityIndicator color="#FFFFFF" />
        ) : (
          <Text style={styles.sendButtonText}>Send ${displayAmount(totalCents)}</Text>
        )}
      </TouchableOpacity>
    </ScrollView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0A1628',
  },
  content: {
    paddingHorizontal: 24,
    paddingTop: 60,
    paddingBottom: 40,
  },
  back: {
    color: '#0055FF',
    fontSize: 16,
    marginBottom: 24,
  },
  title: {
    fontSize: 28,
    fontWeight: '800',
    color: '#FFFFFF',
    marginBottom: 4,
  },
  subtitle: {
    fontSize: 14,
    color: '#8899AA',
    marginBottom: 32,
    lineHeight: 20,
  },
  section: {
    marginBottom: 24,
  },
  label: {
    fontSize: 13,
    fontWeight: '600',
    color: '#8899AA',
    textTransform: 'uppercase',
    letterSpacing: 1,
    marginBottom: 8,
  },
  input: {
    backgroundColor: '#1A2A40',
    color: '#FFFFFF',
    fontSize: 16,
    paddingVertical: 14,
    paddingHorizontal: 16,
    borderRadius: 12,
  },
  amountRow: {
    flexDirection: 'row',
    alignItems: 'center',
    backgroundColor: '#1A2A40',
    borderRadius: 12,
    paddingHorizontal: 16,
  },
  currencySign: {
    fontSize: 24,
    fontWeight: '700',
    color: '#FFFFFF',
    marginRight: 8,
  },
  amountInput: {
    flex: 1,
    color: '#FFFFFF',
    fontSize: 24,
    fontWeight: '700',
    paddingVertical: 14,
  },
  splitHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 12,
  },
  splitPercent: {
    fontSize: 14,
    fontWeight: '700',
    color: '#00D4AA',
  },
  sliderContainer: {
    flexDirection: 'row',
    gap: 8,
  },
  sliderOption: {
    flex: 1,
    paddingVertical: 10,
    backgroundColor: '#1A2A40',
    borderRadius: 8,
    alignItems: 'center',
  },
  sliderOptionActive: {
    backgroundColor: '#0055FF',
  },
  sliderOptionText: {
    fontSize: 13,
    fontWeight: '600',
    color: '#667788',
  },
  sliderOptionTextActive: {
    color: '#FFFFFF',
  },
  preview: {
    backgroundColor: '#1A2A40',
    borderRadius: 12,
    padding: 20,
    marginBottom: 24,
  },
  previewTitle: {
    fontSize: 14,
    fontWeight: '600',
    color: '#FFFFFF',
    marginBottom: 16,
  },
  previewBar: {
    flexDirection: 'row',
    height: 8,
    borderRadius: 4,
    overflow: 'hidden',
    marginBottom: 16,
  },
  previewSegment: {
    height: '100%',
  },
  previewPayout: {
    backgroundColor: '#0055FF',
  },
  previewSavings: {
    backgroundColor: '#00D4AA',
  },
  previewFee: {
    backgroundColor: '#FF6B6B',
  },
  previewRow: {
    flexDirection: 'row',
    justifyContent: 'space-between',
  },
  previewItem: {
    alignItems: 'center',
  },
  dotPayout: {
    width: 8,
    height: 8,
    borderRadius: 4,
    backgroundColor: '#0055FF',
    marginBottom: 4,
  },
  dotSavings: {
    width: 8,
    height: 8,
    borderRadius: 4,
    backgroundColor: '#00D4AA',
    marginBottom: 4,
  },
  dotFee: {
    width: 8,
    height: 8,
    borderRadius: 4,
    backgroundColor: '#FF6B6B',
    marginBottom: 4,
  },
  previewLabel: {
    fontSize: 11,
    color: '#667788',
    marginBottom: 2,
  },
  previewValue: {
    fontSize: 14,
    fontWeight: '700',
    color: '#FFFFFF',
  },
  sendButton: {
    backgroundColor: '#0055FF',
    paddingVertical: 16,
    borderRadius: 12,
    alignItems: 'center',
  },
  sendButtonDisabled: {
    opacity: 0.6,
  },
  sendButtonText: {
    color: '#FFFFFF',
    fontSize: 17,
    fontWeight: '700',
  },
});
