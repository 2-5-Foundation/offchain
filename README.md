# vane implementation for risk free transaction sending for web3 businesses and degens

### What are we solving?

1. Losing funds due wrong address input ( a huge pain currently in web3 as the action is not reversible after sending the transaction ).
2. Losings funds due wrong network selection while sending the transaction.
   
   At some point the address can be correct but the choice of the network can result to loss of funds

### Our Solution

vane act as a safety net for web3 users.

1. Receiver address confirmation
2. Transaction execution simulation
3. Receiver account ownership confirmation after transaction execution and network simulation. 

    As this is crucial to make sure that you control account provided ( receiver ) in X network/blockchain.
4. After all confirmation, vane will route and submit the transaction to X address to the Y network/blokchain.

vane operate as a gurdian for the transaction. This is so needed as take for example sending funds to L2s in Ethereum. It is so easy loosing funds, choosing the network ( L2 ) and at somepoint the address you can only control in certain network and not the other, so the simulation part and confirmation is crucial.

We want to eliminate all fear while sending transactions in web3 and to make sure users have a room to make mistakes whitout costing them a fortune.

### User flow

1.  Initiate sending transaction by providing a wallet address ( *sender* )
2.  A transaction data will be sent to receiver address as notification for receiver confirmation.
3. The sender will check with the receiver to confirm they received the notification, ensuring the address is correct and they are the intended recipient. ( *sender & receiver* )

* After simulation of transaction execution
4. The receiver will have to confirm and verify that he can control the account which received the tokens.

* After the confirmation

5. The transaction will be routed and submitted to the confirmed wallet address and network/blockchain.


With this user flow the receiver and sender have only 2 interactions to make the whole transaction risk free.

And at any point of the confirmation, the sender can stop the transaction from progressing once noticed there is some incorrectness.

----

# Technology

vane is an offchain & onchain solution.
![vane-flow](https://github.com/2A-Labs/offchain/assets/69342343/12586e6f-8e1a-4254-8e2b-4ca7a07d7081)


### Offchain components

1. **AV-layer ( Address Verification Layer )**
    
    - Accepts new transaction requests and put them in a queue per sender & receiver address
    - In the transaction message queue, each transaction object is checked for its state ( Pending Confirmation -> Confirmed ).
    - Confirmed transaction object will be propagated to the execution simulation layer
    - Reverted transactions will be dropped from the message queue

2. **Network Simulation**

    - Spawn a parallel reality of the specified network chain ( Chopsticks & EthSim ( coming soon ))
    - Send the transaction to the spawned simulated network for execution
    - Receiver should verify that is able to control the deposited account in the chain
    - Record the proof of attestation
    - Propagate the transaction to network router layer

3. **Network Router Layer**

    - Send the transaction to the verified and attested address and network.

### Onchain component

1. **Smart contract**

    - Store cached value for verified , confirmed and attested transaction object
    - Have tokens for incentivizing relayers
    - Wallet partnership revenue integration
