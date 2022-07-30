import UTXOBlockchain from './UTXOBlockchain'



class BtcSwap extends UTXOBlockchain {
  constructor(options) {
    super({
      ...options,
      account: `lbtc`,
      networks: {
        main: {
          name: `liquid testnet`,
        },
        test: {
          name: `testnet`,
        },
      },
    })
  }
}


export default LBTCSwap
