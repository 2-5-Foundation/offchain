# vane implementation for risk free transaction sending for web3 businesses and degens

**What are we solving?**

1. Losing funds due wrong address input ( a huge pain currently in web3 as the action is not reversible after sending the transaction ).
2. Losings funds due wrong network selection while sending the transaction.
   
   At some point the address can be correct but the choice of the network can result to loss of funds

**Our Solution**

vane act as a safety net for web3 users.

1. Receiver address confirmation
2. Transaction execution simulation
3. Receiver account ownership confirmation after transaction execution and network simulation. 

    As this is crucial to make sure that you control account provided ( receiver ) in X network/blockchain.
4. After all confirmation, vane will route and submit the transaction to X address to the Y network/blokchain.

vane operate as a gurdian for the transaction. This is so needed as take for example sending funds to L2s in Ethereum. It is so easy loosing funds, choosing the network ( L2 ) and at somepoint the address you can only control in certain network and not the other, so the simulation part and confirmation is crucial.

We want to eliminate all fear while sending transactions in web3 and to make sure users have a room to make mistakes whitout costing them a fortune.

**User flow**

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

### Offchain components

1. **AV-layer ( Address Verification Layer )**



2. **Network Simulation**

3. **Network Router Layer**

### Onchain component

1. **Smart contract**