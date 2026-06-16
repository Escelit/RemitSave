import React from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  Dimensions,
} from 'react-native';

const SCREEN_WIDTH = Dimensions.get('window').width;
const RING_SIZE = 80;
const RING_STROKE = 6;

interface Props {
  onBack: () => void;
  onSendMoney: () => void;
}

interface SavingsGoal {
  id: string;
  name: string;
  target: number;
  saved: number;
  currency: string;
  yieldEarned: number;
}

const MOCK_GOALS: SavingsGoal[] = [
  { id: '1', name: 'School Fees', target: 500000, saved: 325000, currency: 'NGN', yieldEarned: 4250 },
  { id: '2', name: 'Emergency Fund', target: 300000, saved: 180000, currency: 'NGN', yieldEarned: 2100 },
  { id: '3', name: 'Home Renovation', target: 1000000, saved: 120000, currency: 'NGN', yieldEarned: 890 },
];

const MOCK_YIELD_DATA = [320, 480, 560, 720, 910, 1250, 1480, 1620, 1810, 2100, 2450, 2850];

function ProgressRing({ progress, size, strokeWidth, color }: {
  progress: number;
  size: number;
  strokeWidth: number;
  color: string;
}) {
  const half = size / 2;
  const radius = half - strokeWidth / 2;
  const circumference = 2 * Math.PI * radius;
  const filledLength = circumference * Math.min(progress, 1);

  return (
    <View style={{ width: size, height: size, alignItems: 'center', justifyContent: 'center' }}>
      <View style={{
        width: size,
        height: size,
        borderRadius: half,
        borderWidth: strokeWidth,
        borderColor: '#1A2A40',
        position: 'absolute',
      }} />
      <View style={{
        width: size,
        height: size,
        borderRadius: half,
        borderWidth: strokeWidth,
        borderColor: 'transparent',
        borderTopColor: color,
        borderRightColor: color,
        position: 'absolute',
        transform: [{ rotate: `${-90 + (progress * 360)}deg` }],
      }} />
      <Text style={{ fontSize: size * 0.25, fontWeight: '700', color: '#FFFFFF' }}>
        {Math.round(progress * 100)}%
      </Text>
    </View>
  );
}

function formatCurrency(amount: number, currency: string) {
  return `${currency} ${amount.toLocaleString()}`;
}

export default function SavingsDashboardScreen({ onBack, onSendMoney }: Props) {
  const totalSaved = MOCK_GOALS.reduce((sum, g) => sum + g.saved, 0);
  const totalYield = MOCK_GOALS.reduce((sum, g) => sum + g.yieldEarned, 0);
  const totalTarget = MOCK_GOALS.reduce((sum, g) => sum + g.target, 0);

  const maxYield = Math.max(...MOCK_YIELD_DATA);

  return (
    <ScrollView style={styles.container} contentContainerStyle={styles.content}>
      <View style={styles.header}>
        <TouchableOpacity onPress={onBack}>
          <Text style={styles.back}>← Back</Text>
        </TouchableOpacity>
        <Text style={styles.headerTitle}>Savings Dashboard</Text>
        <View style={{ width: 60 }} />
      </View>

      <View style={styles.totalCard}>
        <Text style={styles.totalLabel}>Total Saved</Text>
        <Text style={styles.totalValue}>{formatCurrency(totalSaved, 'NGN')}</Text>
        <Text style={styles.totalYield}>
          Yield Earned: +{formatCurrency(totalYield, 'NGN')}
        </Text>
        <View style={styles.totalBar}>
          <View style={[styles.totalBarFill, { width: `${(totalSaved / totalTarget) * 100}%` }]} />
        </View>
        <Text style={styles.totalProgress}>
          {Math.round((totalSaved / totalTarget) * 100)}% of overall goal
        </Text>
      </View>

      <Text style={styles.sectionTitle}>Active Goals</Text>

      {MOCK_GOALS.map((goal) => {
        const progress = goal.target > 0 ? goal.saved / goal.target : 0;
        return (
          <View key={goal.id} style={styles.goalCard}>
            <View style={styles.goalLeft}>
              <ProgressRing
                progress={progress}
                size={RING_SIZE}
                strokeWidth={RING_STROKE}
                color="#00D4AA"
              />
            </View>
            <View style={styles.goalRight}>
              <Text style={styles.goalName}>{goal.name}</Text>
              <Text style={styles.goalAmount}>
                {formatCurrency(goal.saved, goal.currency)}
              </Text>
              <Text style={styles.goalTarget}>
                Target: {formatCurrency(goal.target, goal.currency)}
              </Text>
              <Text style={styles.goalYield}>
                Yield: +{formatCurrency(goal.yieldEarned, goal.currency)}
              </Text>
            </View>
          </View>
        );
      })}

      <View style={styles.yieldCard}>
        <Text style={styles.sectionTitle}>Yield Earned (12 Months)</Text>
        <View style={styles.chart}>
          {MOCK_YIELD_DATA.map((value, i) => {
            const height = (value / maxYield) * 120;
            return (
              <View key={i} style={styles.barWrapper}>
                <View
                  style={[
                    styles.bar,
                    {
                      height: Math.max(height, 4),
                      backgroundColor: i === MOCK_YIELD_DATA.length - 1 ? '#00D4AA' : '#0055FF',
                    },
                  ]}
                />
              </View>
            );
          })}
        </View>
        <View style={styles.chartLabels}>
          {['J', 'F', 'M', 'A', 'M', 'J', 'J', 'A', 'S', 'O', 'N', 'D'].map((m, i) => (
            <Text key={i} style={styles.chartLabel}>{m}</Text>
          ))}
        </View>
        <View style={styles.yieldSummary}>
          <View style={styles.yieldItem}>
            <Text style={styles.yieldItemLabel}>Total Yield</Text>
            <Text style={styles.yieldItemValue}>
              +{formatCurrency(totalYield, 'NGN')}
            </Text>
          </View>
          <View style={styles.yieldItem}>
            <Text style={styles.yieldItemLabel}>APY</Text>
            <Text style={styles.yieldItemValue}>8.5%</Text>
          </View>
          <View style={styles.yieldItem}>
            <Text style={styles.yieldItemLabel}>This Month</Text>
            <Text style={styles.yieldItemValue}>
              +{formatCurrency(MOCK_YIELD_DATA[MOCK_YIELD_DATA.length - 1] - MOCK_YIELD_DATA[MOCK_YIELD_DATA.length - 2], 'NGN')}
            </Text>
          </View>
        </View>
      </View>

      <TouchableOpacity style={styles.sendButton} onPress={onSendMoney}>
        <Text style={styles.sendButtonText}>Send Money & Save</Text>
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
    paddingHorizontal: 20,
    paddingTop: 60,
    paddingBottom: 40,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 24,
  },
  back: {
    color: '#0055FF',
    fontSize: 16,
  },
  headerTitle: {
    fontSize: 20,
    fontWeight: '800',
    color: '#FFFFFF',
  },
  totalCard: {
    backgroundColor: '#1A2A40',
    borderRadius: 16,
    padding: 24,
    marginBottom: 24,
  },
  totalLabel: {
    fontSize: 13,
    fontWeight: '600',
    color: '#667788',
    textTransform: 'uppercase',
    letterSpacing: 1,
    marginBottom: 4,
  },
  totalValue: {
    fontSize: 32,
    fontWeight: '800',
    color: '#FFFFFF',
    marginBottom: 4,
  },
  totalYield: {
    fontSize: 14,
    fontWeight: '600',
    color: '#00D4AA',
    marginBottom: 16,
  },
  totalBar: {
    height: 6,
    backgroundColor: '#0A1628',
    borderRadius: 3,
    marginBottom: 8,
    overflow: 'hidden',
  },
  totalBarFill: {
    height: '100%',
    backgroundColor: '#0055FF',
    borderRadius: 3,
  },
  totalProgress: {
    fontSize: 12,
    color: '#667788',
  },
  sectionTitle: {
    fontSize: 16,
    fontWeight: '700',
    color: '#FFFFFF',
    marginBottom: 16,
  },
  goalCard: {
    backgroundColor: '#1A2A40',
    borderRadius: 12,
    padding: 16,
    flexDirection: 'row',
    alignItems: 'center',
    marginBottom: 12,
  },
  goalLeft: {
    marginRight: 16,
  },
  goalRight: {
    flex: 1,
  },
  goalName: {
    fontSize: 16,
    fontWeight: '700',
    color: '#FFFFFF',
    marginBottom: 4,
  },
  goalAmount: {
    fontSize: 18,
    fontWeight: '800',
    color: '#00D4AA',
    marginBottom: 2,
  },
  goalTarget: {
    fontSize: 12,
    color: '#667788',
    marginBottom: 2,
  },
  goalYield: {
    fontSize: 12,
    fontWeight: '600',
    color: '#00D4AA',
  },
  yieldCard: {
    backgroundColor: '#1A2A40',
    borderRadius: 16,
    padding: 20,
    marginBottom: 24,
  },
  chart: {
    flexDirection: 'row',
    alignItems: 'flex-end',
    height: 130,
    gap: 4,
    marginBottom: 8,
    paddingTop: 10,
  },
  barWrapper: {
    flex: 1,
    alignItems: 'center',
    justifyContent: 'flex-end',
    height: '100%',
  },
  bar: {
    width: '100%',
    borderRadius: 3,
    minHeight: 4,
  },
  chartLabels: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    marginBottom: 20,
  },
  chartLabel: {
    fontSize: 10,
    color: '#667788',
    textAlign: 'center',
    flex: 1,
  },
  yieldSummary: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    borderTopWidth: 1,
    borderTopColor: '#0A1628',
    paddingTop: 16,
  },
  yieldItem: {
    alignItems: 'center',
  },
  yieldItemLabel: {
    fontSize: 11,
    color: '#667788',
    marginBottom: 4,
  },
  yieldItemValue: {
    fontSize: 15,
    fontWeight: '700',
    color: '#00D4AA',
  },
  sendButton: {
    backgroundColor: '#0055FF',
    paddingVertical: 16,
    borderRadius: 12,
    alignItems: 'center',
  },
  sendButtonText: {
    color: '#FFFFFF',
    fontSize: 17,
    fontWeight: '700',
  },
});
