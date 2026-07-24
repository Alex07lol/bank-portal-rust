// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Account {
    pub account_no: i64,
    pub name: String,
    pub gender: String,
    pub phone: String,
    pub email: String,
    pub balance: f64,
    pub account_type: String,
    pub status: String,
    pub pin: String,
    pub created_at: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Transaction {
    pub transaction_id: i64,
    pub account_no: i64,
    pub name: Option<String>,
    pub r#type: String,
    pub amount: f64,
    pub target_account_no: Option<i64>,
    pub timestamp: String,
    pub description: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct Loan {
    pub loan_id: i64,
    pub account_no: i64,
    pub name: Option<String>,
    pub principal: f64,
    pub interest_rate: f64,
    pub remaining_balance: f64,
    pub status: String,
    pub created_at: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct FixedDeposit {
    pub fd_id: i64,
    pub account_no: i64,
    pub name: Option<String>,
    pub amount: f64,
    pub interest_rate: f64,
    pub duration_months: i64,
    pub maturity_amount: f64,
    pub status: String,
    pub created_at: String,
    pub maturity_date: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct GenderStats {
    pub male: i64,
    pub female: i64,
    pub other: i64,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct BankMetrics {
    pub total_accounts: i64,
    pub total_assets: f64,
    pub total_transactions: i64,
    pub avg_balance: f64,
    pub top_holder_name: String,
    pub top_holder_bal: f64,
    pub gender_stats: GenderStats,
    pub active_loans_count: i64,
    pub total_loan_balance: f64,
    pub active_fds_count: i64,
    pub total_fd_balance: f64,
}

fn get_db_path() -> std::path::PathBuf {
    let mut path = if let Some(home) = std::env::var_os("HOME") {
        std::path::PathBuf::from(home)
    } else if let Some(home) = std::env::var_os("USERPROFILE") {
        std::path::PathBuf::from(home)
    } else {
        std::path::PathBuf::from(".")
    };
    
    path.push("Desktop");
    path.push("py");
    path.push("bank_portal");
    path.push("test.db");
    path
}

fn get_db_conn() -> Result<rusqlite::Connection, String> {
    let path = get_db_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    rusqlite::Connection::open(path).map_err(|e| e.to_string())
}

fn init_db() -> Result<(), String> {
    let conn = get_db_conn()?;
    conn.execute("PRAGMA foreign_keys = ON;", []).map_err(|e| e.to_string())?;

    // Create accounts table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS accounts (
            account_no INT PRIMARY KEY,
            name TEXT NOT NULL,
            gender TEXT NOT NULL,
            phone TEXT NOT NULL,
            email TEXT NOT NULL,
            balance REAL DEFAULT 0.0,
            account_type TEXT DEFAULT 'Savings',
            status TEXT DEFAULT 'Active',
            pin TEXT DEFAULT '1234',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    ).map_err(|e| e.to_string())?;

    // Safe column additions for backward compatibility with pre-existing sqlite file
    let _ = conn.execute("ALTER TABLE accounts ADD COLUMN account_type TEXT DEFAULT 'Savings'", []);
    let _ = conn.execute("ALTER TABLE accounts ADD COLUMN status TEXT DEFAULT 'Active'", []);
    let _ = conn.execute("ALTER TABLE accounts ADD COLUMN pin TEXT DEFAULT '1234'", []);
    let _ = conn.execute("ALTER TABLE accounts ADD COLUMN created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP", []);

    // Create transactions table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS transactions (
            transaction_id INTEGER PRIMARY KEY AUTOINCREMENT,
            account_no INT NOT NULL,
            type TEXT NOT NULL,
            amount REAL NOT NULL,
            target_account_no INT,
            timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            description TEXT,
            FOREIGN KEY (account_no) REFERENCES accounts(account_no) ON DELETE CASCADE
        )",
        [],
    ).map_err(|e| e.to_string())?;

    let _ = conn.execute("ALTER TABLE transactions ADD COLUMN description TEXT", []);

    // Create loans table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS loans (
            loan_id INTEGER PRIMARY KEY AUTOINCREMENT,
            account_no INT NOT NULL,
            principal REAL NOT NULL,
            interest_rate REAL NOT NULL,
            remaining_balance REAL NOT NULL,
            status TEXT NOT NULL DEFAULT 'Pending',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (account_no) REFERENCES accounts(account_no) ON DELETE CASCADE
        )",
        [],
    ).map_err(|e| e.to_string())?;

    // Create fixed_deposits table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS fixed_deposits (
            fd_id INTEGER PRIMARY KEY AUTOINCREMENT,
            account_no INT NOT NULL,
            amount REAL NOT NULL,
            interest_rate REAL NOT NULL,
            duration_months INTEGER NOT NULL,
            maturity_amount REAL NOT NULL,
            status TEXT NOT NULL DEFAULT 'Active',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            maturity_date TIMESTAMP NOT NULL,
            FOREIGN KEY (account_no) REFERENCES accounts(account_no) ON DELETE CASCADE
        )",
        [],
    ).map_err(|e| e.to_string())?;

    // Prepopulate if database is empty
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM accounts", [], |row| row.get(0)).map_err(|e| e.to_string())?;
    if count == 0 {
        prepopulate_db(&conn)?;
    }

    Ok(())
}

fn prepopulate_db(conn: &rusqlite::Connection) -> Result<(), String> {
    let accs = vec![
        (1011002031, "Amal Dev", "Male", "+91-9447012345", "amaldev@gmail.com", 25000.0, "Savings", "Active", "1234"),
        (1011002032, "Anupama Nair", "Female", "+91-9447567890", "anupama.nair@gmail.com", 78000.0, "Savings", "Active", "1234"),
        (1011002033, "Hariprasad K.", "Male", "+91-9846012345", "hariprasad.k@gmail.com", 150000.0, "Checking", "Active", "1234"),
        (1011002034, "Sreedevi Kurup", "Female", "+91-9846789012", "sreedevi.k@gmail.com", 9200.0, "Savings", "Active", "1234"),
        (1011002035, "Vishnu Prasad", "Male", "+91-9745123456", "vishnu.p@gmail.com", 3500.0, "Savings", "Active", "1234"),
        (1011002036, "Meera Pillai", "Female", "+91-9447123456", "meera.pillai@gmail.com", 45000.0, "Business", "Active", "1234"),
        (1011002037, "Rahul Krishnan", "Male", "+91-9847123456", "rahul.k@gmail.com", 120000.0, "Savings", "Active", "1234"),
        (1011002038, "Divya Mohan", "Female", "+91-9562123456", "divya.mohan@gmail.com", 62000.0, "Checking", "Active", "1234"),
        (1011002039, "Gautham Suresh", "Male", "+91-9746123456", "gautham.s@gmail.com", 8500.0, "Savings", "Active", "1234"),
        (1011002040, "Kavya Madhavan", "Female", "+91-9446123456", "kavya.m@gmail.com", 95000.0, "Savings", "Active", "1234")
    ];

    for a in accs {
        conn.execute(
            "INSERT INTO accounts (account_no, name, gender, phone, email, balance, account_type, status, pin) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)",
            (a.0, a.1, a.2, a.3, a.4, a.5, a.6, a.7, a.8),
        ).map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO transactions (account_no, type, amount, description) VALUES (?, 'Initial Deposit', ?, 'Account opened with initial balance')",
            (a.0, a.5),
        ).map_err(|e| e.to_string())?;
    }

    // Example 1: Active Loan (Amal Dev - 1011002031)
    conn.execute(
        "INSERT INTO loans (account_no, principal, interest_rate, remaining_balance, status) VALUES (1011002031, 50000.0, 12.0, 35000.0, 'Approved')",
        [],
    ).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO transactions (account_no, type, amount, description) VALUES (1011002031, 'Loan Disbursement', 50000.0, 'Disbursement of Loan ID #1')",
        [],
    ).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO transactions (account_no, type, amount, description) VALUES (1011002031, 'Loan Repayment', 15000.0, 'Amortization repayment for Loan ID #1')",
        [],
    ).map_err(|e| e.to_string())?;

    // Example 2: Fully Paid Loan (Hariprasad K. - 1011002033)
    conn.execute(
        "INSERT INTO loans (account_no, principal, interest_rate, remaining_balance, status) VALUES (1011002033, 20000.0, 10.0, 0.0, 'Paid')",
        [],
    ).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO transactions (account_no, type, amount, description) VALUES (1011002033, 'Loan Disbursement', 20000.0, 'Disbursement of Loan ID #2')",
        [],
    ).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO transactions (account_no, type, amount, description) VALUES (1011002033, 'Loan Repayment', 20000.0, 'Final repayment of Loan ID #2')",
        [],
    ).map_err(|e| e.to_string())?;

    // Example 3: Active Fixed Deposit (Vishnu Prasad - 1011002035)
    conn.execute(
        "INSERT INTO fixed_deposits (account_no, amount, interest_rate, duration_months, maturity_amount, status, maturity_date) 
         VALUES (1011002035, 100000.0, 7.0, 12, 107000.0, 'Active', datetime('now', '+12 months'))",
        [],
    ).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO transactions (account_no, type, amount, description) VALUES (1011002035, 'FD Creation', 100000.0, 'Established Fixed Deposit contract #1')",
        [],
    ).map_err(|e| e.to_string())?;

    // Example 4: Matured Fixed Deposit (Kavya Madhavan - 1011002040)
    conn.execute(
        "INSERT INTO fixed_deposits (account_no, amount, interest_rate, duration_months, maturity_amount, status, maturity_date) 
         VALUES (1011002040, 30000.0, 6.0, 6, 30900.0, 'Matured', datetime('now', '-1 months'))",
        [],
    ).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO transactions (account_no, type, amount, description) VALUES (1011002040, 'FD Creation', 30000.0, 'Established Fixed Deposit contract #2')",
        [],
    ).map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO transactions (account_no, type, amount, description) VALUES (1011002040, 'FD Maturity', 30900.0, 'Maturity payout of FD contract #2')",
        [],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

fn gen_acc_no(conn: &rusqlite::Connection) -> Result<i64, String> {
    loop {
        let n: i64 = conn.query_row(
            "SELECT ABS(RANDOM() % 1000000000) + 1010000000",
            [],
            |row| row.get(0)
        ).map_err(|e| e.to_string())?;

        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM accounts WHERE account_no = ?)",
            [n],
            |row| row.get(0)
        ).map_err(|e| e.to_string())?;

        if !exists {
            return Ok(n);
        }
    }
}

// Custom Tauri Commands
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Welcome back, {}!", name)
}

#[tauri::command]
fn get_accounts() -> Result<Vec<Account>, String> {
    let conn = get_db_conn()?;
    let mut stmt = conn.prepare(
        "SELECT account_no, name, gender, phone, email, balance, account_type, status, pin, created_at 
         FROM accounts ORDER BY name ASC"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map([], |row| {
        Ok(Account {
            account_no: row.get(0)?,
            name: row.get(1)?,
            gender: row.get(2)?,
            phone: row.get(3)?,
            email: row.get(4)?,
            balance: row.get(5)?,
            account_type: row.get(6)?,
            status: row.get(7)?,
            pin: row.get(8)?,
            created_at: row.get(9)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut accounts = Vec::new();
    for r in rows {
        accounts.push(r.map_err(|e| e.to_string())?);
    }
    Ok(accounts)
}

#[tauri::command]
fn search_accounts(query: String) -> Result<Vec<Account>, String> {
    let conn = get_db_conn()?;
    let pattern = format!("%{}%", query);
    let mut stmt = conn.prepare(
        "SELECT account_no, name, gender, phone, email, balance, account_type, status, pin, created_at 
         FROM accounts 
         WHERE name LIKE ? OR phone LIKE ? OR email LIKE ? OR CAST(account_no AS TEXT) LIKE ? 
         ORDER BY name ASC"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map([&pattern, &pattern, &pattern, &pattern], |row| {
        Ok(Account {
            account_no: row.get(0)?,
            name: row.get(1)?,
            gender: row.get(2)?,
            phone: row.get(3)?,
            email: row.get(4)?,
            balance: row.get(5)?,
            account_type: row.get(6)?,
            status: row.get(7)?,
            pin: row.get(8)?,
            created_at: row.get(9)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut accounts = Vec::new();
    for r in rows {
        accounts.push(r.map_err(|e| e.to_string())?);
    }
    Ok(accounts)
}

#[tauri::command]
fn create_account(
    name: String, 
    gender: String, 
    phone: String, 
    email: String, 
    balance: f64, 
    account_type: String, 
    pin: String
) -> Result<i64, String> {
    if balance < 500.0 {
        return Err("Minimum deposit of Rs. 500 is required to open an account.".to_string());
    }
    if pin.len() != 4 || !pin.chars().all(|c| c.is_ascii_digit()) {
        return Err("PIN must be exactly 4 digits.".to_string());
    }

    let conn = get_db_conn()?;
    let acc_no = gen_acc_no(&conn)?;

    conn.execute(
        "INSERT INTO accounts (account_no, name, gender, phone, email, balance, account_type, status, pin) 
         VALUES (?, ?, ?, ?, ?, ?, ?, 'Active', ?)",
        (acc_no, &name, &gender, &phone, &email, balance, &account_type, &pin),
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO transactions (account_no, type, amount, description) VALUES (?, 'Initial Deposit', ?, 'Account opened with initial balance')",
        (acc_no, balance),
    ).map_err(|e| e.to_string())?;

    Ok(acc_no)
}

#[tauri::command]
fn update_account(
    account_no: i64, 
    name: String, 
    gender: String, 
    phone: String, 
    email: String, 
    balance: f64, 
    account_type: String, 
    status: String
) -> Result<(), String> {
    let conn = get_db_conn()?;
    conn.execute(
        "UPDATE accounts SET name=?, gender=?, phone=?, email=?, balance=?, account_type=?, status=? 
         WHERE account_no=?",
        (&name, &gender, &phone, &email, balance, &account_type, &status, account_no),
    ).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn delete_account(account_no: i64) -> Result<(), String> {
    let conn = get_db_conn()?;
    conn.execute("DELETE FROM accounts WHERE account_no=?", [account_no]).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn deposit(account_no: i64, amount: f64, description: Option<String>) -> Result<(), String> {
    if amount <= 0.0 {
        return Err("Deposit amount must be positive.".to_string());
    }
    let conn = get_db_conn()?;

    let status: String = conn.query_row(
        "SELECT status FROM accounts WHERE account_no=?",
        [account_no],
        |row| row.get(0)
    ).map_err(|_| "Account not found.".to_string())?;

    if status == "Frozen" {
        return Err("Account is frozen. Transactions are blocked.".to_string());
    }

    conn.execute(
        "UPDATE accounts SET balance=balance+? WHERE account_no=?",
        (amount, account_no),
    ).map_err(|e| e.to_string())?;

    let desc = description.unwrap_or_else(|| "Cash Deposit".to_string());
    conn.execute(
        "INSERT INTO transactions (account_no, type, amount, description) VALUES (?, 'Deposit', ?, ?)",
        (account_no, amount, desc),
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn withdraw(account_no: i64, amount: f64, pin: String, description: Option<String>) -> Result<(), String> {
    if amount <= 0.0 {
        return Err("Withdrawal amount must be positive.".to_string());
    }
    let conn = get_db_conn()?;

    let (status, balance, db_pin): (String, f64, String) = conn.query_row(
        "SELECT status, balance, pin FROM accounts WHERE account_no=?",
        [account_no],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    ).map_err(|_| "Account not found.".to_string())?;

    if status != "Active" {
        return Err(format!("Account is {}. Transaction rejected.", status));
    }
    if db_pin != pin {
        return Err("Invalid transaction PIN.".to_string());
    }
    if balance < amount {
        return Err("Insufficient balance.".to_string());
    }

    conn.execute(
        "UPDATE accounts SET balance=balance-? WHERE account_no=?",
        (amount, account_no),
    ).map_err(|e| e.to_string())?;

    let desc = description.unwrap_or_else(|| "Cash Withdrawal".to_string());
    conn.execute(
        "INSERT INTO transactions (account_no, type, amount, description) VALUES (?, 'Withdrawal', ?, ?)",
        (account_no, amount, desc),
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn transfer(
    sender_no: i64, 
    receiver_no: i64, 
    amount: f64, 
    pin: String, 
    description: Option<String>
) -> Result<String, String> {
    if amount <= 0.0 {
        return Err("Transfer amount must be positive.".to_string());
    }
    if sender_no == receiver_no {
        return Err("Sender and receiver accounts must be different.".to_string());
    }

    let mut conn = get_db_conn()?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    // Sender details
    let (s_status, s_balance, s_pin, s_name): (String, f64, String, String) = tx.query_row(
        "SELECT status, balance, pin, name FROM accounts WHERE account_no=?",
        [sender_no],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    ).map_err(|_| "Sender account not found.".to_string())?;

    if s_status != "Active" {
        return Err(format!("Sender account is {}. Transfer blocked.", s_status));
    }
    if s_pin != pin {
        return Err("Invalid transaction PIN.".to_string());
    }
    if s_balance < amount {
        return Err("Insufficient balance in sender account.".to_string());
    }

    // Receiver details
    let (r_status, r_name): (String, String) = tx.query_row(
        "SELECT status, name FROM accounts WHERE account_no=?",
        [receiver_no],
        |row| Ok((row.get(0)?, row.get(1)?))
    ).map_err(|_| "Receiver account not found.".to_string())?;

    if r_status == "Frozen" {
        return Err("Receiver account is frozen. Transfer blocked.".to_string());
    }

    tx.execute("UPDATE accounts SET balance=balance-? WHERE account_no=?", (amount, sender_no)).map_err(|e| e.to_string())?;
    tx.execute("UPDATE accounts SET balance=balance+? WHERE account_no=?", (amount, receiver_no)).map_err(|e| e.to_string())?;

    let s_desc = description.clone().unwrap_or_else(|| format!("Transfer to {}", r_name));
    let r_desc = description.unwrap_or_else(|| format!("Transfer from {}", s_name));

    tx.execute(
        "INSERT INTO transactions (account_no, type, amount, target_account_no, description) 
         VALUES (?, 'Transfer Out', ?, ?, ?)",
        (sender_no, amount, receiver_no, s_desc),
    ).map_err(|e| e.to_string())?;

    tx.execute(
        "INSERT INTO transactions (account_no, type, amount, target_account_no, description) 
         VALUES (?, 'Transfer In', ?, ?, ?)",
        (receiver_no, amount, sender_no, r_desc),
    ).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(r_name)
}

#[tauri::command]
fn get_transactions() -> Result<Vec<Transaction>, String> {
    let conn = get_db_conn()?;
    let mut stmt = conn.prepare(
        "SELECT t.transaction_id, t.account_no, a.name, t.type, t.amount, t.target_account_no, t.timestamp, t.description 
         FROM transactions t 
         JOIN accounts a ON t.account_no = a.account_no 
         ORDER BY t.timestamp DESC"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map([], |row| {
        Ok(Transaction {
            transaction_id: row.get(0)?,
            account_no: row.get(1)?,
            name: row.get(2)?,
            r#type: row.get(3)?,
            amount: row.get(4)?,
            target_account_no: row.get(5)?,
            timestamp: row.get(6)?,
            description: row.get(7)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut txs = Vec::new();
    for r in rows {
        txs.push(r.map_err(|e| e.to_string())?);
    }
    Ok(txs)
}

#[tauri::command]
fn get_loans() -> Result<Vec<Loan>, String> {
    let conn = get_db_conn()?;
    let mut stmt = conn.prepare(
        "SELECT l.loan_id, l.account_no, a.name, l.principal, l.interest_rate, l.remaining_balance, l.status, l.created_at 
         FROM loans l 
         JOIN accounts a ON l.account_no = a.account_no 
         ORDER BY l.created_at DESC"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map([], |row| {
        Ok(Loan {
            loan_id: row.get(0)?,
            account_no: row.get(1)?,
            name: row.get(2)?,
            principal: row.get(3)?,
            interest_rate: row.get(4)?,
            remaining_balance: row.get(5)?,
            status: row.get(6)?,
            created_at: row.get(7)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut loans = Vec::new();
    for r in rows {
        loans.push(r.map_err(|e| e.to_string())?);
    }
    Ok(loans)
}

#[tauri::command]
fn apply_loan(account_no: i64, principal: f64, interest_rate: f64) -> Result<i64, String> {
    if principal <= 0.0 {
        return Err("Principal must be positive.".to_string());
    }
    if interest_rate <= 0.0 {
        return Err("Interest rate must be positive.".to_string());
    }

    let mut conn = get_db_conn()?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let status: String = tx.query_row(
        "SELECT status FROM accounts WHERE account_no=?",
        [account_no],
        |row| row.get(0)
    ).map_err(|_| "Account not found.".to_string())?;

    if status != "Active" {
        return Err("Loans can only be applied to Active accounts.".to_string());
    }

    tx.execute(
        "INSERT INTO loans (account_no, principal, interest_rate, remaining_balance, status) 
         VALUES (?, ?, ?, ?, 'Approved')",
        (account_no, principal, interest_rate, principal),
    ).map_err(|e| e.to_string())?;

    let loan_id = tx.last_insert_rowid();

    tx.execute(
        "UPDATE accounts SET balance = balance + ? WHERE account_no = ?",
        (principal, account_no),
    ).map_err(|e| e.to_string())?;

    tx.execute(
        "INSERT INTO transactions (account_no, type, amount, description) VALUES (?, 'Loan Disbursement', ?, ?)",
        (account_no, principal, format!("Disbursement of Loan ID #{}", loan_id)),
    ).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(loan_id)
}

#[tauri::command]
fn pay_loan_installment(loan_id: i64, amount: f64, pin: String) -> Result<(), String> {
    if amount <= 0.0 {
        return Err("Payment amount must be positive.".to_string());
    }

    let mut conn = get_db_conn()?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let (account_no, remaining_balance, status): (i64, f64, String) = tx.query_row(
        "SELECT account_no, remaining_balance, status FROM loans WHERE loan_id=?",
        [loan_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    ).map_err(|_| "Loan record not found.".to_string())?;

    if status != "Approved" {
        return Err("This loan is not active or is already paid off.".to_string());
    }

    let (acc_balance, db_pin, acc_status): (f64, String, String) = tx.query_row(
        "SELECT balance, pin, status FROM accounts WHERE account_no=?",
        [account_no],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    ).map_err(|_| "Associated account not found.".to_string())?;

    if acc_status != "Active" {
        return Err(format!("Associated account is {}. Payment rejected.", acc_status));
    }
    if db_pin != pin {
        return Err("Invalid transaction PIN.".to_string());
    }

    let payment = if amount > remaining_balance {
        remaining_balance
    } else {
        amount
    };

    if acc_balance < payment {
        return Err("Insufficient balance in account for payment.".to_string());
    }

    tx.execute("UPDATE accounts SET balance=balance-? WHERE account_no=?", (payment, account_no)).map_err(|e| e.to_string())?;

    let new_remaining = remaining_balance - payment;
    let new_status = if new_remaining <= 0.0 { "Paid" } else { "Approved" };

    tx.execute(
        "UPDATE loans SET remaining_balance=?, status=? WHERE loan_id=?",
        (new_remaining, new_status, loan_id),
    ).map_err(|e| e.to_string())?;

    tx.execute(
        "INSERT INTO transactions (account_no, type, amount, description) VALUES (?, 'Loan Repayment', ?, ?)",
        (account_no, payment, format!("Repayment for Loan ID #{}. Remaining principal: Rs. {:.2}", loan_id, new_remaining)),
    ).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn get_fixed_deposits() -> Result<Vec<FixedDeposit>, String> {
    let conn = get_db_conn()?;
    let mut stmt = conn.prepare(
        "SELECT f.fd_id, f.account_no, a.name, f.amount, f.interest_rate, f.duration_months, f.maturity_amount, f.status, f.created_at, f.maturity_date 
         FROM fixed_deposits f 
         JOIN accounts a ON f.account_no = a.account_no 
         ORDER BY f.created_at DESC"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map([], |row| {
        Ok(FixedDeposit {
            fd_id: row.get(0)?,
            account_no: row.get(1)?,
            name: row.get(2)?,
            amount: row.get(3)?,
            interest_rate: row.get(4)?,
            duration_months: row.get(5)?,
            maturity_amount: row.get(6)?,
            status: row.get(7)?,
            created_at: row.get(8)?,
            maturity_date: row.get(9)?,
        })
    }).map_err(|e| e.to_string())?;

    let mut fds = Vec::new();
    for r in rows {
        fds.push(r.map_err(|e| e.to_string())?);
    }
    Ok(fds)
}

#[tauri::command]
fn create_fixed_deposit(
    account_no: i64, 
    amount: f64, 
    duration_months: i64, 
    pin: String
) -> Result<i64, String> {
    if amount < 1000.0 {
        return Err("Minimum FD amount is Rs. 1,000.".to_string());
    }
    if duration_months < 1 {
        return Err("FD duration must be at least 1 month.".to_string());
    }

    let mut conn = get_db_conn()?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let (acc_balance, db_pin, status): (f64, String, String) = tx.query_row(
        "SELECT balance, pin, status FROM accounts WHERE account_no=?",
        [account_no],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    ).map_err(|_| "Account not found.".to_string())?;

    if status != "Active" {
        return Err(format!("Account is {}. Transaction rejected.", status));
    }
    if db_pin != pin {
        return Err("Invalid transaction PIN.".to_string());
    }
    if acc_balance < amount {
        return Err("Insufficient balance to create Fixed Deposit.".to_string());
    }

    let interest_rate = if duration_months < 6 {
        5.0
    } else if duration_months < 12 {
        6.0
    } else if duration_months <= 24 {
        7.0
    } else {
        7.5
    };

    let maturity_amount = amount * (1.0 + (interest_rate / 100.0) * (duration_months as f64 / 12.0));

    let maturity_date: String = tx.query_row(
        "SELECT datetime('now', ?)",
        [format!("+{} months", duration_months)],
        |row| row.get(0)
    ).map_err(|e| e.to_string())?;

    tx.execute("UPDATE accounts SET balance=balance-? WHERE account_no=?", (amount, account_no)).map_err(|e| e.to_string())?;

    tx.execute(
        "INSERT INTO fixed_deposits (account_no, amount, interest_rate, duration_months, maturity_amount, status, maturity_date) 
         VALUES (?, ?, ?, ?, ?, 'Active', ?)",
        (account_no, amount, interest_rate, duration_months, maturity_amount, &maturity_date),
    ).map_err(|e| e.to_string())?;

    let fd_id = tx.last_insert_rowid();

    tx.execute(
        "INSERT INTO transactions (account_no, type, amount, description) VALUES (?, 'FD Creation', ?, ?)",
        (account_no, amount, format!("Created Fixed Deposit #{}. Maturity: {}", fd_id, maturity_date)),
    ).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(fd_id)
}

#[tauri::command]
fn mature_fixed_deposit(fd_id: i64) -> Result<(), String> {
    let mut conn = get_db_conn()?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let (account_no, amount, maturity_amount, status): (i64, f64, f64, String) = tx.query_row(
        "SELECT account_no, amount, maturity_amount, status FROM fixed_deposits WHERE fd_id=?",
        [fd_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
    ).map_err(|_| "Fixed Deposit record not found.".to_string())?;

    if status != "Active" {
        return Err("This FD is not active.".to_string());
    }

    tx.execute("UPDATE accounts SET balance=balance+? WHERE account_no=?", (maturity_amount, account_no)).map_err(|e| e.to_string())?;
    tx.execute("UPDATE fixed_deposits SET status='Matured' WHERE fd_id=?", [fd_id]).map_err(|e| e.to_string())?;

    tx.execute(
        "INSERT INTO transactions (account_no, type, amount, description) VALUES (?, 'FD Maturity', ?, ?)",
        (account_no, maturity_amount, format!("FD #{} matured. Principal: Rs. {:.2}", fd_id, amount)),
    ).map_err(|e| e.to_string())?;

    tx.commit().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn get_bank_metrics() -> Result<BankMetrics, String> {
    let conn = get_db_conn()?;

    let total_accounts: i64 = conn.query_row("SELECT COUNT(*) FROM accounts", [], |row| row.get(0)).map_err(|e| e.to_string())?;
    let total_assets: f64 = conn.query_row("SELECT COALESCE(SUM(balance), 0.0) FROM accounts", [], |row| row.get(0)).map_err(|e| e.to_string())?;
    let total_transactions: i64 = conn.query_row("SELECT COUNT(*) FROM transactions", [], |row| row.get(0)).map_err(|e| e.to_string())?;

    let avg_balance = if total_accounts > 0 {
        total_assets / total_accounts as f64
    } else {
        0.0
    };

    let top_holder: Option<(String, f64)> = conn.query_row(
        "SELECT name, balance FROM accounts ORDER BY balance DESC LIMIT 1",
        [],
        |row| Ok(Some((row.get(0)?, row.get(1)?)))
    ).unwrap_or(None);

    let (top_holder_name, top_holder_bal) = top_holder.unwrap_or_else(|| ("None".to_string(), 0.0));

    let male_count: i64 = conn.query_row("SELECT COUNT(*) FROM accounts WHERE gender = 'Male'", [], |row| row.get(0)).unwrap_or(0);
    let female_count: i64 = conn.query_row("SELECT COUNT(*) FROM accounts WHERE gender = 'Female'", [], |row| row.get(0)).unwrap_or(0);
    let other_count: i64 = conn.query_row("SELECT COUNT(*) FROM accounts WHERE gender NOT IN ('Male', 'Female')", [], |row| row.get(0)).unwrap_or(0);

    let active_loans_count: i64 = conn.query_row("SELECT COUNT(*) FROM loans WHERE status = 'Approved'", [], |row| row.get(0)).unwrap_or(0);
    let total_loan_balance: f64 = conn.query_row("SELECT COALESCE(SUM(remaining_balance), 0.0) FROM loans WHERE status = 'Approved'", [], |row| row.get(0)).unwrap_or(0.0);

    let active_fds_count: i64 = conn.query_row("SELECT COUNT(*) FROM fixed_deposits WHERE status = 'Active'", [], |row| row.get(0)).unwrap_or(0);
    let total_fd_balance: f64 = conn.query_row("SELECT COALESCE(SUM(amount), 0.0) FROM fixed_deposits WHERE status = 'Active'", [], |row| row.get(0)).unwrap_or(0.0);

    Ok(BankMetrics {
        total_accounts,
        total_assets,
        total_transactions,
        avg_balance,
        top_holder_name,
        top_holder_bal,
        gender_stats: GenderStats {
            male: male_count,
            female: female_count,
            other: other_count,
        },
        active_loans_count,
        total_loan_balance,
        active_fds_count,
        total_fd_balance,
    })
}

#[tauri::command]
fn verify_pin(account_no: i64, pin: String) -> Result<bool, String> {
    let conn = get_db_conn()?;
    let db_pin: String = conn.query_row(
        "SELECT pin FROM accounts WHERE account_no = ?",
        [account_no],
        |row| row.get(0)
    ).map_err(|_| "Account not found.".to_string())?;
    Ok(db_pin == pin)
}

#[tauri::command]
fn change_pin(account_no: i64, old_pin: String, new_pin: String) -> Result<(), String> {
    if new_pin.len() != 4 || !new_pin.chars().all(|c| c.is_ascii_digit()) {
        return Err("New PIN must be exactly 4 digits.".to_string());
    }
    let conn = get_db_conn()?;
    let db_pin: String = conn.query_row(
        "SELECT pin FROM accounts WHERE account_no = ?",
        [account_no],
        |row| row.get(0)
    ).map_err(|_| "Account not found.".to_string())?;

    if db_pin != old_pin {
        return Err("Incorrect current PIN.".to_string());
    }

    conn.execute(
        "UPDATE accounts SET pin = ? WHERE account_no = ?",
        (&new_pin, account_no)
    ).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn wipe_transactions() -> Result<(), String> {
    let conn = get_db_conn()?;
    conn.execute("DELETE FROM transactions", []).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn reset_database() -> Result<(), String> {
    let conn = get_db_conn()?;
    let _ = conn.execute("DROP TABLE IF EXISTS fixed_deposits", []);
    let _ = conn.execute("DROP TABLE IF EXISTS loans", []);
    let _ = conn.execute("DROP TABLE IF EXISTS transactions", []);
    let _ = conn.execute("DROP TABLE IF EXISTS accounts", []);
    init_db()?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(target_os = "linux")]
    {
        // Fix for WebKitGTK DMABUF rendering bug on Linux (NVIDIA/AMD GPU drivers on Arch, Ubuntu, etc.)
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        // Disable accelerated compositing entirely as a last resort for white screen issues
        std::env::set_var("WEBKIT_DISABLE_COMPOSITING_MODE", "1");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
            greet,
            get_accounts,
            search_accounts,
            create_account,
            update_account,
            delete_account,
            deposit,
            withdraw,
            transfer,
            get_transactions,
            get_loans,
            apply_loan,
            pay_loan_installment,
            get_fixed_deposits,
            create_fixed_deposit,
            mature_fixed_deposit,
            get_bank_metrics,
            verify_pin,
            change_pin,
            wipe_transactions,
            reset_database
        ])
        .setup(|app| {
            // Initialize database tables and run migrations
            if let Err(e) = init_db() {
                eprintln!("Error initializing database: {}", e);
            }

            // Retrieve main window instance to apply native operating system styling
            if let Some(_window) = app.get_webview_window("main") {
                // Apply visual vibrancy effects natively on Windows
                #[cfg(target_os = "windows")]
                {
                    use window_vibrancy::{apply_acrylic, apply_mica, apply_blur};
                    // Try Acrylic -> Mica -> Blur fallbacks
                    if let Err(_) = apply_acrylic(&_window, Some((6, 18, 36, 120))) {
                        if let Err(_) = apply_mica(&_window, None) {
                            let _ = apply_blur(&_window, Some((6, 18, 36, 120)));
                        }
                    }
                }

                // Apply visual vibrancy effects natively on macOS
                #[cfg(target_os = "macos")]
                {
                    use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};
                    let _ = apply_vibrancy(&_window, NSVisualEffectMaterial::HudWindow, None, None);
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
