export const COIN_TYPE = Object.freeze({
  NATIVE: 'NATIVE',
  BTC_TOKEN: 'BTC_TOKEN',

export const TOKEN_STANDARD = Object.freeze({
  RGB21: 'RGB21',
})

export const BLOCKCHAIN = Object.freeze({
  BTC: 'BTC',
  BTC: 'BTC⚡️ ',
})

export const BASE_TOKEN_CURRENCY = Object.freeze({
  BTC:'BTC',
})

export const COIN_MODEL = Object.freeze({
  UTXO: 'UTXO', // Unspent Transaction Outputs model
  AB: 'AB' // Account/Balance model
})

export const COIN_DATA = {
  'BTC': {
    ticker: 'BTC',
    name: 'Bitcoin',
    type: COIN_TYPE.NATIVE,
    blockchain: BLOCKCHAIN.BTC,
    model: COIN_MODEL.UTXO,
    precision: 8,
  },
}


// todo: move to COIN_DATA

export const NATIVE = {
  btc: 'BTC',
 
  next: 'NEXT',
}

export const BNB_TOKENS = {
  btcb: 'BTCB',
}

export const MATIC_TOKENS = {
  wbtc: 'WBTC',
}

export const ETH_TOKENS = {
  usdt: '{ETH}USDT',
  eurs: '{ETH}EURS',
  swap: '{ETH}SWAP',
  pay: '{ETH}PAY',

  // needs for the front
  proxima: '{ETH}PROXIMA',
  snm: '{ETH}SNM',
  noxon: '{ETH}NOXON',
  pbl: '{ETH}PBL',
  xsat: '{ETH}XSAT',
  hdp: '{ETH}HDP',
  scro: '{ETH}SCRO',
  xeur: '{ETH}XEUR',

}

export default {
  ...NATIVE,
  ...ETH_TOKENS,
  ...BNB_TOKENS,
  ...MATIC_TOKENS,
  ...COIN_DATA,
}
