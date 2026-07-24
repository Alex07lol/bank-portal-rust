const { invoke } = window.__TAURI__.core;

// Memory Cache State
let state = {
  accounts: [],
  transactions: [],
  loans: [],
  fds: [],
  metrics: null,
  activeScreen: 'dashboard'
};

// Global helpers for opening/closing modals (accessible inside HTML onclick attributes)
window.showModal = function(id) {
  const modal = document.getElementById(id);
  if (modal) {
    modal.classList.add('active');
    
    // Clear inputs inside modal
    if (id === 'modal-pin-confirm') {
      document.getElementById('pin-confirm-input').value = '';
      document.getElementById('pin-confirm-input').focus();
    }
  }
};

window.closeModal = function(id) {
  const modal = document.getElementById(id);
  if (modal) modal.classList.remove('active');
};

// Formatting helpers
function formatCurrency(amount) {
  return new Intl.NumberFormat('en-IN', {
    style: 'currency',
    currency: 'INR',
    maximumFractionDigits: 2
  }).format(amount).replace('INR', 'Rs.');
}

function formatDate(timestampStr) {
  if (!timestampStr) return '-';
  try {
    const dt = new Date(timestampStr.replace(' ', 'T'));
    if (isNaN(dt.getTime())) return timestampStr;
    return dt.toLocaleString('en-IN', {
      day: '2-digit',
      month: 'short',
      year: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit'
    });
  } catch (e) {
    return timestampStr;
  }
}

// Toast alerts helper
function showToast(message, type = 'info') {
  let container = document.getElementById('toast-container');
  if (!container) {
    container = document.createElement('div');
    container.id = 'toast-container';
    container.style.cssText = 'position: fixed; bottom: 24px; right: 24px; display: flex; flex-direction: column; gap: 8px; z-index: 9999; pointer-events: none;';
    document.body.appendChild(container);
  }
  
  const toast = document.createElement('div');
  toast.style.cssText = `
    padding: 14px 20px;
    background-color: var(--bg-secondary);
    border-radius: 12px;
    color: var(--text-primary);
    font-size: 13.5px;
    font-weight: 600;
    box-shadow: 0 10px 30px rgba(0,0,0,0.5);
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 320px;
    max-width: 480px;
    pointer-events: auto;
    opacity: 0;
    transform: translateY(20px);
    transition: opacity 0.3s ease, transform 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
  `;
  
  let borderColor = 'var(--color-primary)';
  let icon = 'ℹ️';
  if (type === 'success') {
    borderColor = 'var(--color-success)';
    icon = '✅';
  } else if (type === 'error') {
    borderColor = 'var(--color-danger)';
    icon = '❌';
  } else if (type === 'warning') {
    borderColor = 'var(--color-warning)';
    icon = '⚠️';
  }
  
  toast.style.borderLeft = `4px solid ${borderColor}`;
  toast.innerHTML = `<span style="font-size:16px">${icon}</span><span style="flex-grow:1; line-height:1.4">${message}</span>`;
  
  container.appendChild(toast);
  
  setTimeout(() => {
    toast.style.opacity = '1';
    toast.style.transform = 'translateY(0)';
  }, 10);
  
  setTimeout(() => {
    toast.style.opacity = '0';
    toast.style.transform = 'translateY(-10px)';
    setTimeout(() => {
      toast.remove();
    }, 300);
  }, 4500);
}

// Digital clock ticking
function initClock() {
  const clockEl = document.getElementById('digital-clock');
  if (!clockEl) return;
  
  setInterval(() => {
    const now = new Date();
    clockEl.innerHTML = `
      <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor" style="width:14px; height:14px">
        <path stroke-linecap="round" stroke-linejoin="round" d="M12 6v6h4.5m4.5 0a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z" />
      </svg>
      ${now.toLocaleTimeString('en-IN')}
    `;
  }, 1000);
}

// Draw demographics circular chart
function renderGenderPieChart(male, female, other) {
  const total = male + female + other;
  const pieChart = document.getElementById("gender-pie-chart");
  if (!pieChart) return;
  
  if (total === 0) {
    pieChart.innerHTML = '<circle cx="16" cy="16" r="12" fill="none" stroke="var(--text-muted)" stroke-width="4"></circle>';
    return;
  }
  
  const pMale = (male / total) * 100;
  const pFemale = (female / total) * 100;
  
  const r = 12;
  const circ = 2 * Math.PI * r; // 75.4
  
  let html = '';
  let offset = 0;
  
  // Male (Blue)
  if (male > 0) {
    const dash = (male / total) * circ;
    html += `<circle cx="16" cy="16" r="${r}" fill="none" stroke="var(--color-primary)" stroke-width="4" stroke-dasharray="${dash} ${circ}" stroke-dashoffset="${-offset}" transform="rotate(-90 16 16)"></circle>`;
    offset += dash;
  }
  
  // Female (Purple/Teal/Rose)
  if (female > 0) {
    const dash = (female / total) * circ;
    html += `<circle cx="16" cy="16" r="${r}" fill="none" stroke="var(--color-info)" stroke-width="4" stroke-dasharray="${dash} ${circ}" stroke-dashoffset="${-offset}" transform="rotate(-90 16 16)"></circle>`;
    offset += dash;
  }
  
  // Other (Amber/Orange)
  if (other > 0) {
    const dash = (other / total) * circ;
    html += `<circle cx="16" cy="16" r="${r}" fill="none" stroke="var(--color-warning)" stroke-width="4" stroke-dasharray="${dash} ${circ}" stroke-dashoffset="${-offset}" transform="rotate(-90 16 16)"></circle>`;
  }
  
  pieChart.innerHTML = html;
}

// Fetch database records from Rust backend
async function loadData() {
  try {
    state.accounts = await invoke('get_accounts');
    state.transactions = await invoke('get_transactions');
    state.loans = await invoke('get_loans');
    state.fds = await invoke('get_fixed_deposits');
    state.metrics = await invoke('get_bank_metrics');
    
    // Refresh screens
    updateMetrics();
    renderDashboard();
    renderAccounts();
    renderTransactions();
    renderLoans();
    renderFixedDeposits();
    populateDropdowns();
  } catch (err) {
    showToast(`Error fetching bank database: ${err}`, 'error');
  }
}

// Update the overview metrics
function updateMetrics() {
  if (!state.metrics) return;
  const m = state.metrics;
  
  document.getElementById('metric-assets').textContent = formatCurrency(m.total_assets);
  document.getElementById('metric-accounts').textContent = m.total_accounts;
  document.getElementById('metric-avg-balance').textContent = `Avg balance: ${formatCurrency(m.avg_balance)}`;
  document.getElementById('metric-loans').textContent = formatCurrency(m.total_loan_balance);
  document.getElementById('metric-loans-count').textContent = `${m.active_loans_count} active credit disbursements`;
  document.getElementById('metric-fds').textContent = formatCurrency(m.total_fd_balance);
  document.getElementById('metric-fds-count').textContent = `${m.active_fds_count} active deposit contracts`;
}

// Render Dashboard Panel
function renderDashboard() {
  if (!state.metrics) return;
  const m = state.metrics;
  
  // Demographic chart & counts
  document.getElementById('gender-count-male').textContent = m.gender_stats.male;
  document.getElementById('gender-count-female').textContent = m.gender_stats.female;
  document.getElementById('gender-count-other').textContent = m.gender_stats.other;
  renderGenderPieChart(m.gender_stats.male, m.gender_stats.female, m.gender_stats.other);
  
  // Top Depositor Spotlight
  if (m.total_accounts > 0) {
    document.getElementById('top-holder-name').textContent = m.top_holder_name;
    document.getElementById('top-holder-avatar').textContent = m.top_holder_name.charAt(0);
    document.getElementById('top-holder-balance').textContent = formatCurrency(m.top_holder_bal);
    
    // Find account number
    const holderAcc = state.accounts.find(a => a.name === m.top_holder_name);
    document.getElementById('top-holder-no').textContent = holderAcc ? `Account: #${holderAcc.account_no}` : 'Asset Whale';
  } else {
    document.getElementById('top-holder-name').textContent = 'None';
    document.getElementById('top-holder-avatar').textContent = '?';
    document.getElementById('top-holder-balance').textContent = formatCurrency(0);
    document.getElementById('top-holder-no').textContent = 'Open accounts to seed';
  }
  
  // Recent Transactions (limit to 6 rows)
  const txTable = document.getElementById('dashboard-tx-table');
  txTable.innerHTML = '';
  
  const recent = state.transactions.slice(0, 6);
  if (recent.length === 0) {
    txTable.innerHTML = `<tr><td colspan="5" style="text-align:center; color:var(--text-muted)">No recorded transactions</td></tr>`;
    return;
  }
  
  recent.forEach(tx => {
    const isCredit = tx.type === 'Deposit' || tx.type === 'Transfer In' || tx.type === 'Loan Disbursement' || tx.type === 'FD Maturity';
    const sign = isCredit ? '+' : '-';
    const badgeClass = isCredit ? 'badge-tx-in' : 'badge-tx-out';
    
    txTable.innerHTML += `
      <tr>
        <td style="color:var(--text-secondary); font-size:12px">${formatDate(tx.timestamp).split(',')[0]}</td>
        <td style="font-weight:600">${tx.name || 'Account ' + tx.account_no}</td>
        <td><span class="badge ${badgeClass}">${tx.type}</span></td>
        <td style="font-weight:700; color:${isCredit ? 'var(--color-success)' : 'var(--color-danger)'}">${sign} ${formatCurrency(tx.amount)}</td>
        <td style="color:var(--text-secondary); font-size:12px">${tx.description || '-'}</td>
      </tr>
    `;
  });
}

// Render Accounts Grid (Custom styled Credit Cards)
function renderAccounts() {
  const container = document.getElementById('accounts-cards-container');
  container.innerHTML = '';
  
  const searchQuery = document.getElementById('input-search-accounts').value.toLowerCase();
  const filtered = state.accounts.filter(acc => {
    return acc.name.toLowerCase().includes(searchQuery) ||
           acc.account_no.toString().includes(searchQuery) ||
           acc.phone.includes(searchQuery) ||
           acc.email.toLowerCase().includes(searchQuery);
  });
  
  if (filtered.length === 0) {
    container.innerHTML = `<div style="grid-column:1/-1; text-align:center; padding:40px; color:var(--text-muted)">No bank account matches found</div>`;
    return;
  }
  
  filtered.forEach(acc => {
    let statusClass = 'badge-success';
    if (acc.status === 'Suspended') statusClass = 'badge-warning';
    if (acc.status === 'Frozen') statusClass = 'badge-danger';
    
    const cardEl = document.createElement('div');
    cardEl.className = 'account-card-glow';
    cardEl.innerHTML = `
      <div>
        <div class="acc-card-chip"></div>
        <div class="acc-card-num">${acc.account_no}</div>
        <div class="acc-card-name">${acc.name}</div>
        <div style="font-size:11px; color:var(--text-muted); margin-top:2px">${acc.email} | ${acc.phone}</div>
      </div>
      
      <div>
        <span class="badge ${statusClass} acc-card-type-badge">${acc.account_type}</span>
        <div class="acc-card-balance">${formatCurrency(acc.balance)}</div>
        <div class="acc-card-actions">
          <button class="btn btn-sm" style="padding:4px 10px; font-size:11px" onclick="editAccountTrigger(${acc.account_no})">Edit</button>
          <button class="btn btn-sm" style="padding:4px 10px; font-size:11px" onclick="changePinTrigger(${acc.account_no})">PIN</button>
          <button class="btn btn-sm btn-danger" style="padding:4px 10px; font-size:11px" onclick="deleteAccountTrigger(${acc.account_no})">Close</button>
        </div>
      </div>
    `;
    container.appendChild(cardEl);
  });
}

// Render Transactions History Ledger
function renderTransactions() {
  const tableBody = document.getElementById('full-transactions-table');
  tableBody.innerHTML = '';
  
  const search = document.getElementById('input-filter-transactions').value.toLowerCase();
  const activePill = document.querySelector('#tx-type-filters .pill-filter.active').dataset.type;
  
  const filtered = state.transactions.filter(tx => {
    // Text search filter
    const matchesText = tx.account_no.toString().includes(search) || 
                        (tx.name && tx.name.toLowerCase().includes(search)) ||
                        (tx.description && tx.description.toLowerCase().includes(search));
    
    // Type Filter
    let matchesType = true;
    if (activePill !== 'all') {
      if (activePill === 'Loan') {
        matchesType = tx.type.includes('Loan');
      } else if (activePill === 'FD') {
        matchesType = tx.type.includes('FD');
      } else {
        matchesType = tx.type.includes(activePill);
      }
    }
    
    return matchesText && matchesType;
  });
  
  if (filtered.length === 0) {
    tableBody.innerHTML = `<tr><td colspan="8" style="text-align:center; color:var(--text-muted)">No transaction records match criteria</td></tr>`;
    return;
  }
  
  filtered.forEach(tx => {
    const isCredit = tx.type === 'Deposit' || tx.type === 'Transfer In' || tx.type === 'Loan Disbursement' || tx.type === 'FD Maturity';
    const sign = isCredit ? '+' : '-';
    const badgeClass = isCredit ? 'badge-tx-in' : 'badge-tx-out';
    
    tableBody.innerHTML += `
      <tr>
        <td>#${tx.transaction_id}</td>
        <td>${formatDate(tx.timestamp)}</td>
        <td style="font-weight:600">${tx.account_no}</td>
        <td>${tx.name || '-'}</td>
        <td><span class="badge ${badgeClass}">${tx.type}</span></td>
        <td style="font-weight:700; color:${isCredit ? 'var(--color-success)' : 'var(--color-danger)'}">${sign} ${formatCurrency(tx.amount)}</td>
        <td>${tx.target_account_no || '-'}</td>
        <td style="font-size:12px; color:var(--text-secondary)">${tx.description || '-'}</td>
      </tr>
    `;
  });
}

// Render Loans Department Screen
function renderLoans() {
  const ledger = document.getElementById('loans-ledger-table');
  ledger.innerHTML = '';
  
  if (state.loans.length === 0) {
    ledger.innerHTML = `<tr><td colspan="7" style="text-align:center; color:var(--text-muted)">No active loan accounts</td></tr>`;
    return;
  }
  
  state.loans.forEach(loan => {
    const percentPaid = ((loan.principal - loan.remaining_balance) / loan.principal) * 100;
    
    let statusClass = 'badge-warning';
    if (loan.status === 'Paid') statusClass = 'badge-success';
    if (loan.status === 'Defaulted') statusClass = 'badge-danger';
    
    let actionBtn = '';
    if (loan.status === 'Approved' && loan.remaining_balance > 0) {
      actionBtn = `<button class="btn btn-sm btn-success" style="padding:4px 8px; font-size:11px" onclick="payLoanTrigger(${loan.loan_id}, ${loan.remaining_balance})">Pay Installment</button>`;
    }
    
    ledger.innerHTML += `
      <tr>
        <td>#${loan.loan_id}</td>
        <td>
          <div style="font-weight:600">${loan.name}</div>
          <div style="font-size:11px; color:var(--text-muted)">Acc: #${loan.account_no}</div>
        </td>
        <td>${formatCurrency(loan.principal)}</td>
        <td style="font-weight:600">${loan.interest_rate}%</td>
        <td>
          <div style="font-weight:700">${formatCurrency(loan.remaining_balance)}</div>
          <div class="progress-bar-container">
            <div class="progress-bar blue" style="width: ${percentPaid}%"></div>
          </div>
          <div style="font-size:10px; color:var(--text-muted)">Repaid: ${percentPaid.toFixed(1)}%</div>
        </td>
        <td><span class="badge ${statusClass}">${loan.status}</span></td>
        <td>${actionBtn || '-'}</td>
      </tr>
    `;
  });
}

// Render Fixed Deposits Screen
function renderFixedDeposits() {
  const ledger = document.getElementById('fds-ledger-table');
  ledger.innerHTML = '';
  
  if (state.fds.length === 0) {
    ledger.innerHTML = `<tr><td colspan="9" style="text-align:center; color:var(--text-muted)">No active Fixed Deposits</td></tr>`;
    return;
  }
  
  state.fds.forEach(fd => {
    let statusClass = 'badge-success';
    if (fd.status === 'Matured') statusClass = 'badge-info';
    if (fd.status === 'Withdrawn') statusClass = 'badge-danger';
    
    let action = '-';
    if (fd.status === 'Active') {
      action = `<button class="btn btn-sm btn-primary" onclick="matureFdTrigger(${fd.fd_id})">Simulate Maturity</button>`;
    }
    
    ledger.innerHTML += `
      <tr>
        <td>#${fd.fd_id}</td>
        <td>
          <div style="font-weight:600">${fd.name}</div>
          <div style="font-size:11px; color:var(--text-muted)">Acc: #${fd.account_no}</div>
        </td>
        <td style="font-weight:600">${formatCurrency(fd.amount)}</td>
        <td>${fd.interest_rate}%</td>
        <td>${fd.duration_months} Mos</td>
        <td style="font-weight:700; color:var(--color-success)">${formatCurrency(fd.maturity_amount)}</td>
        <td style="font-size:12px; color:var(--text-secondary)">${formatDate(fd.maturity_date).split(',')[0]}</td>
        <td><span class="badge ${statusClass}">${fd.status}</span></td>
        <td>${action}</td>
      </tr>
    `;
  });
}

// Populate Selector lists in forms dynamically
function populateDropdowns() {
  const loanSelect = document.getElementById('select-loan-account');
  const fdSelect = document.getElementById('select-fd-account');
  const depSelect = document.getElementById('deposit-account');
  const withSelect = document.getElementById('withdraw-account');
  const trSelect = document.getElementById('transfer-sender');
  
  const options = state.accounts
    .filter(a => a.status === 'Active')
    .map(a => `<option value="${a.account_no}">${a.name} (#${a.account_no}) - Bal: ${formatCurrency(a.balance)}</option>`)
    .join('');
    
  if (loanSelect) loanSelect.innerHTML = options;
  if (fdSelect) fdSelect.innerHTML = options;
  if (depSelect) {
    // Deposits can go to any account, even frozen ones (sometimes)
    depSelect.innerHTML = state.accounts.map(a => `<option value="${a.account_no}">${a.name} (#${a.account_no})</option>`).join('');
  }
  if (withSelect) withSelect.innerHTML = options;
  if (trSelect) trSelect.innerHTML = options;
}

// TRIGGERS (Open Form Modals)
window.editAccountTrigger = function(accountNo) {
  const acc = state.accounts.find(a => a.account_no === accountNo);
  if (!acc) return;
  
  document.getElementById('modal-account-title').textContent = 'Modify Bank Account';
  document.getElementById('form-account-id').value = acc.account_no;
  document.getElementById('form-account-name').value = acc.name;
  document.getElementById('form-account-gender').value = acc.gender;
  document.getElementById('form-account-type').value = acc.account_type;
  document.getElementById('form-account-phone').value = acc.phone;
  document.getElementById('form-account-email').value = acc.email;
  
  // Hide initial fields for editing, show status field
  document.getElementById('form-account-balance-group').style.display = 'none';
  document.getElementById('form-account-pin-group').style.display = 'none';
  document.getElementById('form-account-status-group').style.display = 'block';
  document.getElementById('form-account-status').value = acc.status;
  
  showModal('modal-account');
};

window.changePinTrigger = function(accountNo) {
  // Use authorization PIN modal to confirm changing PIN
  document.getElementById('pin-confirm-action').value = 'change-pin-init';
  document.getElementById('pin-confirm-payload').value = JSON.stringify({ account_no: accountNo });
  document.getElementById('pin-confirm-prompt-msg').textContent = 'Enter current 4-Digit PIN to authorize PIN reset:';
  showModal('modal-pin-confirm');
};

window.deleteAccountTrigger = function(accountNo) {
  document.getElementById('pin-confirm-action').value = 'delete-account';
  document.getElementById('pin-confirm-payload').value = JSON.stringify({ account_no: accountNo });
  document.getElementById('pin-confirm-prompt-msg').textContent = `Enter security PIN to confirm closing account #${accountNo}:`;
  showModal('modal-pin-confirm');
};

window.payLoanTrigger = function(loanId, remainingBalance) {
  document.getElementById('pin-confirm-action').value = 'pay-loan-installment-init';
  document.getElementById('pin-confirm-payload').value = JSON.stringify({ loan_id: loanId, remaining: remainingBalance });
  document.getElementById('pin-confirm-prompt-msg').textContent = `Enter account PIN to authorize repayment processing:`;
  showModal('modal-pin-confirm');
};

window.matureFdTrigger = async function(fdId) {
  try {
    await invoke('mature_fixed_deposit', { fdId });
    showToast('Fixed Deposit successfully matured. Funds disbursed back to base savings account.', 'success');
    loadData();
  } catch (err) {
    showToast(`Simulation failed: ${err}`, 'error');
  }
};

// FORM SUBMISSIONS
document.getElementById('form-account').addEventListener('submit', async (e) => {
  e.preventDefault();
  const idVal = document.getElementById('form-account-id').value;
  const name = document.getElementById('form-account-name').value.trim();
  const gender = document.getElementById('form-account-gender').value;
  const type = document.getElementById('form-account-type').value;
  const phone = document.getElementById('form-account-phone').value.trim();
  const email = document.getElementById('form-account-email').value.trim();
  
  try {
    if (idVal) {
      // Update
      const accountNo = parseInt(idVal);
      const acc = state.accounts.find(a => a.account_no === accountNo);
      const balance = acc ? acc.balance : 0;
      const status = document.getElementById('form-account-status').value;
      
      await invoke('update_account', { 
        accountNo, name, gender, phone, email, balance, accountType: type, status 
      });
      showToast('Customer account files updated successfully.', 'success');
    } else {
      // Create
      const balance = parseFloat(document.getElementById('form-account-balance').value);
      const pin = document.getElementById('form-account-pin').value;
      
      const newAccNo = await invoke('create_account', {
        name, gender, phone, email, balance, accountType: type, pin
      });
      showToast(`Account successfully established. Account No: ${newAccNo}`, 'success');
    }
    
    closeModal('modal-account');
    loadData();
  } catch (err) {
    showToast(`Operation rejected: ${err}`, 'error');
  }
});

document.getElementById('form-deposit').addEventListener('submit', async (e) => {
  e.preventDefault();
  const accountNo = parseInt(document.getElementById('deposit-account').value);
  const amount = parseFloat(document.getElementById('deposit-amount').value);
  const description = document.getElementById('deposit-desc').value.trim() || 'Counter Cash Deposit';
  
  try {
    await invoke('deposit', { accountNo, amount, description: description || null });
    showToast(`Deposited ${formatCurrency(amount)} into account #${accountNo}`, 'success');
    closeModal('modal-deposit');
    loadData();
  } catch (err) {
    showToast(`Deposit failed: ${err}`, 'error');
  }
});

document.getElementById('form-withdraw').addEventListener('submit', async (e) => {
  e.preventDefault();
  const accountNo = parseInt(document.getElementById('withdraw-account').value);
  const amount = parseFloat(document.getElementById('withdraw-amount').value);
  const pin = document.getElementById('withdraw-pin').value;
  const description = document.getElementById('withdraw-desc').value.trim() || 'ATM Cash Withdrawal';
  
  try {
    await invoke('withdraw', { accountNo, amount, pin, description: description || null });
    showToast(`Withdrew ${formatCurrency(amount)} from account #${accountNo}`, 'success');
    closeModal('modal-withdraw');
    loadData();
  } catch (err) {
    showToast(`Withdrawal failed: ${err}`, 'error');
  }
});

document.getElementById('form-transfer').addEventListener('submit', async (e) => {
  e.preventDefault();
  const senderNo = parseInt(document.getElementById('transfer-sender').value);
  const receiverNo = parseInt(document.getElementById('transfer-receiver').value);
  const amount = parseFloat(document.getElementById('transfer-amount').value);
  const pin = document.getElementById('transfer-pin').value;
  const description = document.getElementById('transfer-desc').value.trim() || 'Online Transfer';
  
  try {
    const receiverName = await invoke('transfer', { senderNo, receiverNo, amount, pin, description: description || null });
    showToast(`Transferred ${formatCurrency(amount)} to ${receiverName} (#${receiverNo})`, 'success');
    closeModal('modal-transfer');
    loadData();
  } catch (err) {
    showToast(`Transfer failed: ${err}`, 'error');
  }
});

document.getElementById('form-apply-loan').addEventListener('submit', async (e) => {
  e.preventDefault();
  const accountNo = parseInt(document.getElementById('select-loan-account').value);
  const principal = parseFloat(document.getElementById('input-loan-principal').value);
  const rate = parseFloat(document.getElementById('input-loan-rate').value);
  
  try {
    const loanId = await invoke('apply_loan', { accountNo, principal, interestRate: rate });
    showToast(`Loan ID #${loanId} approved. Principal disbursed to borrower balance.`, 'success');
    
    // Clear form
    document.getElementById('input-loan-principal').value = '';
    loadData();
  } catch (err) {
    showToast(`Credit application rejected: ${err}`, 'error');
  }
});

document.getElementById('form-create-fd').addEventListener('submit', async (e) => {
  e.preventDefault();
  const accountNo = parseInt(document.getElementById('select-fd-account').value);
  const amount = parseFloat(document.getElementById('input-fd-amount').value);
  const duration = parseInt(document.getElementById('select-fd-duration').value);
  
  // Prompt PIN to authorize FD creation
  document.getElementById('pin-confirm-action').value = 'create-fd';
  document.getElementById('pin-confirm-payload').value = JSON.stringify({ account_no: accountNo, amount, duration });
  document.getElementById('pin-confirm-prompt-msg').textContent = `Enter security PIN to create FD contract for Rs. ${amount.toLocaleString('en-IN')}:`;
  showModal('modal-pin-confirm');
});

// SECURE PIN CONFIRMATION FORM HANDLER
document.getElementById('form-pin-confirm').addEventListener('submit', async (e) => {
  e.preventDefault();
  const action = document.getElementById('pin-confirm-action').value;
  const payload = JSON.parse(document.getElementById('pin-confirm-payload').value);
  const pin = document.getElementById('pin-confirm-input').value;
  
  try {
    if (action === 'create-fd') {
      await invoke('create_fixed_deposit', { 
        accountNo: payload.account_no, 
        amount: payload.amount, 
        durationMonths: payload.duration, 
        pin 
      });
      showToast('Fixed Deposit established. Principal locked.', 'success');
      closeModal('modal-pin-confirm');
      document.getElementById('input-fd-amount').value = '';
      loadData();
    } 
    else if (action === 'delete-account') {
      // Verify pin first
      const correct = await invoke('verify_pin', { accountNo: payload.account_no, pin });
      if (!correct) {
        showToast('Invalid authorization PIN.', 'error');
        return;
      }
      
      await invoke('delete_account', { accountNo: payload.account_no });
      showToast(`Account #${payload.account_no} closed. Liquid balances returned.`, 'success');
      closeModal('modal-pin-confirm');
      loadData();
    }
    else if (action === 'change-pin-init') {
      // Verify pin
      const correct = await invoke('verify_pin', { accountNo: payload.account_no, pin });
      if (!correct) {
        showToast('Incorrect current PIN.', 'error');
        return;
      }
      
      // Prompt for new PIN (using browser prompt for simplicity here, but styled nicely inside toast)
      const newPin = window.prompt("Enter new 4-Digit PIN:");
      if (newPin) {
        await invoke('change_pin', { accountNo: payload.account_no, oldPin: pin, newPin });
        showToast('Account PIN updated successfully.', 'success');
      }
      closeModal('modal-pin-confirm');
    }
    else if (action === 'pay-loan-installment-init') {
      // Let's ask for the payment amount
      const amtStr = window.prompt(`Pay Loan #${payload.loan_id} (Remaining: Rs. ${payload.remaining.toFixed(2)})\nEnter payment amount (Rs.):`);
      if (amtStr) {
        const amount = parseFloat(amtStr);
        if (isNaN(amount) || amount <= 0) {
          showToast('Payment amount must be a positive number.', 'error');
          return;
        }
        await invoke('pay_loan_installment', { loanId: payload.loan_id, amount, pin });
        showToast('Loan installment payment posted successfully.', 'success');
        closeModal('modal-pin-confirm');
        loadData();
      } else {
        closeModal('modal-pin-confirm');
      }
    }
  } catch (err) {
    showToast(`Authorization error: ${err}`, 'error');
  }
});

// CSV EXPORT LEDGER
document.getElementById('btn-export-csv').addEventListener('click', () => {
  if (state.transactions.length === 0) {
    showToast('Ledger empty. Nothing to export.', 'warning');
    return;
  }
  
  try {
    let csv = 'TX_ID,Timestamp,Account_No,Holder_Name,Type,Amount,Target_Account_No,Description\n';
    state.transactions.forEach(t => {
      csv += `${t.transaction_id},"${t.timestamp}",${t.account_no},"${t.name || ''}","${t.type}",${t.amount},${t.target_account_no || ''},"${t.description || ''}"\n`;
    });
    
    // Create download element
    const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
    const link = document.createElement('a');
    link.href = URL.createObjectURL(blob);
    link.setAttribute('download', `aura_trust_transactions_${Date.now()}.csv`);
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    
    showToast('Transaction history ledger exported to CSV file.', 'success');
  } catch (e) {
    showToast(`Export failed: ${e.message}`, 'error');
  }
});

// ADMIN PANEL BUTTONS
document.getElementById('btn-admin-wipe-transactions').addEventListener('click', async () => {
  if (confirm('CAUTION: Are you sure you want to delete all transaction records? This action is irreversible.')) {
    try {
      await invoke('wipe_transactions');
      showToast('Transaction history ledger successfully cleared.', 'success');
      loadData();
    } catch (err) {
      showToast(`Wipe failed: ${err}`, 'error');
    }
  }
});

document.getElementById('btn-admin-wipe-database').addEventListener('click', async () => {
  if (confirm('CRITICAL CAUTION: Are you sure you want to restore database to factory defaults? All accounts, loans, and fixed deposits will be deleted and reset.')) {
    try {
      await invoke('reset_database');
      showToast('Database successfully reset to default seed values.', 'success');
      loadData();
    } catch (err) {
      showToast(`Database reset failed: ${err}`, 'error');
    }
  }
});

// Screen switcher navigation
function switchScreen(screenId) {
  // Update nav link active state
  document.querySelectorAll('.nav-link').forEach(link => {
    if (link.dataset.screen === screenId) {
      link.classList.add('active');
    } else {
      link.classList.remove('active');
    }
  });
  
  // Show active screen
  document.querySelectorAll('.screen').forEach(screen => {
    if (screen.id === `screen-${screenId}`) {
      screen.classList.add('active');
    } else {
      screen.classList.remove('active');
    }
  });
  
  state.activeScreen = screenId;
  
  // Update header titles
  const titles = {
    dashboard: ['Overview Dashboard', 'Aura Trust Digital Core Node'],
    accounts: ['Customer Accounts', 'Deposit & Portfolio Records'],
    transactions: ['Operations Ledger', 'Historical Transaction Vault'],
    loans: ['Commercial Credit Department', 'Credit Risk & Amortization'],
    'fixed-deposits': ['Investment Operations', 'Time Deposits & APY Accruals'],
    settings: ['Administrative Settings', 'Node Core Configuration']
  };
  
  const [title, subtitle] = titles[screenId] || ['Aura Core', 'Security Environment'];
  document.getElementById('current-page-title').textContent = title;
  document.getElementById('current-page-subtitle').textContent = subtitle;
}

// SETUP LISTENERS
window.addEventListener('DOMContentLoaded', () => {
  initClock();
  
  // Screen Swapping
  document.querySelectorAll('.nav-link').forEach(link => {
    link.addEventListener('click', (e) => {
      const screenId = e.currentTarget.dataset.screen;
      switchScreen(screenId);
    });
  });
  
  // Navigation shortcuts
  document.getElementById('btn-view-all-tx-shortcut').addEventListener('click', () => switchScreen('transactions'));
  document.getElementById('btn-add-account-shortcut').addEventListener('click', () => {
    document.getElementById('modal-account-title').textContent = 'Open New Account';
    document.getElementById('form-account-id').value = '';
    document.getElementById('form-account-name').value = '';
    document.getElementById('form-account-gender').value = 'Male';
    document.getElementById('form-account-type').value = 'Savings';
    document.getElementById('form-account-phone').value = '';
    document.getElementById('form-account-email').value = '';
    document.getElementById('form-account-balance').value = '5000';
    document.getElementById('form-account-pin').value = '1234';
    
    document.getElementById('form-account-balance-group').style.display = 'block';
    document.getElementById('form-account-pin-group').style.display = 'block';
    document.getElementById('form-account-status-group').style.display = 'none';
    
    showModal('modal-account');
  });
  
  document.getElementById('btn-create-account').addEventListener('click', () => {
    document.getElementById('btn-add-account-shortcut').click();
  });
  
  // Search inputs live filter
  document.getElementById('input-search-accounts').addEventListener('input', renderAccounts);
  document.getElementById('input-filter-transactions').addEventListener('input', renderTransactions);
  
  // Transaction type pill filters
  document.querySelectorAll('#tx-type-filters .pill-filter').forEach(pill => {
    pill.addEventListener('click', (e) => {
      document.querySelectorAll('#tx-type-filters .pill-filter').forEach(p => p.classList.remove('active'));
      e.currentTarget.classList.add('active');
      renderTransactions();
    });
  });
  
  // Quick actions
  document.getElementById('action-deposit').addEventListener('click', () => showModal('modal-deposit'));
  document.getElementById('action-withdraw').addEventListener('click', () => showModal('modal-withdraw'));
  document.getElementById('action-transfer').addEventListener('click', () => showModal('modal-transfer'));
  
  // Load initial bank dataset
  loadData();
});
