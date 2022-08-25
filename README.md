# TXpigeon-CLI

bitcoin transaction broadcaster written in rust

broadcasts raw bitcoin transaction to the bitcoin network. 

by default, connects to 10 nodes via bitcoin dns seeds, sends an inv message to a random connected node, including the imported transaction, then waits to recieve the GetData message and replies with a TX message which includes the imported raw bitcoin transaction and disconnects the node(it will then connect to a new node so we always have 10 connections). 

We then listens to the other connected nodes for an inv message with the txid of the broadcasted transaction which confirmes that it has propergated around the network and we can safely say that it has reached the mempool.  


A new node will be selected every 10 seconds if we do not recieve the inv message from any of the connected nodes. 

# Commands

Broadcast raw bitcoin transaction to the bitcoin network

USAGE:
    main [OPTIONS] <TX>

ARGS:
    <TX>    Input Raw Bitcoin Transaction

OPTIONS:
    -h, --help          Print help information
    -l                  enable logging
    -n <NETWORK>        Select Network [testnet: TestNet] [default: bitcoin]
    -V, --version       Print version information

