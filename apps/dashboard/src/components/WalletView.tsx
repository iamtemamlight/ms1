/**
 * @license
 * SPDX-License-Identifier: Apache-2.0
 */

import React, { useState, useEffect } from 'react';
import { 
  WalletState, 
  DashboardSettings, 
  CustomWalletItem 
} from '../types';
import { 
  Wallet, 
  Plus, 
  Pencil, 
  Trash2, 
  Save, 
  X, 
  ChevronUp, 
  ChevronDown,
  Check,
  Clock,
  Search
} from 'lucide-react';

interface WalletViewProps {
  wallet: WalletState | null;
  settings: DashboardSettings | null;
  onDeposit: (amount: number, token: string) => Promise<{ success: boolean; error?: string }>;
  onWithdraw: (amount: number, token: string) => Promise<{ success: boolean; error?: string }>;
  onTransferProfit?: () => Promise<void>;
  transferringProfit?: boolean;
  isUpdating: boolean;
  themeMode: 'dark' | 'bright' | 'dusty-blue';
  convertAndFormat: (usdValue: number, minFractionDigits?: number) => string;
  
  walletsList: CustomWalletItem[];
  onUpdateWalletsList: (newList: CustomWalletItem[]) => void;
  onUpdateSettings: (updated: Partial<DashboardSettings>) => Promise<boolean>;
}

interface TransferHistoryItem {
  id: string;
  timestamp: number;
  type: 'AUTO' | 'MANUAL';
  amount: number;
  token: string;
  recipient: string;
  txHash: string;
  status: 'SUCCESS' | 'FAILED';
}

const CHAINS = ['Arbitrum Mainnet', 'Ethereum Mainnet', 'Optimism Mainnet', 'Polygon POS', 'Base', 'Avalanche'];

function shortenAddress(addr: string): string {
  if (!addr || addr.length <= 10) return addr;
  return `${addr.slice(0, 6)}...${addr.slice(-4)}`;
}

function shortenPrivateKey(key: string): string {
  if (!key || key.length <= 10) return key;
  if (key === 'VAULT_MANAGED' || key === '••••••••') return key;
  return `${key.slice(0, 6)}...${key.slice(-4)}`;
}

function simulateChainAndBalance(address: string): { chain: string; balance: number } {
  let hash = 0;
  for (let i = 0; i < address.length; i++) {
    hash = ((hash << 5) - hash) + address.charCodeAt(i);
    hash = hash & hash;
  }
  const chain = CHAINS[Math.abs(hash) % CHAINS.length];
  const balance = Math.abs(hash % 50000) + 100;
  return { chain, balance };
}

export default function WalletView(props: WalletViewProps) {
  const {
    wallet,
    settings,
    onTransferProfit,
    transferringProfit,
    themeMode,
    convertAndFormat,
    walletsList,
    onUpdateWalletsList,
    onUpdateSettings,
  } = props;

  const [sortBy, setSortBy] = useState<keyof CustomWalletItem>('address');
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('asc');
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editAddress, setEditAddress] = useState('');
  const [editPrivateKey, setEditPrivateKey] = useState('');
  const [editChain, setEditChain] = useState('');
  const [editBalance, setEditBalance] = useState<number>(0);
  const [showAddForm, setShowAddForm] = useState(false);
  const [formAddress, setFormAddress] = useState('');
  const [formPrivateKey, setFormPrivateKey] = useState('');
  const [formChain, setFormChain] = useState('');
  const [formBalance, setFormBalance] = useState<number>(0);
  const [isDetecting, setIsDetecting] = useState(false);
  const [manualAmount, setManualAmount] = useState<string>('');
  const [withdrawalMessage, setWithdrawalMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);

  const [transferHistory, setTransferHistory] = useState<TransferHistoryItem[]>(() => {
    const saved = localStorage.getItem('allbright_transfer_history');
    if (saved) {
      try { return JSON.parse(saved); } catch (e) { }
    }
    return [];
  });

  const getThemeClasses = () => {
    switch (themeMode) {
      case 'bright':
        return {
          card: 'bg-white border border-slate-200 rounded-2xl p-5 shadow-sm',
          textMuted: 'text-slate-500',
          textTitle: 'text-slate-800 font-bold',
          textWhite: 'text-slate-900',
          accentText: 'text-teal-600',
          tableHeaderBg: 'bg-slate-50 border-b border-slate-200 text-slate-600',
          tableRowBorder: 'border-b border-slate-100',
          tableRowHover: 'hover:bg-slate-50/50',
          inputBg: 'bg-white border border-slate-200 text-slate-800 focus:border-teal-500',
          btnPrimary: 'bg-teal-600 hover:bg-teal-700 text-white',
          btnSecondary: 'bg-slate-100 hover:bg-slate-200 border border-slate-200 text-slate-700',
        };
      case 'dusty-blue':
        return {
          card: 'bg-[#1e2a3d] border border-[#314363] rounded-2xl p-5 shadow-md',
          textMuted: 'text-slate-400',
          textTitle: 'text-sky-100 font-bold',
          textWhite: 'text-white',
          accentText: 'text-teal-400',
          tableHeaderBg: 'bg-[#131b27] border-b border-[#314363] text-sky-200',
          tableRowBorder: 'border-b border-[#314363]/40',
          tableRowHover: 'hover:bg-[#1a2537]',
          inputBg: 'bg-[#131b27] border border-[#314363] text-slate-100 focus:border-teal-400',
          btnPrimary: 'bg-gradient-to-r from-teal-500 to-teal-600 text-slate-950 font-bold',
          btnSecondary: 'bg-[#24324a] hover:bg-[#2d3e5c] border border-[#314363] text-sky-200',
        };
      case 'dark':
      default:
        return {
          card: 'bg-slate-900 border border-slate-800/80 rounded-2xl p-5 shadow-xl',
          textMuted: 'text-slate-400',
          textTitle: 'text-white font-bold',
          textWhite: 'text-white',
          accentText: 'text-teal-400',
          tableHeaderBg: 'bg-slate-950 border-b border-slate-800 text-slate-300',
          tableRowBorder: 'border-b border-slate-800/50',
          tableRowHover: 'hover:bg-slate-800/30',
          inputBg: 'bg-slate-950 border border-slate-800 text-slate-100 focus:border-teal-500',
          btnPrimary: 'bg-teal-500 hover:bg-teal-600 text-slate-950 font-extrabold',
          btnSecondary: 'bg-slate-800 hover:bg-slate-750 border border-slate-700 text-slate-300',
        };
    }
  };

  const styles = getThemeClasses();

  useEffect(() => {
    if (formAddress && formAddress.startsWith('0x') && formAddress.length === 42) {
      setIsDetecting(true);
      const timer = setTimeout(() => {
        const detected = simulateChainAndBalance(formAddress);
        setFormChain(detected.chain);
        setFormBalance(detected.balance);
        setIsDetecting(false);
      }, 800);
      return () => clearTimeout(timer);
    }
  }, [formAddress]);

  const handleSort = (column: keyof CustomWalletItem) => {
    if (sortBy === column) {
      setSortOrder(sortOrder === 'asc' ? 'desc' : 'asc');
    } else {
      setSortBy(column);
      setSortOrder('asc');
    }
  };

  const handleAddWallet = (e: React.FormEvent) => {
    e.preventDefault();
    const newWallet: CustomWalletItem = {
      id: `w-${Date.now()}`,
      name: 'Wallet',
      address: shortenAddress(formAddress),
      privateKey: shortenPrivateKey(formPrivateKey) || '••••••••',
      chain: formChain,
      balance: Number(formBalance) || 0,
      isActive: true
    };
    onUpdateWalletsList([...walletsList, newWallet]);
    setFormAddress('');
    setFormPrivateKey('');
    setFormChain('');
    setFormBalance(0);
    setShowAddForm(false);
  };

  const handleStartEdit = (w: CustomWalletItem) => {
    setEditingId(w.id);
    setEditAddress(w.address);
    setEditPrivateKey(w.privateKey);
    setEditChain(w.chain);
    setEditBalance(w.balance);
  };

  const handleSaveEdit = (id: string) => {
    const updated = walletsList.map((item) => {
      if (item.id === id) {
        return {
          ...item,
          address: shortenAddress(editAddress),
          privateKey: shortenPrivateKey(editPrivateKey) || '••••••••',
          chain: editChain,
          balance: Number(editBalance) || 0
        };
      }
      return item;
    });
    onUpdateWalletsList(updated);
    setEditingId(null);
  };

  const handleDeleteWallet = (id: string) => {
    if (confirm('Delete this wallet?')) {
      const filtered = walletsList.filter((item) => item.id !== id);
      onUpdateWalletsList(filtered);
    }
  };

  const executeTransfer = async () => {
    const amount = Number(manualAmount);
    const maxBalance = settings?.accumulatedProfitsUsd || 250.0;

    if (isNaN(amount) || amount <= 0) {
      setWithdrawalMessage({ type: 'error', text: 'Enter a valid amount.' });
      return;
    }
    if (amount > maxBalance) {
      setWithdrawalMessage({ type: 'error', text: `Insufficient balance. Max: ${convertAndFormat(maxBalance)}.` });
      return;
    }

    try {
      if (onTransferProfit) {
        await onTransferProfit();
      }

      const txHash = '0x' + Array.from({ length: 64 }, () => Math.floor(Math.random() * 16).toString(16)).join('');
      const newHistoryItem: TransferHistoryItem = {
        id: `tf-${Date.now()}`,
        timestamp: Date.now(),
        type: settings?.profitTransferMode === 'AUTO' ? 'AUTO' : 'MANUAL',
        amount,
        token: 'USDC',
        recipient: settings?.ownerWalletAddress || '0x...',
        txHash,
        status: 'SUCCESS'
      };

      setTransferHistory([newHistoryItem, ...transferHistory]);
      setWithdrawalMessage({
        type: 'success',
        text: `Transferred ${convertAndFormat(amount)} successfully!`
      });
      setManualAmount('');
    } catch (err) {
      setWithdrawalMessage({ type: 'error', text: 'Transfer failed.' });
    }
  };

  const sortedWallets = [...walletsList].sort((a, b) => {
    let aVal = a[sortBy];
    let bVal = b[sortBy];
    if (typeof aVal === 'string' && typeof bVal === 'string') {
      return sortOrder === 'asc' ? aVal.localeCompare(bVal) : bVal.localeCompare(aVal);
    }
    if (typeof aVal === 'number' && typeof bVal === 'number') {
      return sortOrder === 'asc' ? aVal - bVal : bVal - aVal;
    }
    return 0;
  });

  const totalBalance = walletsList.reduce((sum, w) => sum + w.balance, 0);
  const isAutoPayoutOn = settings?.profitTransferMode === 'AUTO';

  return (
    <div className="space-y-6 animate-fadeIn" id="wallet-dashboard-parent">
      
      {/* HEADER */}
      <div className="flex flex-col md:flex-row md:items-center justify-between gap-4 border-b border-slate-800/10 pb-4">
        <div>
          <h1 className={`text-xl font-sans font-extrabold tracking-tight ${styles.textWhite}`}>
            Wallets
          </h1>
          <p className={`text-xs ${styles.textMuted} mt-0.5`}>
            Manage your wallets
          </p>
        </div>

        <div className={`flex items-center space-x-3 px-4 py-2.5 rounded-xl ${styles.card}`}>
          <div className="p-1.5 rounded-lg bg-teal-500/10 text-teal-400">
            <Wallet className="h-5 w-5" />
          </div>
          <div>
            <div className="text-[10px] font-bold uppercase tracking-wider text-slate-400">Total Balance</div>
            <div className="text-lg font-mono font-bold text-teal-400">{convertAndFormat(totalBalance)}</div>
          </div>
        </div>
      </div>

      {/* WALLET TABLE */}
      <div className={styles.card}>
        <div className="flex items-center justify-between mb-4">
          <h2 className={`text-base font-bold ${styles.textWhite}`}>
            Wallets
            <span className="text-xs px-2 py-0.5 rounded-full bg-teal-500/10 text-teal-400 font-mono font-bold ml-2">
              {walletsList.length}
            </span>
          </h2>
          <button
            onClick={() => setShowAddForm(!showAddForm)}
            className={`px-3 py-1.5 rounded-xl text-xs font-bold transition-all flex items-center space-x-1.5 cursor-pointer ${styles.btnPrimary}`}
          >
            <Plus className="h-4 w-4" />
            <span>Add Wallet</span>
          </button>
        </div>

        {/* INLINE ADD FORM */}
        {showAddForm && (
          <form onSubmit={handleAddWallet} className="mb-4 p-4 rounded-xl bg-slate-950/40 border border-slate-800/60 space-y-3">
            <div className="grid grid-cols-1 md:grid-cols-4 gap-3">
              <div>
                <label className="block text-[10px] font-bold text-slate-400 uppercase tracking-wider mb-1">Address</label>
                <div className="relative">
                  <input 
                    type="text" 
                    required
                    placeholder="0x..."
                    value={formAddress}
                    onChange={(e) => {
                      setFormAddress(e.target.value);
                      setFormChain('');
                      setFormBalance(0);
                    }}
                    className={`w-full px-3 py-2 text-xs font-mono rounded-xl ${styles.inputBg}`}
                  />
                  {isDetecting && (
                    <div className="absolute right-2 top-2">
                      <Search className="h-3.5 w-3.5 text-teal-400 animate-spin" />
                    </div>
                  )}
                </div>
                {isDetecting && (
                  <p className="text-[9px] text-teal-400 mt-0.5">Detecting chain & balance...</p>
                )}
              </div>
              <div>
                <label className="block text-[10px] font-bold text-slate-400 uppercase tracking-wider mb-1">Private Key</label>
                <input 
                  type="text" 
                  placeholder="Paste private key"
                  value={formPrivateKey}
                  onChange={(e) => setFormPrivateKey(e.target.value)}
                  className={`w-full px-3 py-2 text-xs font-mono rounded-xl ${styles.inputBg}`}
                />
              </div>
              <div>
                <label className="block text-[10px] font-bold text-slate-400 uppercase tracking-wider mb-1">Chain</label>
                <input 
                  type="text"
                  required
                  placeholder={isDetecting ? 'Detecting...' : 'Auto-detected'}
                  value={formChain}
                  readOnly
                  className={`w-full px-3 py-2 text-xs font-mono rounded-xl ${styles.inputBg} ${isDetecting ? 'animate-pulse' : ''}`}
                />
              </div>
              <div>
                <label className="block text-[10px] font-bold text-slate-400 uppercase tracking-wider mb-1">Balance (USD)</label>
                <input 
                  type="number" 
                  required
                  placeholder={isDetecting ? 'Detecting...' : 'Auto-detected'}
                  value={formBalance}
                  readOnly
                  className={`w-full px-3 py-2 text-xs font-mono rounded-xl ${styles.inputBg} ${isDetecting ? 'animate-pulse' : ''}`}
                />
              </div>
            </div>
            <div className="flex items-center space-x-2">
              <button
                type="submit"
                disabled={!formAddress || !formChain || isDetecting}
                className={`px-4 py-2 rounded-xl text-xs font-bold ${styles.btnPrimary} disabled:opacity-40 disabled:cursor-not-allowed`}
              >
                Save
              </button>
              <button
                type="button"
                onClick={() => {
                  setShowAddForm(false);
                  setFormAddress('');
                  setFormPrivateKey('');
                  setFormChain('');
                  setFormBalance(0);
                  setIsDetecting(false);
                }}
                className={`px-4 py-2 rounded-xl text-xs font-bold ${styles.btnSecondary}`}
              >
                Cancel
              </button>
            </div>
          </form>
        )}

        <div className="overflow-x-auto rounded-xl border border-slate-800/15">
          <table className="w-full text-left text-xs border-collapse">
            <thead>
              <tr className={styles.tableHeaderBg}>
                <th 
                  onClick={() => handleSort('address')}
                  className="py-3 px-4 font-bold tracking-wider cursor-pointer hover:text-white select-none"
                >
                  <div className="flex items-center space-x-1">
                    <span>Address</span>
                    {sortBy === 'address' && (sortOrder === 'asc' ? <ChevronUp className="h-3.5 w-3.5 text-teal-400" /> : <ChevronDown className="h-3.5 w-3.5 text-teal-400" />)}
                  </div>
                </th>
                <th className="py-3 px-4 font-bold tracking-wider">Private Key</th>
                <th 
                  onClick={() => handleSort('chain')}
                  className="py-3 px-4 font-bold tracking-wider cursor-pointer hover:text-white select-none"
                >
                  <div className="flex items-center space-x-1">
                    <span>Chain</span>
                    {sortBy === 'chain' && (sortOrder === 'asc' ? <ChevronUp className="h-3.5 w-3.5 text-teal-400" /> : <ChevronDown className="h-3.5 w-3.5 text-teal-400" />)}
                  </div>
                </th>
                <th 
                  onClick={() => handleSort('balance')}
                  className="py-3 px-4 font-bold tracking-wider text-right cursor-pointer hover:text-white select-none"
                >
                  <div className="flex items-center justify-end space-x-1">
                    <span>Balance</span>
                    {sortOrder === 'asc' && sortBy === 'balance' ? <ChevronUp className="h-3.5 w-3.5 text-teal-400" /> : sortOrder === 'desc' && sortBy === 'balance' ? <ChevronDown className="h-3.5 w-3.5 text-teal-400" /> : null}
                  </div>
                </th>
                <th className="py-3 px-4 text-center w-24">Actions</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-800/10">
              {sortedWallets.map((w) => {
                const isEditing = editingId === w.id;

                return (
                  <tr 
                    key={w.id} 
                    className={`${styles.tableRowBorder} ${styles.tableRowHover} transition-all`}
                  >
                    <td className="py-3 px-4 font-mono text-slate-400">
                      {isEditing ? (
                        <input 
                          type="text" 
                          value={editAddress}
                          onChange={(e) => setEditAddress(e.target.value)}
                          className={`w-full px-2 py-1 text-xs font-mono rounded border ${styles.inputBg}`}
                        />
                      ) : (
                        <div className="flex items-center space-x-1.5">
                          <span className="text-[11px]">{shortenAddress(w.address)}</span>
                          <button 
                            onClick={() => navigator.clipboard.writeText(w.address)}
                            className="text-slate-500 hover:text-teal-400 transition-colors"
                            title="Copy full address"
                          >
                            <svg className="h-3.5 w-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 5H6a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2v-1M8 5a2 2 0 002 2h2a2 2 0 002-2M8 5a2 2 0 012-2h2a2 2 0 012 2m0 0h2a2 2 0 012 2v3m2 4H10m0 0l3-3m-3 3l3 3" />
                            </svg>
                          </button>
                        </div>
                      )}
                    </td>

                    <td className="py-3 px-4 font-mono text-slate-400">
                      {isEditing ? (
                        <input 
                          type="text" 
                          value={editPrivateKey}
                          onChange={(e) => setEditPrivateKey(e.target.value)}
                          className={`w-full px-2 py-1 text-xs font-mono rounded border ${styles.inputBg}`}
                        />
                      ) : (
                        <span className="text-[11px]">{shortenPrivateKey(w.privateKey)}</span>
                      )}
                    </td>

                    <td className="py-3 px-4 font-medium text-slate-300">
                      {isEditing ? (
                        <select 
                          value={editChain}
                          onChange={(e) => setEditChain(e.target.value)}
                          className={`w-full px-2 py-1 text-xs rounded border ${styles.inputBg}`}
                        >
                          {CHAINS.map(c => <option key={c} value={c}>{c}</option>)}
                        </select>
                      ) : (
                        <span>{w.chain}</span>
                      )}
                    </td>

                    <td className="py-3 px-4 text-right font-mono font-bold text-slate-200">
                      {isEditing ? (
                        <input 
                          type="number" 
                          step="any"
                          value={editBalance}
                          onChange={(e) => setEditBalance(Number(e.target.value))}
                          className={`w-28 text-right px-2 py-1 text-xs rounded border ${styles.inputBg}`}
                        />
                      ) : (
                        <span>{convertAndFormat(w.balance)}</span>
                      )}
                    </td>

                    <td className="py-3 px-4 text-center font-mono">
                      <div className="flex items-center justify-center space-x-2">
                        {isEditing ? (
                          <>
                            <button
                              onClick={() => handleSaveEdit(w.id)}
                              className="p-1 rounded bg-teal-500/10 hover:bg-teal-500/20 text-teal-400 transition-colors"
                              title="Save"
                            >
                              <Check className="h-3.5 w-3.5" />
                            </button>
                            <button
                              onClick={() => setEditingId(null)}
                              className="p-1 rounded bg-rose-500/10 hover:bg-rose-500/20 text-rose-400 transition-colors"
                              title="Cancel"
                            >
                              <X className="h-3.5 w-3.5" />
                            </button>
                          </>
                        ) : (
                          <>
                            <button
                              onClick={() => handleStartEdit(w)}
                              className="p-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 hover:text-teal-400 transition-colors"
                              title="Edit"
                            >
                              <Pencil className="h-3.5 w-3.5" />
                            </button>
                            <button
                              onClick={() => handleDeleteWallet(w.id)}
                              className="p-1 rounded bg-slate-800 hover:bg-rose-950/40 text-slate-300 hover:text-rose-400 transition-colors"
                              title="Delete"
                            >
                              <Trash2 className="h-3.5 w-3.5" />
                            </button>
                          </>
                        )}
                      </div>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      </div>

      {/* TRANSFER MODE + MANUAL TRANSFER + HISTORY */}
      <div className={styles.card}>
        <div className="flex items-center justify-between mb-4">
          <h2 className={`text-base font-bold ${styles.textWhite}`}>Transfer</h2>
          <div className="flex items-center space-x-3">
            <span className={`px-2 py-0.5 rounded-md text-[10px] font-bold font-mono ${
              isAutoPayoutOn ? 'bg-teal-500/10 text-teal-300' : 'bg-amber-500/10 text-amber-300'
            }`}>
              {isAutoPayoutOn ? 'AUTO' : 'MANUAL'}
            </span>
            <label className="relative inline-flex items-center cursor-pointer">
              <input 
                type="checkbox" 
                checked={isAutoPayoutOn}
                onChange={(e) => onUpdateSettings({ profitTransferMode: e.target.checked ? 'AUTO' : 'MANUAL' })}
                className="sr-only peer" 
              />
              <div className="w-9 h-5 bg-slate-800 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-slate-400 after:border-slate-300 after:border after:rounded-full after:h-4 after:w-4 after:transition-all peer-checked:bg-teal-500 peer-checked:after:bg-slate-950"></div>
            </label>
          </div>
        </div>

        {withdrawalMessage && (
          <div className={`p-3 rounded-xl mb-3 text-xs font-semibold flex items-center space-x-2 border ${
            withdrawalMessage.type === 'success' ? 'bg-emerald-500/10 text-emerald-400 border-emerald-500/20' : 'bg-rose-500/10 text-rose-400 border-rose-500/20'
          }`}>
            <span>{withdrawalMessage.text}</span>
          </div>
        )}

        {/* Manual transfer form - only shown in MANUAL mode */}
        {!isAutoPayoutOn && (
          <form onSubmit={(e) => { e.preventDefault(); executeTransfer(); }} className="flex items-end gap-3 mb-4">
            <div className="flex-1">
              <label className="block text-[10px] font-bold text-slate-400 uppercase tracking-wider mb-1">Amount (USD)</label>
              <input
                type="number"
                step="any"
                placeholder="0.00"
                value={manualAmount}
                onChange={(e) => setManualAmount(e.target.value)}
                className={`w-full text-xs rounded-xl px-3 py-2 ${styles.inputBg}`}
              />
            </div>
            <button
              type="submit"
              disabled={!manualAmount}
              className={`px-4 py-2 rounded-xl text-xs font-bold transition-all flex items-center space-x-1.5 ${styles.btnPrimary} disabled:opacity-40 disabled:cursor-not-allowed`}
            >
              <span>Transfer</span>
            </button>
          </form>
        )}

        {/* Transfer History */}
        <div className="overflow-x-auto rounded-xl border border-slate-800/15">
          <table className="w-full text-left text-xs border-collapse">
            <thead>
              <tr className={styles.tableHeaderBg}>
                <th className="py-2.5 px-4">Time</th>
                <th className="py-2.5 px-4">Type</th>
                <th className="py-2.5 px-4">Recipient</th>
                <th className="py-2.5 px-4">Amount</th>
                <th className="py-2.5 px-4 font-mono">Tx Hash</th>
                <th className="py-2.5 px-4 text-center">Status</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-800/10">
              {transferHistory.map((tf) => (
                <tr key={tf.id} className={`${styles.tableRowBorder} ${styles.tableRowHover}`}>
                  <td className="py-2.5 px-4 text-slate-400">
                    {new Date(tf.timestamp).toLocaleString()}
                  </td>
                  <td className="py-2.5 px-4">
                    <span className={`px-2 py-0.5 rounded text-[10px] font-bold font-mono ${
                      tf.type === 'AUTO' ? 'bg-indigo-500/10 text-indigo-300' : 'bg-teal-500/10 text-teal-300'
                    }`}>
                      {tf.type}
                    </span>
                  </td>
                  <td className="py-2.5 px-4 font-mono text-slate-400 text-[11px]">
                    {tf.recipient.slice(0, 6)}...{tf.recipient.slice(-4)}
                  </td>
                  <td className="py-2.5 px-4 font-mono font-bold text-teal-400">
                    {convertAndFormat(tf.amount)} {tf.token}
                  </td>
                  <td className="py-2.5 px-4 font-mono text-slate-500 text-[11px]">
                    <span className="hover:text-teal-400 cursor-pointer transition-colors" onClick={() => navigator.clipboard.writeText(tf.txHash)} title="Copy">
                      {tf.txHash.slice(0, 18)}...{tf.txHash.slice(-10)}
                    </span>
                  </td>
                  <td className="py-2.5 px-4 text-center">
                    <span className="text-[10px] text-emerald-400 font-bold bg-emerald-500/10 border border-emerald-500/20 px-2 py-0.5 rounded-full">
                      {tf.status}
                    </span>
                  </td>
                </tr>
              ))}
              {transferHistory.length === 0 && (
                <tr>
                  <td colSpan={6} className="py-6 text-center text-slate-500 text-xs">
                    No transfers yet
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </div>

    </div>
  );
}
