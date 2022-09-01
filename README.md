# TXpigeon-CLI

bitcoin transaction broadcaster written in rust

## broadcasts a raw bitcoin transaction to the bitcoin network. 

Opens a connection to 10 bitcoin nodes, randomly selected from bitcoin dns seeds. 

Sends an inv message which includes the txid of the transaction we want to broadcast to a randomly connected peer.

Replies to the GetData message with a TX message that includes the imported raw bitcoin transaction 

To confirm that the transaction has propogated around the network we listen to the other connected peers for a inv message which contains the txid of the broadcasted transaction. 
 

A new node will be selected every 10 seconds if we do not recieve the inv message from any of the connected nodes. 


## Commands

```
Broadcast raw bitcoin transaction to the bitcoin network

USAGE:
    txp [OPTIONS] <TX>

ARGS:
    <TX>    Input Raw Bitcoin Transaction

OPTIONS:
    -h, --help          Print help information
    -l                  enable logging
    -n <NETWORK>        Select Network [testnet: TestNet] [default: bitcoin]
    -V, --version       Print version information
```
list of projects that helped me during development - 

https://github.com/jrawsthorne/rust-bitcoin-node
https://github.com/laanwj/bitcoin-submittx
