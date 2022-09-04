# Toy Payments Engine 🧸💸🚒
##### Because even payment engines can be fun if they're written in Rust!

**Toy Payments Engine**, or **TPE** as I call it, is a pretend payments processing engine that tracks deposits and withdrawals for multiple clients, as well as disputes, resolutions, and charge-backs for transactions.

###### Disclaimer: Please don't use this in production :)

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
_I was on the fence about this one for a while, but decided it doesn't make sense to "charge back" a withdrawal._

### 3. Amount values cannot be input as negative.
_It doesn't make sense to deposit or withdrawal a negative value. Output amounts can still be negative._

### 4. Amount values must be within the following bounds (inclusive):
- Min: `-922,337,203,685,477.5808`
- Max: `922,337,203,685,477.5807`

_The numbers are explained below. But since this is a toy project, there's no need to waste memory by supporting higher numbers._

### 5. Bad transactions won't panic the application.

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

Each line of the CSV is deserialized to an InputEvent.

An InputEvent is parsed as a Transaction.

## Part 2: Append to Ledger 🧾

Transactions are appended to our ledger, following WORM (Write Once, Read Many).

## Part 3: Apply changes to Accounts ⚙️

We grab the AccountSnapshot for whichever client the transaction is for.

Then we apply new transactions to our snapshot, using our updated ledger.

## Part 4: Report 📈

After all processing is completed, we loop through every `AccountSnapshot`, and build a report.

This is quite simple really.

- Building the report can be found in the Account Service: [src/tpe/services/account_service.rs:43-72](https://github.com/adam-bates/toy-payments-engine/blob/main/src/tpe/services/account_service.rs#L43-L72)

- Serializing the report can be found in main: [src/main.rs83-101](https://github.com/adam-bates/toy-payments-engine/blob/main/src/main.rs#L83-L101)

# And, well ... that's it! 😁

Thanks for stopping by :) And feel free to reach out if you have any questions or concerns!
