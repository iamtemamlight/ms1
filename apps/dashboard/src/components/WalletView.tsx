/**
 * @license
 * SPDX-License-Identifier: Apache-2.0
 *
 * AllBright Withdrawal Wallet page.
 *
 * Security model:
 *  - The EXECUTION wallet (backend env, key held only in Render env vars) is shown
 *    read-only and is kept structurally separate from withdrawal wallets.
 *  - Withdrawal wallets store ONLY: wallet_name, wallet_address, chain, created_date,
 *    status. No PRIVATE_KEY / SEED_PHRASE / MNEMONIC is ever accepted, rendered, or stored.
 */

import React, { useState, useEffect } from 'react';
import {
  Wallet,
  Plus,
  Pencil,
  Trash2,
  ArrowDownToLine,
  ArrowUpFromLine,
  Send,
  Loader2,
  ShieldCheck,
  Zap,
  CheckCircle2,
} from 'lucide-react';
import { WalletState, DashboardSettings, CustomWalletItem } from '../types';

interface WalletViewProps {
  wallet: WalletState | null;
  settings: DashboardSettings;
  onDeposit: (amount: number, token: string) => Promise<{ success: boolean; error?: string }>;
  onWithdraw: (amount: number, token: string) => Promise<{ success: boolean; error?: string }>;
  onTransferProfit: () => Promise<void>;
  transferringProfit: boolean;
  isUpdating: boolean;
  themeMode: 'dark' | 'bright' | 'dusty-blue';
  convertAndFormat: (usdValue: number, minFractionDigits?: number) => string;
  walletsList: CustomWalletItem[];
  onUpdateWalletsList: (updater: (prev: CustomWalletItem[]) => CustomWalletItem[]) => void;
  onUpdateSettings: (updated: Partial<DashboardSettings>) => Promise<boolean>;
}

const CHAIN_OPTIONS = [
  'Ethereum Mainnet',
  'Arbitrum Mainnet',
  'Polygon POS',
  'BNB Smart Chain',
  'Optimism Mainnet',
  'Base Mainnet',
  'Avalanche C-Chain',
];

const TOKENS = ['USDC', 'USDT', 'ETH', 'WBTC'];

// Design system tokens (dark)
const COLORS = {
  bg: '#0D1117',
  card: '#161B22',
  input: '#21262D',
  border: '#30363D',
  text: '#FFFFFF',
  muted: '#8B949E',
  accent: '#2EA043', // solid green for primary actions
  accentHover: '#3FB950',
  danger: '#F85149',
  dangerHover: '#FF6B62',
  emerald: '#3FB950',
  rose: '#F85149',
};

function shortAddress(address: string): string {
  if (!address || address.length < 12) return address || '—';
  return `${address.slice(0, 6)}…${address.slice(-4)}`;
}

function formatDate(iso?: string): string {
  if (!iso) return '—';
  try {
    return new Date(iso).toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' });
  } catch {
    return '—';
  }
}

export default function WalletView({
  wallet,
  settings,
  onDeposit,
  onWithdraw,
  onTransferProfit,
  transferringProfit,
  isUpdating,
  themeMode,
  convertAndFormat,
  walletsList,
  onUpdateWalletsList,
  onUpdateSettings,
}: WalletViewProps) {
  const [sortField, setSortField] = useState<'name' | 'address' | 'chain' | 'balance'>('balance');
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('desc');

  const [modalOpen, setModalOpen] = useState<boolean>(false);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [formName, setFormName] = useState<string>('');
  const [formAddress, setFormAddress] = useState<string>('');
  const [formChain, setFormChain] = useState<string>(CHAIN_OPTIONS[0]);
  const [formCreated, setFormCreated] = useState<string>(new Date().toISOString());

  const [depositAmount, setDepositAmount] = useState<string>('');
  const [depositToken, setDepositToken] = useState<string>('USDC');
  const [withdrawAmount, setWithdrawAmount] = useState<string>('');
  const [withdrawToken, setWithdrawToken] = useState<string>('USDC');

  const [formError, setFormError] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const [confirmWithdraw, setConfirmWithdraw] = useState<{ amount: number; token: string } | null>(null);

  const sortedWallets = [...walletsList].sort((a, b) => {
    let c = 0;
    if (sortField === 'name') c = a.name.localeCompare(b.name);
    else if (sortField === 'address') c = a.address.localeCompare(b.address);
    else if (sortField === 'chain') c = a.chain.localeCompare(b.chain);
    else if (sortField === 'balance') c = a.balance - b.balance;
    return sortOrder === 'desc' ? -c : c;
  });

  const handleSort = (field: 'name' | 'address' | 'chain' | 'balance') => {
    if (sortField === field) setSortOrder(sortOrder === 'asc' ? 'desc' : 'asc');
    else { setSortField(field); setSortOrder('desc'); }
  };

  const totalTrackedUsd = walletsList.reduce((sum, w) => sum + (w.balance || 0), 0);
  const activeCount = walletsList.filter((w) => w.isActive).length;
  const sortIndicator = (field: string) => (sortField === field ? (sortOrder === 'asc' ? ' ↑' : ' ↓') : '');

  // ---- Modal (add / edit withdrawal wallet) ----
  const openAddModal = () => {
    setEditingId(null);
    setFormName('');
    setFormAddress('');
    setFormChain(CHAIN_OPTIONS[0]);
    setFormCreated(new Date().toISOString());
    setFormError(null);
    setModalOpen(true);
  };

  const openEditModal = (item: CustomWalletItem) => {
    setEditingId(item.id);
    setFormName(item.name);
    setFormAddress(item.address);
    setFormChain(item.chain);
    setFormCreated(item.createdDate || new Date().toISOString());
    setFormError(null);
    setModalOpen(true);
  };

  const closeModal = () => { setModalOpen(false); setEditingId(null); };

  const saveWallet = () => {
    if (!formName.trim() || !formAddress.trim()) {
      setFormError('Name and address are required.');
      return;
    }
    if (editingId) {
      onUpdateWalletsList((prev) =>
        prev.map((w) =>
          w.id === editingId
            ? { ...w, name: formName.trim(), address: formAddress.trim(), chain: formChain, createdDate: formCreated }
            : w,
        ),
      );
    } else {
      onUpdateWalletsList((prev) => [
        ...prev,
        {
          id: `w-${Date.now()}`,
          name: formName.trim(),
          address: formAddress.trim(),
          chain: formChain,
          balance: 0,
          isActive: true,
          createdDate: new Date().toISOString(),
        } as CustomWalletItem,
      ]);
    }
    closeModal();
  };

  const deleteWallet = (id: string) => {
    const item = walletsList.find((w) => w.id === id);
    if (!item) return;
    if (!window.confirm(`Remove withdrawal wallet "${item.name}"?`)) return;
    onUpdateWalletsList((prev) => prev.filter((w) => w.id !== id));
  };

  const toggleActive = (id: string) => {
    onUpdateWalletsList((prev) =>
      prev.map((w) => (w.id === id ? { ...w, isActive: !w.isActive } : w)),
    );
  };

  // ---- Profit transfer (auto / manual) ----
  const [transferModeLocal, setTransferModeLocal] = useState<'AUTO' | 'MANUAL'>(settings.profitTransferMode);
  useEffect(() => setTransferModeLocal(settings.profitTransferMode), [settings.profitTransferMode]);

  const handleTransferMode = async (mode: 'AUTO' | 'MANUAL') => {
    setTransferModeLocal(mode);
    await onUpdateSettings({ profitTransferMode: mode });
  };

  // ---- Deposit ----
  const handleDepositSubmit = async () => {
    const amount = parseFloat(depositAmount);
    if (!amount || amount <= 0) { setActionError('Enter a valid deposit amount.'); return; }
    setActionError(null);
    const res = await onDeposit(amount, depositToken);
    if (!res.success) setActionError(res.error || 'Deposit failed.');
    else setDepositAmount('');
  };

  // ---- Withdraw (2-step confirmation) ----
  const requestWithdraw = () => {
    const amount = parseFloat(withdrawAmount);
    if (!amount || amount <= 0) { setActionError('Enter a valid withdraw amount.'); return; }
    setActionError(null);
    setConfirmWithdraw({ amount, token: withdrawToken });
  };

  const confirmWithdrawSubmit = async () => {
    if (!confirmWithdraw) return;
    const res = await onWithdraw(confirmWithdraw.amount, confirmWithdraw.token);
    if (!res.success) setActionError(res.error || 'Withdrawal failed.');
    else setWithdrawAmount('');
    setConfirmWithdraw(null);
  };

  const inputClass = 'w-full px-3 py-2.5 rounded-lg text-sm font-mono outline-none bg-[#21262D] border border-[#30363D] text-white placeholder:text-[#6E7681] focus:border-[#2EA043] transition-colors';
  const labelClass = 'block text-[10px] uppercase tracking-wider text-[#8B949E] mb-1.5';
  const cardClass = 'bg-[#161B22] border border-[#30363D] rounded-xl p-5';
  const btnPrimary = 'flex items-center justify-center gap-1.5 px-4 py-2.5 rounded-lg text-[13px] font-bold bg-[#2EA043] hover:bg-[#3FB950] text-white transition-colors disabled:opacity-50 disabled:cursor-not-allowed';
  const btnRose = 'flex items-center justify-center gap-1.5 px-4 py-2.5 rounded-lg text-[13px] font-bold bg-[#F85149] hover:bg-[#FF6B62] text-white transition-colors disabled:opacity-50 disabled:cursor-not-allowed';

  return (
    <div className="space-y-6 animate-fadeIn" id="wallet-view" style={{ color: COLORS.text }}>
      {/* Header */}
      <div className="flex items-center justify-between flex-wrap gap-3">
        <div className="flex items-center space-x-3">
          <div className="p-2.5 rounded-xl" style={{ background: 'rgba(46,160,67,0.15)', border: '1px solid rgba(46,160,67,0.3)' }}>
            <Wallet className="h-5 w-5" style={{ color: COLORS.accent }} />
          </div>
          <div>
            <h1 className="text-xl font-extrabold tracking-tight" style={{ color: COLORS.text }}>Withdrawal Wallets</h1>
            <p className="text-xs mt-0.5" style={{ color: COLORS.muted }}>
              {walletsList.length} tracked · {activeCount} active · {convertAndFormat(totalTrackedUsd)} total
            </p>
          </div>
        </div>
        <button onClick={openAddModal} className="flex items-center gap-1.5 px-4 py-2 rounded-lg text-[13px] font-bold text-white transition-colors" style={{ background: COLORS.accent, border: 'none' }}>
          <Plus className="h-4 w-4" /> Add Wallet
        </button>
      </div>

      {/* Execution Wallet — separated, read-only, backend-keyed */}
      <div className={cardClass} style={{ borderColor: 'rgba(46,160,67,0.35)' }}>
        <div className="flex items-center justify-between flex-wrap gap-3">
          <div className="flex items-center space-x-2">
            <ShieldCheck className="h-4 w-4" style={{ color: COLORS.accent }} />
            <span className="text-[11px] font-extrabold tracking-wider uppercase" style={{ color: COLORS.text }}>
              Execution Wallet
            </span>
            <span className="text-[10px] font-mono px-2 py-0.5 rounded-full" style={{ background: 'rgba(46,160,67,0.12)', color: COLORS.accent }}>
              Vault-managed · key never exposed
            </span>
          </div>
          <div className="text-right">
            <div className="text-lg font-bold font-mono" style={{ color: COLORS.text }}>{convertAndFormat(settings.accumulatedProfitsUsd)}</div>
            <div className="text-[10px]" style={{ color: COLORS.muted }}>Accumulated profit</div>
          </div>
        </div>
        <div className="mt-3 pt-3 flex items-center justify-between flex-wrap gap-2" style={{ borderTop: '1px solid #21262D' }}>
          <span className="text-[11px] font-mono" style={{ color: COLORS.muted }}>
            {shortAddress(wallet?.address || settings.ownerWalletAddress || '0x0000…0000')}
          </span>
          <span className="text-[10px] font-mono" style={{ color: COLORS.muted }}>{wallet?.network || settings.selectedNetwork || 'Arbitrum Mainnet'}</span>
        </div>
      </div>

      {/* Profit Transfer Controls */}
      <div className={cardClass}>
        <div className="flex items-center justify-between flex-wrap gap-3">
          <div className="flex items-center space-x-2">
            <Send className="h-4 w-4" style={{ color: COLORS.accent }} />
            <span className="text-[11px] font-extrabold tracking-wider uppercase" style={{ color: COLORS.text }}>Profit Transfer</span>
            <span className="text-[10px] font-mono" style={{ color: COLORS.muted }}>
              min {convertAndFormat(settings.profitTransferMinThresholdUsd)}
            </span>
          </div>
          <div className="flex items-center gap-2">
            <button
              onClick={() => handleTransferMode('AUTO')}
              className="px-3 py-1.5 rounded-lg text-[11px] font-mono font-bold uppercase transition-colors cursor-pointer"
              style={transferModeLocal === 'AUTO'
                ? { background: COLORS.accent, color: '#fff' }
                : { background: '#21262D', color: COLORS.muted, border: '1px solid #30363D' }}
            >
              Auto
            </button>
            <button
              onClick={() => handleTransferMode('MANUAL')}
              className="px-3 py-1.5 rounded-lg text-[11px] font-mono font-bold uppercase transition-colors cursor-pointer"
              style={transferModeLocal === 'MANUAL'
                ? { background: COLORS.accent, color: '#fff' }
                : { background: '#21262D', color: COLORS.muted, border: '1px solid #30363D' }}
            >
              Manual
            </button>
            <button
              onClick={() => onTransferProfit()}
              disabled={transferringProfit}
              className="flex items-center gap-1.5 px-4 py-1.5 rounded-lg text-[12px] font-bold transition-colors"
              style={transferringProfit
                ? { background: '#21262D', color: COLORS.muted, cursor: 'not-allowed' }
                : { background: COLORS.accent, color: '#fff' }}
            >
              {transferringProfit ? <Loader2 className="h-3.5 w-3.5 animate-spin" /> : <Zap className="h-3.5 w-3.5" />}
              {transferringProfit ? 'Transferring…' : 'Transfer Profit'}
            </button>
          </div>
        </div>
        <p className="text-[10px] mt-2.5" style={{ color: COLORS.muted }}>
          Sweeps accumulated profit above the minimum threshold to the owner wallet. Keys remain vault-managed and are never shown.
        </p>
      </div>

      {/* Deposit / Withdraw */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-5">
        <div className={cardClass}>
          <div className="flex items-center space-x-2 mb-4">
            <ArrowDownToLine className="h-4 w-4" style={{ color: COLORS.emerald }} />
            <span className="text-[11px] font-extrabold tracking-wider uppercase" style={{ color: COLORS.text }}>Deposit</span>
          </div>
          <div className="flex items-end gap-2">
            <div className="flex-1">
              <label className={labelClass}>Amount</label>
              <input type="number" value={depositAmount} onChange={(e) => setDepositAmount(e.target.value)} placeholder="0.00" className={inputClass} />
            </div>
            <select value={depositToken} onChange={(e) => setDepositToken(e.target.value)} className={inputClass} style={{ width: '90px' }}>
              {TOKENS.map((t) => <option key={t} value={t}>{t}</option>)}
            </select>
            <button onClick={handleDepositSubmit} disabled={isUpdating} className={btnPrimary}>
              Deposit
            </button>
          </div>
        </div>

        <div className={cardClass}>
          <div className="flex items-center space-x-2 mb-4">
            <ArrowUpFromLine className="h-4 w-4" style={{ color: COLORS.rose }} />
            <span className="text-[11px] font-extrabold tracking-wider uppercase" style={{ color: COLORS.text }}>Withdraw</span>
          </div>
          {!confirmWithdraw ? (
            <div className="flex items-end gap-2">
              <div className="flex-1">
                <label className={labelClass}>Amount</label>
                <input type="number" value={withdrawAmount} onChange={(e) => setWithdrawAmount(e.target.value)} placeholder="0.00" className={inputClass} />
              </div>
              <select value={withdrawToken} onChange={(e) => setWithdrawToken(e.target.value)} className={inputClass} style={{ width: '90px' }}>
                {TOKENS.map((t) => <option key={t} value={t}>{t}</option>)}
              </select>
              <button onClick={requestWithdraw} disabled={isUpdating} className={btnRose}>
                Withdraw
              </button>
            </div>
          ) : (
            <div className="space-y-3">
              <p className="text-[11px]" style={{ color: COLORS.muted }}>
                Confirm withdrawal of <span className="font-mono font-bold" style={{ color: COLORS.text }}>{confirmWithdraw.amount} {confirmWithdraw.token}</span> to
                an active withdrawal wallet? This action is recorded.
              </p>
              <div className="flex gap-2">
                <button onClick={() => setConfirmWithdraw(null)} className="flex-1 px-4 py-2.5 rounded-lg text-[13px] font-bold" style={{ background: '#21262D', color: COLORS.muted, border: '1px solid #30363D' }}>
                  Cancel
                </button>
                <button onClick={confirmWithdrawSubmit} className={btnRose + ' flex-1'}>
                  <CheckCircle2 className="h-4 w-4" /> Confirm Withdraw
                </button>
              </div>
            </div>
          )}
        </div>
      </div>

      {actionError && (
        <div className="p-3 rounded-lg text-[12px] font-mono" style={{ border: '1px solid rgba(248,81,73,0.4)', background: 'rgba(248,81,73,0.1)', color: '#FF9D94' }}>
          {actionError}
        </div>
      )}

      {/* Withdrawal Wallets Table */}
      <div className="bg-[#161B22] border border-[#30363D] rounded-xl p-5">
        <div className="flex items-center justify-between mb-4">
          <h3 className="font-bold text-base" style={{ color: COLORS.text }}>Withdrawal Wallets</h3>
          <span className="text-[10px] font-mono" style={{ color: COLORS.muted }}>{walletsList.length} entries</span>
        </div>

        <div className="overflow-x-auto rounded-lg" style={{ border: '1px solid #21262D' }}>
          <table className="w-full text-left text-[12px] border-collapse">
            <thead>
              <tr style={{ background: '#0D1117' }}>
                <th onClick={() => handleSort('name')} className="py-3 px-4 font-bold cursor-pointer hover:text-white select-none" style={{ color: COLORS.muted }}>Name{sortIndicator('name')}</th>
                <th onClick={() => handleSort('address')} className="py-3 px-4 font-bold cursor-pointer hover:text-white select-none" style={{ color: COLORS.muted }}>Address{sortIndicator('address')}</th>
                <th onClick={() => handleSort('chain')} className="py-3 px-4 font-bold cursor-pointer hover:text-white select-none" style={{ color: COLORS.muted }}>Chain{sortIndicator('chain')}</th>
                <th onClick={() => handleSort('balance')} className="py-3 px-4 font-bold cursor-pointer hover:text-white select-none" style={{ color: COLORS.muted }}>Balance{sortIndicator('balance')}</th>
                <th className="py-3 px-4 font-bold" style={{ color: COLORS.muted }}>Added</th>
                <th className="py-3 px-4 font-bold text-right" style={{ color: COLORS.muted }}>Status</th>
                <th className="py-3 px-4 font-bold text-right" style={{ color: COLORS.muted }}>Actions</th>
              </tr>
            </thead>
            <tbody>
              {sortedWallets.length === 0 ? (
                <tr>
                  <td colSpan={7} className="py-10 text-center font-mono text-[12px] animate-pulse" style={{ color: COLORS.muted }}>
                    No withdrawal wallets yet. Click “Add Wallet” to get started.
                  </td>
                </tr>
              ) : (
                sortedWallets.map((w) => (
                  <tr key={w.id} className="border-t" style={{ borderColor: '#21262D' }}>
                    <td className="py-3 px-4">
                      <div className="flex items-center gap-2">
                        <span className="text-[10px]" style={{ color: w.isActive ? COLORS.accent : COLORS.muted }}>{w.isActive ? '●' : '○'}</span>
                        <span className="font-semibold" style={{ color: COLORS.text }}>{w.name}</span>
                      </div>
                    </td>
                    <td className="py-3 px-4 font-mono" style={{ color: COLORS.muted }}>{shortAddress(w.address)}</td>
                    <td className="py-3 px-4"><span className="px-2 py-0.5 rounded-full text-[10px] font-mono" style={{ background: '#21262D', color: COLORS.muted }}>{w.chain}</span></td>
                    <td className="py-3 px-4 font-mono font-bold" style={{ color: COLORS.emerald }}>{convertAndFormat(w.balance)}</td>
                    <td className="py-3 px-4 font-mono text-[11px]" style={{ color: COLORS.muted }}>{formatDate(w.createdDate)}</td>
                    <td className="py-3 px-4 text-right">
                      <span className="px-2 py-0.5 rounded-full text-[10px] font-mono" style={{ background: w.isActive ? 'rgba(46,160,67,0.12)' : 'rgba(139,148,158,0.12)', color: w.isActive ? COLORS.accent : COLORS.muted }}>
                        {w.isActive ? 'ACTIVE' : 'PAUSED'}
                      </span>
                    </td>
                    <td className="py-3 px-4">
                      <div className="flex items-center justify-end gap-1.5">
                        <button
                          onClick={() => toggleActive(w.id)}
                          className="px-2.5 py-1 rounded-lg text-[10px] font-bold transition-colors"
                          style={{ background: '#21262D', color: COLORS.muted, border: '1px solid #30363D' }}
                        >
                          {w.isActive ? 'Pause' : 'Activate'}
                        </button>
                        <button onClick={() => openEditModal(w)} className="p-1.5 rounded-lg transition-colors" style={{ color: COLORS.muted }} title="Edit" onMouseEnter={(e) => (e.currentTarget.style.color = COLORS.accent)} onMouseLeave={(e) => (e.currentTarget.style.color = COLORS.muted)}>
                          <Pencil className="h-3.5 w-3.5" />
                        </button>
                        <button onClick={() => deleteWallet(w.id)} className="p-1.5 rounded-lg transition-colors" style={{ color: COLORS.muted }} title="Delete" onMouseEnter={(e) => (e.currentTarget.style.color = COLORS.rose)} onMouseLeave={(e) => (e.currentTarget.style.color = COLORS.muted)}>
                          <Trash2 className="h-3.5 w-3.5" />
                        </button>
                      </div>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* Add / Edit Modal */}
      {modalOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4" style={{ background: 'rgba(0,0,0,0.6)', backdropFilter: 'blur(4px)' }} onClick={closeModal}>
          <div className="w-full max-w-md bg-[#161B22] border border-[#30363D] rounded-xl p-6" onClick={(e) => e.stopPropagation()} id="wallet-modal">
            <h3 className="font-bold text-base mb-5" style={{ color: COLORS.text }}>{editingId ? 'Edit Withdrawal Wallet' : 'Add Withdrawal Wallet'}</h3>

            <label className={labelClass}>Name</label>
            <input value={formName} onChange={(e) => setFormName(e.target.value)} placeholder="Treasury Wallet" className={inputClass + ' mb-4'} />

            <label className={labelClass}>Address</label>
            <input value={formAddress} onChange={(e) => setFormAddress(e.target.value)} placeholder="0x…" className={inputClass + ' mb-4'} />

            <label className={labelClass}>Chain</label>
            <select value={formChain} onChange={(e) => setFormChain(e.target.value)} className={inputClass + ' mb-4'}>
              {CHAIN_OPTIONS.map((c) => <option key={c} value={c}>{c}</option>)}
            </select>

            {formError && (
              <div className="mb-4 p-2.5 rounded-lg text-[11px] font-mono" style={{ border: '1px solid rgba(248,81,73,0.4)', background: 'rgba(248,81,73,0.1)', color: '#FF9D94' }}>
                {formError}
              </div>
            )}

            <div className="flex justify-end gap-2 mt-2">
              <button onClick={closeModal} className="px-4 py-2 rounded-lg text-[13px] font-bold" style={{ background: '#21262D', color: COLORS.muted, border: '1px solid #30363D' }}>
                Cancel
              </button>
              <button onClick={saveWallet} className="px-4 py-2 rounded-lg text-[13px] font-bold text-white" style={{ background: COLORS.accent, border: 'none' }}>
                Save
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
