import React, { useState, useEffect, useCallback } from 'react';
import {
  View,
  Text,
  StyleSheet,
  ScrollView,
  TouchableOpacity,
  TextInput,
  ActivityIndicator,
  Alert,
} from 'react-native';
import { api, VaultPool, VaultPosition } from '../services/api';

interface Props {
  onBack: () => void;
}

const FALLBACK_POOLS: VaultPool[] = [
  {
    id: 'mock-ngn-vault',
    onchain_pool_id: 0,
    name: 'NGN High Yield Vault',
    local_asset: 'NGN',
    apy_bps: 1200,
    apy_percent: 12.0,
    min_lockup_secs: 2592000,
    total_deposits: 14500000,
    total_shares: 14500000,
    share_price: 10000000,
    is_paused: false,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
  {
    id: 'mock-kes-pool',
    onchain_pool_id: 1,
    name: 'KES Growth Pool',
    local_asset: 'KES',
    apy_bps: 950,
    apy_percent: 9.5,
    min_lockup_secs: 604800,
    total_deposits: 8200000,
    total_shares: 8200000,
    share_price: 10000000,
    is_paused: false,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
  {
    id: 'mock-usd-flex',
    onchain_pool_id: 2,
    name: 'USD Flex Yield Pool',
    local_asset: 'USDC',
    apy_bps: 600,
    apy_percent: 6.0,
    min_lockup_secs: 0,
    total_deposits: 250000,
    total_shares: 250000,
    share_price: 10000000,
    is_paused: false,
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  },
];

function formatDuration(seconds: number): string {
  if (seconds <= 0) return 'Flexible (No lock)';
  const days = Math.round(seconds / 86400);
  if (days >= 30) {
    const months = Math.round(days / 30);
    return `${months} month${months > 1 ? 's' : ''} lock`;
  }
  if (days >= 7) {
    const weeks = Math.round(days / 7);
    return `${weeks} week${weeks > 1 ? 's' : ''} lock`;
  }
  return `${days} day${days > 1 ? 's' : ''} lock`;
}

export default function VaultPoolsScreen({ onBack }: Props) {
  const [pools, setPools] = useState<VaultPool[]>(FALLBACK_POOLS);
  const [positions, setPositions] = useState<VaultPosition[]>([]);
  const [loading, setLoading] = useState(true);
  const [activeTab, setActiveTab] = useState<'pools' | 'positions'>('pools');

  // Deposit state
  const [depositModalPool, setDepositModalPool] = useState<VaultPool | null>(null);
  const [depositAmount, setDepositAmount] = useState('');
  const [depositing, setDepositing] = useState(false);

  // Withdraw state
  const [withdrawModalPosition, setWithdrawModalPosition] = useState<VaultPosition | null>(null);
  const [withdrawShares, setWithdrawShares] = useState('');
  const [withdrawing, setWithdrawing] = useState(false);

  const fetchData = useCallback(async () => {
    try {
      const [fetchedPools, fetchedPositions] = await Promise.allSettled([
        api.listVaultPools(),
        api.listVaultPositions(),
      ]);

      if (fetchedPools.status === 'fulfilled' && fetchedPools.value.length > 0) {
        setPools(fetchedPools.value);
      }
      if (fetchedPositions.status === 'fulfilled') {
        setPositions(fetchedPositions.value);
      }
    } catch {
      // Use fallback pools
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  const handleDeposit = async () => {
    if (!depositModalPool) return;
    const amountNum = parseFloat(depositAmount);
    if (isNaN(amountNum) || amountNum <= 0) {
      Alert.alert('Invalid Amount', 'Please enter a valid deposit amount greater than 0.');
      return;
    }

    setDepositing(true);
    try {
      await api.depositToVault(depositModalPool.id, amountNum);
      Alert.alert('Deposit Successful', `Successfully deposited into ${depositModalPool.name}!`);
      setDepositAmount('');
      setDepositModalPool(null);
      await fetchData();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : 'Deposit failed';
      Alert.alert('Deposit Error', msg);
    } finally {
      setDepositing(false);
    }
  };

  const handleWithdraw = async () => {
    if (!withdrawModalPosition) return;
    const sharesNum = parseFloat(withdrawShares);
    if (isNaN(sharesNum) || sharesNum <= 0 || sharesNum > withdrawModalPosition.shares) {
      Alert.alert('Invalid Shares', 'Please enter a valid number of shares up to your balance.');
      return;
    }

    if (withdrawModalPosition.is_locked) {
      Alert.alert(
        'Lockup Active',
        `This position is locked until ${new Date(withdrawModalPosition.locked_until).toLocaleDateString()}. Withdrawals are not allowed before expiry.`,
      );
      return;
    }

    setWithdrawing(true);
    try {
      await api.withdrawFromVault(withdrawModalPosition.pool_id, sharesNum);
      Alert.alert('Withdrawal Successful', 'Funds have been returned to your wallet balance.');
      setWithdrawShares('');
      setWithdrawModalPosition(null);
      await fetchData();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : 'Withdrawal failed';
      Alert.alert('Withdrawal Error', msg);
    } finally {
      setWithdrawing(false);
    }
  };

  const totalVaultValue = positions.reduce((sum, pos) => sum + (pos.estimated_value || 0), 0);
  const bestApy = pools.reduce((max, pool) => Math.max(max, pool.apy_percent || (pool.apy_bps / 100)), 0);

  return (
    <View style={styles.container}>
      {/* Header */}
      <View style={styles.header}>
        <TouchableOpacity style={styles.backButton} onPress={onBack}>
          <Text style={styles.backButtonText}>←</Text>
        </TouchableOpacity>
        <View style={styles.headerTitleContainer}>
          <Text style={styles.headerTitle}>Yield Vault Pools</Text>
          <Text style={styles.headerSubtitle}>Passive APY on diaspora savings</Text>
        </View>
      </View>

      <ScrollView contentContainerStyle={styles.scrollContent}>
        {/* Metric Cards */}
        <View style={styles.metricsRow}>
          <View style={styles.metricCard}>
            <Text style={styles.metricLabel}>Total Vault Value</Text>
            <Text style={styles.metricValue}>
              {totalVaultValue > 0 ? `$${totalVaultValue.toLocaleString()}` : '$0.00'}
            </Text>
            <Text style={styles.metricSub}>{positions.length} active position{positions.length === 1 ? '' : 's'}</Text>
          </View>
          <View style={[styles.metricCard, styles.metricCardAccent]}>
            <Text style={styles.metricLabel}>Top APY Rate</Text>
            <Text style={[styles.metricValue, styles.apyText]}>{bestApy.toFixed(1)}%</Text>
            <Text style={styles.metricSub}>Automated yield compounding</Text>
          </View>
        </View>

        {/* Tab Switcher */}
        <View style={styles.tabContainer}>
          <TouchableOpacity
            style={[styles.tabButton, activeTab === 'pools' && styles.tabButtonActive]}
            onPress={() => setActiveTab('pools')}
          >
            <Text style={[styles.tabButtonText, activeTab === 'pools' && styles.tabButtonTextActive]}>
              Available Pools ({pools.length})
            </Text>
          </TouchableOpacity>
          <TouchableOpacity
            style={[styles.tabButton, activeTab === 'positions' && styles.tabButtonActive]}
            onPress={() => setActiveTab('positions')}
          >
            <Text style={[styles.tabButtonText, activeTab === 'positions' && styles.tabButtonTextActive]}>
              My Deposits ({positions.length})
            </Text>
          </TouchableOpacity>
        </View>

        {loading ? (
          <View style={styles.loadingContainer}>
            <ActivityIndicator size="large" color="#0055FF" />
            <Text style={styles.loadingText}>Loading vault pools...</Text>
          </View>
        ) : activeTab === 'pools' ? (
          /* Pools Tab */
          <View style={styles.poolsList}>
            {pools.map((pool) => {
              const apy = pool.apy_percent || (pool.apy_bps / 100);
              return (
                <View key={pool.id} style={styles.poolCard}>
                  <View style={styles.poolCardHeader}>
                    <View style={styles.poolNameContainer}>
                      <Text style={styles.poolName}>{pool.name}</Text>
                      <View style={styles.assetBadge}>
                        <Text style={styles.assetBadgeText}>{pool.local_asset}</Text>
                      </View>
                    </View>
                    <View style={styles.apyBadge}>
                      <Text style={styles.apyBadgeText}>{apy.toFixed(1)}% APY</Text>
                    </View>
                  </View>

                  <View style={styles.poolDetails}>
                    <View style={styles.detailItem}>
                      <Text style={styles.detailLabel}>Lockup Period</Text>
                      <Text style={styles.detailValue}>{formatDuration(pool.min_lockup_secs)}</Text>
                    </View>
                    <View style={styles.detailItem}>
                      <Text style={styles.detailLabel}>TVL</Text>
                      <Text style={styles.detailValue}>
                        {pool.total_deposits.toLocaleString()} {pool.local_asset}
                      </Text>
                    </View>
                    <View style={styles.detailItem}>
                      <Text style={styles.detailLabel}>Status</Text>
                      <Text style={[styles.detailValue, pool.is_paused ? styles.pausedStatus : styles.activeStatus]}>
                        {pool.is_paused ? 'Paused' : 'Active'}
                      </Text>
                    </View>
                  </View>

                  <TouchableOpacity
                    style={[styles.actionButton, pool.is_paused && styles.actionButtonDisabled]}
                    disabled={pool.is_paused}
                    onPress={() => {
                      setDepositModalPool(pool);
                      setDepositAmount('');
                    }}
                  >
                    <Text style={styles.actionButtonText}>
                      {pool.is_paused ? 'Pool Paused' : 'Deposit & Earn'}
                    </Text>
                  </TouchableOpacity>
                </View>
              );
            })}
          </View>
        ) : (
          /* Positions Tab */
          <View style={styles.positionsList}>
            {positions.length === 0 ? (
              <View style={styles.emptyContainer}>
                <Text style={styles.emptyIcon}>🏦</Text>
                <Text style={styles.emptyTitle}>No Vault Deposits Yet</Text>
                <Text style={styles.emptySubtitle}>
                  Deposit local or diaspora funds into yield vaults to earn automated interest.
                </Text>
                <TouchableOpacity
                  style={styles.emptyButton}
                  onPress={() => setActiveTab('pools')}
                >
                  <Text style={styles.emptyButtonText}>Explore Vault Pools</Text>
                </TouchableOpacity>
              </View>
            ) : (
              positions.map((pos) => {
                const isLocked = pos.is_locked || new Date(pos.locked_until) > new Date();
                const lockDate = new Date(pos.locked_until).toLocaleDateString();
                return (
                  <View key={pos.id} style={styles.poolCard}>
                    <View style={styles.poolCardHeader}>
                      <View style={styles.poolNameContainer}>
                        <Text style={styles.poolName}>{pos.pool_name}</Text>
                        <View style={styles.assetBadge}>
                          <Text style={styles.assetBadgeText}>{pos.local_asset}</Text>
                        </View>
                      </View>
                      <View style={[styles.lockBadge, isLocked ? styles.lockBadgeLocked : styles.lockBadgeUnlocked]}>
                        <Text style={styles.lockBadgeText}>
                          {isLocked ? `🔒 Locked until ${lockDate}` : '🔓 Unlocked'}
                        </Text>
                      </View>
                    </View>

                    <View style={styles.poolDetails}>
                      <View style={styles.detailItem}>
                        <Text style={styles.detailLabel}>Deposited</Text>
                        <Text style={styles.detailValue}>
                          {pos.deposited_amount.toLocaleString()} {pos.local_asset}
                        </Text>
                      </View>
                      <View style={styles.detailItem}>
                        <Text style={styles.detailLabel}>Shares</Text>
                        <Text style={styles.detailValue}>{pos.shares.toLocaleString()}</Text>
                      </View>
                      <View style={styles.detailItem}>
                        <Text style={styles.detailLabel}>Est. Value</Text>
                        <Text style={[styles.detailValue, styles.apyText]}>
                          {pos.estimated_value.toLocaleString()} {pos.local_asset}
                        </Text>
                      </View>
                    </View>

                    <TouchableOpacity
                      style={[styles.withdrawButton, isLocked && styles.actionButtonDisabled]}
                      disabled={isLocked}
                      onPress={() => {
                        setWithdrawModalPosition(pos);
                        setWithdrawShares(pos.shares.toString());
                      }}
                    >
                      <Text style={styles.actionButtonText}>
                        {isLocked ? 'Locked (Timelock Active)' : 'Withdraw Funds'}
                      </Text>
                    </TouchableOpacity>
                  </View>
                );
              })
            )}
          </View>
        )}

        {/* Deposit Modal / Form Overlay */}
        {depositModalPool && (
          <View style={styles.modalCard}>
            <View style={styles.modalHeader}>
              <Text style={styles.modalTitle}>Deposit into {depositModalPool.name}</Text>
              <TouchableOpacity onPress={() => setDepositModalPool(null)}>
                <Text style={styles.closeButton}>✕</Text>
              </TouchableOpacity>
            </View>

            <Text style={styles.modalSub}>
              Earn {((depositModalPool.apy_percent || depositModalPool.apy_bps / 100)).toFixed(1)}% APY. Minimum lockup:{' '}
              {formatDuration(depositModalPool.min_lockup_secs)}.
            </Text>

            <View style={styles.inputContainer}>
              <Text style={styles.inputLabel}>Deposit Amount ({depositModalPool.local_asset})</Text>
              <TextInput
                style={styles.textInput}
                keyboardType="numeric"
                placeholder="e.g. 50000"
                placeholderTextColor="#667788"
                value={depositAmount}
                onChangeText={setDepositAmount}
              />
            </View>

            <View style={styles.quickAmounts}>
              {[5000, 10000, 50000, 100000].map((val) => (
                <TouchableOpacity
                  key={val}
                  style={styles.quickButton}
                  onPress={() => setDepositAmount(val.toString())}
                >
                  <Text style={styles.quickButtonText}>+{val.toLocaleString()}</Text>
                </TouchableOpacity>
              ))}
            </View>

            <TouchableOpacity
              style={[styles.submitButton, depositing && styles.actionButtonDisabled]}
              disabled={depositing}
              onPress={handleDeposit}
            >
              {depositing ? (
                <ActivityIndicator color="#FFFFFF" />
              ) : (
                <Text style={styles.submitButtonText}>Confirm Deposit</Text>
              )}
            </TouchableOpacity>
          </View>
        )}

        {/* Withdraw Modal / Form Overlay */}
        {withdrawModalPosition && (
          <View style={styles.modalCard}>
            <View style={styles.modalHeader}>
              <Text style={styles.modalTitle}>Withdraw from {withdrawModalPosition.pool_name}</Text>
              <TouchableOpacity onPress={() => setWithdrawModalPosition(null)}>
                <Text style={styles.closeButton}>✕</Text>
              </TouchableOpacity>
            </View>

            <Text style={styles.modalSub}>
              Available shares: {withdrawModalPosition.shares.toLocaleString()} (Est.{' '}
              {withdrawModalPosition.estimated_value.toLocaleString()} {withdrawModalPosition.local_asset})
            </Text>

            <View style={styles.inputContainer}>
              <Text style={styles.inputLabel}>Shares to Redeem</Text>
              <TextInput
                style={styles.textInput}
                keyboardType="numeric"
                placeholder={`Max: ${withdrawModalPosition.shares}`}
                placeholderTextColor="#667788"
                value={withdrawShares}
                onChangeText={setWithdrawShares}
              />
            </View>

            <TouchableOpacity
              style={[styles.submitButton, styles.withdrawSubmit, withdrawing && styles.actionButtonDisabled]}
              disabled={withdrawing}
              onPress={handleWithdraw}
            >
              {withdrawing ? (
                <ActivityIndicator color="#FFFFFF" />
              ) : (
                <Text style={styles.submitButtonText}>Confirm Withdrawal</Text>
              )}
            </TouchableOpacity>
          </View>
        )}
      </ScrollView>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0A1628',
  },
  header: {
    flexDirection: 'row',
    alignItems: 'center',
    paddingHorizontal: 16,
    paddingTop: 16,
    paddingBottom: 16,
    borderBottomWidth: 1,
    borderBottomColor: '#1A2A40',
  },
  backButton: {
    padding: 8,
    marginRight: 12,
  },
  backButtonText: {
    color: '#0055FF',
    fontSize: 24,
    fontWeight: '700',
  },
  headerTitleContainer: {
    flex: 1,
  },
  headerTitle: {
    fontSize: 20,
    fontWeight: '700',
    color: '#FFFFFF',
  },
  headerSubtitle: {
    fontSize: 12,
    color: '#8899AA',
    marginTop: 2,
  },
  scrollContent: {
    padding: 16,
    paddingBottom: 40,
  },
  metricsRow: {
    flexDirection: 'row',
    gap: 12,
    marginBottom: 20,
  },
  metricCard: {
    flex: 1,
    backgroundColor: '#1A2A40',
    borderRadius: 12,
    padding: 16,
  },
  metricCardAccent: {
    borderLeftWidth: 4,
    borderLeftColor: '#00D084',
  },
  metricLabel: {
    fontSize: 12,
    color: '#8899AA',
    fontWeight: '600',
    marginBottom: 6,
    textTransform: 'uppercase',
  },
  metricValue: {
    fontSize: 22,
    fontWeight: '800',
    color: '#FFFFFF',
    marginBottom: 4,
  },
  apyText: {
    color: '#00D084',
  },
  metricSub: {
    fontSize: 11,
    color: '#667788',
  },
  tabContainer: {
    flexDirection: 'row',
    backgroundColor: '#1A2A40',
    borderRadius: 10,
    padding: 4,
    marginBottom: 20,
  },
  tabButton: {
    flex: 1,
    paddingVertical: 10,
    alignItems: 'center',
    borderRadius: 8,
  },
  tabButtonActive: {
    backgroundColor: '#0055FF',
  },
  tabButtonText: {
    fontSize: 13,
    fontWeight: '600',
    color: '#8899AA',
  },
  tabButtonTextActive: {
    color: '#FFFFFF',
  },
  loadingContainer: {
    alignItems: 'center',
    paddingVertical: 40,
  },
  loadingText: {
    color: '#8899AA',
    marginTop: 12,
    fontSize: 14,
  },
  poolsList: {
    gap: 16,
  },
  positionsList: {
    gap: 16,
  },
  poolCard: {
    backgroundColor: '#1A2A40',
    borderRadius: 14,
    padding: 18,
    borderWidth: 1,
    borderColor: '#2A3A50',
  },
  poolCardHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'flex-start',
    marginBottom: 14,
  },
  poolNameContainer: {
    flex: 1,
    marginRight: 10,
  },
  poolName: {
    fontSize: 17,
    fontWeight: '700',
    color: '#FFFFFF',
    marginBottom: 6,
  },
  assetBadge: {
    alignSelf: 'flex-start',
    backgroundColor: '#0D1B2A',
    borderRadius: 6,
    paddingHorizontal: 8,
    paddingVertical: 2,
    borderWidth: 1,
    borderColor: '#2A3A50',
  },
  assetBadgeText: {
    fontSize: 11,
    fontWeight: '700',
    color: '#8899AA',
  },
  apyBadge: {
    backgroundColor: 'rgba(0, 208, 132, 0.15)',
    borderRadius: 8,
    paddingHorizontal: 10,
    paddingVertical: 6,
    borderWidth: 1,
    borderColor: '#00D084',
  },
  apyBadgeText: {
    color: '#00D084',
    fontSize: 13,
    fontWeight: '700',
  },
  lockBadge: {
    borderRadius: 8,
    paddingHorizontal: 10,
    paddingVertical: 5,
  },
  lockBadgeLocked: {
    backgroundColor: 'rgba(255, 184, 0, 0.15)',
    borderWidth: 1,
    borderColor: '#FFB800',
  },
  lockBadgeUnlocked: {
    backgroundColor: 'rgba(0, 208, 132, 0.15)',
    borderWidth: 1,
    borderColor: '#00D084',
  },
  lockBadgeText: {
    fontSize: 11,
    fontWeight: '700',
    color: '#FFFFFF',
  },
  poolDetails: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    backgroundColor: '#0D1B2A',
    borderRadius: 10,
    padding: 12,
    marginBottom: 16,
  },
  detailItem: {
    flex: 1,
  },
  detailLabel: {
    fontSize: 11,
    color: '#667788',
    marginBottom: 4,
    textTransform: 'uppercase',
  },
  detailValue: {
    fontSize: 13,
    fontWeight: '700',
    color: '#FFFFFF',
  },
  activeStatus: {
    color: '#00D084',
  },
  pausedStatus: {
    color: '#FF3B30',
  },
  actionButton: {
    backgroundColor: '#0055FF',
    borderRadius: 10,
    paddingVertical: 12,
    alignItems: 'center',
  },
  withdrawButton: {
    backgroundColor: '#2A3A50',
    borderRadius: 10,
    paddingVertical: 12,
    alignItems: 'center',
  },
  actionButtonDisabled: {
    opacity: 0.5,
  },
  actionButtonText: {
    color: '#FFFFFF',
    fontSize: 14,
    fontWeight: '700',
  },
  emptyContainer: {
    backgroundColor: '#1A2A40',
    borderRadius: 14,
    padding: 32,
    alignItems: 'center',
  },
  emptyIcon: {
    fontSize: 48,
    marginBottom: 12,
  },
  emptyTitle: {
    fontSize: 18,
    fontWeight: '700',
    color: '#FFFFFF',
    marginBottom: 8,
  },
  emptySubtitle: {
    fontSize: 13,
    color: '#8899AA',
    textAlign: 'center',
    lineHeight: 18,
    marginBottom: 20,
  },
  emptyButton: {
    backgroundColor: '#0055FF',
    borderRadius: 10,
    paddingVertical: 10,
    paddingHorizontal: 20,
  },
  emptyButtonText: {
    color: '#FFFFFF',
    fontSize: 14,
    fontWeight: '700',
  },
  modalCard: {
    marginTop: 20,
    backgroundColor: '#1A2A40',
    borderRadius: 14,
    padding: 20,
    borderWidth: 2,
    borderColor: '#0055FF',
  },
  modalHeader: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: 8,
  },
  modalTitle: {
    fontSize: 18,
    fontWeight: '700',
    color: '#FFFFFF',
    flex: 1,
  },
  closeButton: {
    color: '#8899AA',
    fontSize: 18,
    padding: 4,
  },
  modalSub: {
    fontSize: 13,
    color: '#8899AA',
    marginBottom: 16,
    lineHeight: 18,
  },
  inputContainer: {
    marginBottom: 14,
  },
  inputLabel: {
    fontSize: 12,
    fontWeight: '600',
    color: '#8899AA',
    marginBottom: 6,
  },
  textInput: {
    backgroundColor: '#0D1B2A',
    borderRadius: 10,
    paddingHorizontal: 14,
    paddingVertical: 12,
    color: '#FFFFFF',
    fontSize: 16,
    borderWidth: 1,
    borderColor: '#2A3A50',
  },
  quickAmounts: {
    flexDirection: 'row',
    gap: 8,
    marginBottom: 18,
  },
  quickButton: {
    flex: 1,
    backgroundColor: '#0D1B2A',
    borderRadius: 8,
    paddingVertical: 8,
    alignItems: 'center',
    borderWidth: 1,
    borderColor: '#2A3A50',
  },
  quickButtonText: {
    fontSize: 12,
    color: '#8899AA',
    fontWeight: '600',
  },
  submitButton: {
    backgroundColor: '#0055FF',
    borderRadius: 10,
    paddingVertical: 14,
    alignItems: 'center',
  },
  withdrawSubmit: {
    backgroundColor: '#00D084',
  },
  submitButtonText: {
    color: '#FFFFFF',
    fontSize: 15,
    fontWeight: '700',
  },
});
