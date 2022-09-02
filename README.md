# Toy Payments Engine
##### Because even payment engines can be fun if they're written in Rust!

**Toy Payments Engine**, or **TPE** as I call it, is a pretend payments processing engine that tracks deposits and withdrawals for multiple clients, as well as disputes, resolutions, and charge-backs for transactions.

###### Disclaimer: Please don't use this in production :)

## Usage
The easiest way to see it in action is using one of the test-example files:
```
cargo run -- ./resources/test-examples/inputs/transactions_1.csv
```

This should give an output that looks something like:
```
2022-09-02T20:21:23.857Z WARN [toy_payments_engine] Invalid transaction event: Dispute event contains an invalid transaction_id: 2 for client 1
2022-09-02T20:21:23.857Z WARN [toy_payments_engine] Invalid withdrawal attempt: Cannot withdraw 2.0000 from client 1 when available amount is 1.0000
client,available,held,total,locked
2,2.0000,0.0000,2.0000,false
1,1.0000,0.0000,1.0000,false
```

### Writing to a file

The logs are written to `stderr`, so we can direct our output into a file like so:
```
cargo run -- ./resources/test-examples/inputs/transactions_1.csv > results.csv
```

And now you can open `./results.csv` to see:
```
client,available,held,total,locked
2,2.0000,0.0000,2.0000,false
1,1.0000,0.0000,1.0000,false
```

### Testing
Running the test suite is as simple as:
```
cargo test
```

# Oh, but great wise Adam, how does it all actually work?
###### Is that too cheesy of a title? Lol.

_TL;DR:_
```
1. Stream CSV lines
  a. Deserialize to TransactionEvents
  b. Process event as Transaction
  c. Process Transaction against Account

2. Build report of accounts
  a. Write as CSV to stdout
```

---

_Full explanation:_

### Part 1: Input
The program expects to read a CSV file with the following structure.

**CSV Headers:**
| **Header** | **Type**                    | **Required** | **Example** |
|------------|-----------------------------|--------------|-------------|
| `type`     | TransactionType (see below) | `True`       | `deposit`   |
| `client`   | Unsigned 16-bit Integer     | `True`       | `123`       |
| `tx`       | Unsigned 32-bit Integer     | `True`       | `456`       |
| `amount`   | Money (see below)           | `False`      | `314.1592`  |

**TransactionType:**
| **TransactionType**  | **Description**                                                           |
|----------------------|---------------------------------------------------------------------------|
| `deposit`            | Deposit amount to account                                                 |
| `withdrawal`         | Withdrawal amount from account                                            |
| `dispute`            | Begin to dispute a transaction, moving amount in question into held funds |
| `resolve`            | Undo a transaction's dispute, moving amount in question out of held funds |
| `chargeback`         | Close a dispute and lock the account                                      |

**Money**:

Money is stored as a Signed 64-bit Integer representing hundredths-of-cents value
ie. `314.1592` is stored as `3141592`. This means the maximum value allowed is `922,337,203,685,477.5807`, and the minimum value allowed is `-922,337,203,685,477.5808`.

And example CSV file might look like:
```
type,       client,    tx,    amount
deposit,         1,     1,       1.0
```

## Part 2: Process

#### Main

You can see in `main` that we have deserialized a single line of our CSV as a `TransactionEvent`, and are now processing it using the `TransactionService`.

- Link to `main`: [src/main.rs:47](https://github.com/adam-bates/toy-payments-engine/blob/main/src/main.rs#L47)

---

#### Transaction Service

This is a nice setup because it doesn't matter how we stream our data in, as long as we can iterate over some events, we can process them!

The `TransactionService` is responsible for taking in a `TransactionEvent`, building/updating a `Transaction`, and calling the `AccountService`.

- Link to `TransactionService`: [src/tpe/services/transaction_service.rs:34](https://github.com/adam-bates/toy-payments-engine/blob/main/src/tpe/services/transaction_service.rs#L34)

---

#### Transaction

A `Transaction` is a finite state-machine representation of a transaction:
```
ValidTransaction
  -> dispute() = DisputedTransaction

DisputedTransaction
  -> resolve() = ValidTransaction
  -> charge_back() = ChargedBackTransaction

ChargedBackTransaction
  -> _
```

- Link to `Transaction`: [src/tpe/models/transactions/transaction.rs:40](https://github.com/adam-bates/toy-payments-engine/blob/main/src/tpe/models/transactions/transaction.rs#L40)

---

#### AccountService

The `AccountService` is responsible for 3 things:

1. Interfacing with data store for accounts
2. Applying transactions to account snapshots (ie. updating funds)
3. Building overview report of all accounts

- Link to `AccountService`: [src/tpe/services/account_service.rs:32](https://github.com/adam-bates/toy-payments-engine/blob/main/src/tpe/services/account_service.rs#L32)

## Part 3: Report

After all processing is completed, we ask the `AccountService` to loop through every `Account`, and build a report using the latest state-snapshots.

This is quite simple really.
- Building the report can be found in the Account Service: [src/tpe/services/account_service.rs:43-72](https://github.com/adam-bates/toy-payments-engine/blob/main/src/tpe/services/account_service.rs#L43-L72)
- Serializing the report can be found in main: [src/main.rs70-88](https://github.com/adam-bates/toy-payments-engine/blob/main/src/main.rs#L70-L88)

# And, well ... that's it!

Thanks for stopping by :) And feel free to reach out if you have any questions or concerns!
