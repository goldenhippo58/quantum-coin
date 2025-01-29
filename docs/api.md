# QuantCoin API Documentation

## Base URL
All API requests should be made to: `http://localhost:8080/`

## Wallet Operations

### Create Wallet
- **Method:** POST
- **Endpoint:** `/wallet/create`
- **Auth:** None required
- **Response:** Creates and returns a new wallet

### Send Funds
- **Method:** POST
- **Endpoint:** `/wallet/send`
- **Auth:** None required
- **Headers:** 
  - Content-Type: application/json
- **Request Body:**
```json
{
    "sender": "SenderPublicKey",
    "to": "ReceiverPublicKey",
    "amount": 50
}
```

## Staking Operations

### Stake Funds
- **Method:** POST
- **Endpoint:** `/stake`
- **Auth:** None required
- **Headers:** 
  - Content-Type: application/json
- **Request Body:**
```json
{
    "wallet": "WalletPublicKey",
    "amount": 100
}
```

### Unstake Funds
- **Method:** POST
- **Endpoint:** `/unstake`
- **Auth:** None required
- **Headers:** 
  - Content-Type: application/json
- **Request Body:**
```json
{
    "wallet": "WalletPublicKey",
    "amount": 50
}
```

### Check Staking Rewards
- **Method:** GET
- **Endpoint:** `/staking/rewards`
- **Auth:** None required
- **Headers:** 
  - Content-Type: application/json
- **Query Parameters:**
  - wallet: WalletPublicKey

## Task Management

### Add Task
- **Method:** POST
- **Endpoint:** `/tasks/add`
- **Auth:** None required
- **Headers:** 
  - Content-Type: application/json
- **Request Body:**
```json
{
    "id": "task1",
    "description": "Perform a simple sum task",
    "data": "base64_encoded_task_data",
    "reward": 50
}
```

### Complete Task
- **Method:** POST
- **Endpoint:** `/tasks/complete`
- **Auth:** None required
- **Headers:** 
  - Content-Type: application/json
- **Request Body:**
```json
{
    "id": "task1"
}
```

### Check Pending Tasks
- **Method:** GET
- **Endpoint:** `/tasks/pending`
- **Auth:** None required
- **Headers:** 
  - Content-Type: application/json

### Check Completed Tasks
- **Method:** GET
- **Endpoint:** `/tasks/completed`
- **Auth:** None required
- **Headers:** 
  - Content-Type: application/json

## Blockchain Information

### Check Blocks
- **Method:** GET
- **Endpoint:** `/blocks`
- **Auth:** None required

### Check Wallets
- **Method:** GET
- **Endpoint:** `/wallets`
- **Auth:** None required