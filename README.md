# Toy Payments Engine 🧸💸🚒
##### Because even payment engines can be fun if they're written in Rust!

**Toy Payments Engine**, or **TPE** as I call it, is a pretend payments processing engine that tracks deposits and withdrawals for multiple clients, as well as disputes, resolutions, and charge-backs for transactions.

###### Disclaimer: Please don't use this in production :)


## Contents

- [Usage](#usage-)
    * [Running](#usage-)
    * [Writing to a file](#writing-to-a-file-%EF%B8%8F)
    * [Testing](#testing-)
- [Under the hood](#oh-but-great-wise-adam-how-does-it-all-actually-work)
    * [Part 0: Assuptions](#part-0-assuptions)
    * [Part 1: Input](#part-1-input-)
    * [Part 2: Append to Ledger](#part-2-append-to-ledger-)
    * [Part 3: Apply changes to Accounts](#part-3-apply-changes-to-accounts-%EF%B8%8F)
    * [Part 4: Report](#part-4-report-)


## Usage 🚴
The easiest way to see it in action is to use the root-level example file:
```
cargo run -- transactions.csv
```

This should give an output that looks something like:
```
2022-09-02T21:32:00.437Z WARN [toy_payments_engine] Invalid withdrawal attempt: Cannot withdraw 3.0000 from client 2 when available amount is 2.0000
client,available,held,total,locked
1,1.5000,0.0000,1.5000,false
2,2.0000,0.0000,2.0000,false
```

### Writing to a file ✍️

Don't worry! The logs are written to `stderr`, so we easily can direct our output into a file like so:
```
cargo run -- transactions.csv > accounts.csv
```

And now you can open `./accounts.csv` to see:
```
client,available,held,total,locked
1,1.5000,0.0000,1.5000,false
2,2.0000,0.0000,2.0000,false
```

### Testing 🧪
Running the test suite is as simple as:
```
cargo test
```

---
I know what you must be asking yourself ...
# Oh, but great wise Adam, how does it all actually work?
###### Is that too cheesy of a title? Lol.

_TL;DR:_
```
1. Stream CSV rows
  a. Deserialize row to Transaction
  b. Append Transaction to Ledger
  c. Apply Ledger changes to Accounts

2. Build report of Accounts
  a. Write as CSV to stdout
```

---

## Part 0: Assuptions

When processing transactions, it's not always clear if a transaction is valid, or how it should affect an account. I've made the following assuptions, which heavily affect the outcome of the program:

### 1. Transaction IDs are globally unique.
_The following is considered invalid:_
```
- Client 1, Transaction 1, Deposit 50
- Client 2, Transaction 1, Deposit 50
```

### 2. Withdrawals cannot be disputed.
_Withdrawals are only valid if the client had enough available to withdrawal. Money comes in through deposits, so any disputes against a client having money they shouldn't, should be against the deposits._

### 3. Input amounts cannot be negative.
_It doesn't make sense to deposit or withdrawal a negative value._

### 4. Maximum amount supported is: `922,337,203,685,477.5807`

_This number is explained below, but since this is a toy project, there's no need to waste memory by supporting more._

### 5. Bad transactions won't kill the application.

_Whether its a deserialize issue, an overflow, or an invalid action due to business logic, the application will mark the transaction as invalid and move on to the next one._

## Part 1: Input 🔠
The program expects to read a CSV file with the following structure.

### Row:
| **Header** | **Type**                    | **Required** | **Example** |
|------------|-----------------------------|--------------|-------------|
| `type`     | TransactionType (see below) | `True`       | `deposit`   |
| `client`   | Unsigned 16-bit Integer     | `True`       | `123`       |
| `tx`       | Unsigned 32-bit Integer     | `True`       | `456`       |
| `amount`   | Money (see below)           | `False`      | `314.1592`  |

### TransactionType:
| **TransactionType**  | **Description**                                                           |
|----------------------|---------------------------------------------------------------------------|
| `deposit`            | Deposit amount to account                                                 |
| `withdrawal`         | Withdrawal amount from account                                            |
| `dispute`            | Begin to dispute a transaction, moving amount in question into held funds |
| `resolve`            | Undo a transaction's dispute, moving amount in question out of held funds |
| `chargeback`         | Close a dispute and lock the account                                      |

### Money:

Money is stored as a Signed 64-bit Integer representing hundredths-of-cents value.

_ie. `314.1592` is stored as `3141592`. This means the maximum value allowed is `922,337,203,685,477.5807`, and the minimum value allowed is `-922,337,203,685,477.5808`._

### Example File:

An example CSV file might look like:
```
type,       client,    tx,    amount
deposit,         1,     1,        10
withdrawal,      1,     2,      7.25
dispute,         1,     1,
```

---

Each line of the CSV is deserialized to an InputEvent.

Each InputEvent is parsed as a Transaction.

- Link to deserialization and parsing calls: [src/main.rs:36-52](https://github.com/adam-bates/toy-payments-engine/blob/main/src/main.rs#L36-L52)

- Link to InputEvent: [src/tpe/input.rs:13](https://github.com/adam-bates/toy-payments-engine/blob/main/src/tpe/input.rs#L13)

- Link to Transaction: [src/tpe/transaction.rs:6](https://github.com/adam-bates/toy-payments-engine/blob/main/src/tpe/transaction.rs#L6)

## Part 2: Append to Ledger 🧾

Transactions are appended to our ledger, following WORM (Write Once, Read Many).

- Link to append call: [src/main.rs:56-59](https://github.com/adam-bates/toy-payments-engine/blob/main/src/main.rs#L56-L59)

- Link to Ledger: [src/tpe/ledger.rs:8](https://github.com/adam-bates/toy-payments-engine/blob/main/src/tpe/ledger.rs#L8)

## Part 3: Apply changes to Accounts ⚙️

We grab the AccountSnapshot for whichever client the transaction is for.

Then we apply new transactions to our snapshot, using our updated ledger.

- Link to apply call: [src/main.rs:63-66](https://github.com/adam-bates/toy-payments-engine/blob/main/src/main.rs#L63-L66)

- Link to apply logic: [src/tpe/snapshots/account_snapshot.rs:103](https://github.com/adam-bates/toy-payments-engine/blob/main/src/tpe/snapshots/account_snapshot.rs#L103)

## Part 4: Report 📈

After all processing is completed, we loop through every `AccountSnapshot`, and build a report.

- Building the report: [src/tpe/snapshots/account_snapshots.rs:26](https://github.com/adam-bates/toy-payments-engine/blob/main/src/tpe/snapshots/account_snapshots.rs#L26)

- Serializing the report: [src/main.rs:82](https://github.com/adam-bates/toy-payments-engine/blob/main/src/main.rs#L82)

# And, well ... that's it! 😁

Thanks for stopping by :) And feel free to reach out if you have any questions or concerns!
